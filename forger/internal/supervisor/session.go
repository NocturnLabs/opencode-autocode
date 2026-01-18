package supervisor

import (
	"bufio"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// Session handles execution of OpenCode sessions.
// It manages timeouts, output streaming, and stop signals.
type Session struct {
	opencodePath string
	model        string
	timeout      time.Duration
	idleTimeout  time.Duration
	stopChan     chan struct{}
	outputChan   chan string
	errorChan    chan error
	doneChan     chan struct{}
	outputBuffer *strings.Builder
	lastActivity atomic.Int64 // Unix timestamp of last activity
	mu           sync.Mutex
	stopped      atomic.Bool
}

// NewSession creates a new session executor.
// opencodePath is the path to the opencode binary.
// model is the AI model to use.
// timeout is the maximum session duration.
// idleTimeout is the maximum time without output before stopping.
func NewSession(opencodePath, model string, timeout, idleTimeout time.Duration) *Session {
	s := &Session{
		opencodePath: opencodePath,
		model:        model,
		timeout:      timeout,
		idleTimeout:  idleTimeout,
		stopChan:     make(chan struct{}),
		outputChan:   make(chan string, 100),
		errorChan:    make(chan error, 1),
		doneChan:     make(chan struct{}),
		outputBuffer: &strings.Builder{},
	}
	s.lastActivity.Store(time.Now().Unix())
	return s
}

// Execute runs an OpenCode session with the given command.
// Returns the complete output or an error if the session fails.
func (s *Session) Execute(command string, prompt string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), s.timeout)
	defer cancel()

	// Build command
	args := []string{
		"run",
		"--command", command,
		"--model", s.model,
	}

	cmd := exec.CommandContext(ctx, s.opencodePath, args...)
	cmd.Stdin = strings.NewReader(prompt)

	// Capture stdout and stderr
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return "", fmt.Errorf("failed to create stdout pipe: %w", err)
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return "", fmt.Errorf("failed to create stderr pipe: %w", err)
	}

	// Start command
	if err := cmd.Start(); err != nil {
		return "", fmt.Errorf("failed to start opencode: %w", err)
	}

	// Start output reader goroutines
	var readerWg sync.WaitGroup
	readerWg.Add(2)
	go s.readOutput(stdout, &readerWg)
	go s.readOutput(stderr, &readerWg)

	// Start monitors (these don't use the WaitGroup - they exit on stopChan/doneChan)
	idleCtx, idleCancel := context.WithCancel(context.Background())
	defer idleCancel()
	go s.monitorIdleTimeout(idleCtx)
	go s.monitorStopFile(idleCtx)

	// Wait for completion or cancellation
	waitChan := make(chan error, 1)
	go func() {
		waitChan <- cmd.Wait()
	}()

	var result error
	select {
	case err := <-waitChan:
		// Command completed normally
		idleCancel() // Stop monitors
		readerWg.Wait()
		s.closeChannels()
		if err != nil {
			return "", fmt.Errorf("command failed: %w", err)
		}
	case err := <-s.errorChan:
		// Error from monitors (idle timeout, stop file)
		idleCancel()
		if cmd.Process != nil {
			cmd.Process.Kill()
		}
		readerWg.Wait()
		s.closeChannels()
		return "", err
	case <-s.stopChan:
		// Manual stop requested
		idleCancel()
		if cmd.Process != nil {
			cmd.Process.Kill()
		}
		readerWg.Wait()
		s.closeChannels()
		return "", fmt.Errorf("session stopped by user")
	case <-ctx.Done():
		// Context timeout
		idleCancel()
		if cmd.Process != nil {
			cmd.Process.Kill()
		}
		readerWg.Wait()
		s.closeChannels()
		return "", fmt.Errorf("session timed out after %v", s.timeout)
	}

	if result != nil {
		return "", result
	}
	return s.GetOutput(), nil
}

// closeChannels safely closes output and error channels
func (s *Session) closeChannels() {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Only close if not already closed
	select {
	case <-s.doneChan:
		// Already closed
	default:
		close(s.doneChan)
		close(s.outputChan)
		close(s.errorChan)
	}
}

// readOutput reads from a reader and sends to output channel.
// It also updates the lastActivity timestamp for idle detection.
func (s *Session) readOutput(r io.Reader, wg *sync.WaitGroup) {
	defer wg.Done()

	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		line := scanner.Text()

		// Update last activity timestamp
		s.lastActivity.Store(time.Now().Unix())

		// Send to channel (non-blocking to avoid deadlock)
		select {
		case s.outputChan <- line:
		default:
			// Channel full, skip (buffer overflow)
		}

		// Append to buffer
		s.mu.Lock()
		s.outputBuffer.WriteString(line)
		s.outputBuffer.WriteString("\n")
		s.mu.Unlock()
	}
}

// monitorIdleTimeout monitors for idle timeout using atomic timestamp.
// It does not consume from outputChan to avoid interfering with external readers.
func (s *Session) monitorIdleTimeout(ctx context.Context) {
	if s.idleTimeout <= 0 {
		return // Disabled
	}

	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-s.stopChan:
			return
		case <-ticker.C:
			lastActivityUnix := s.lastActivity.Load()
			lastActivity := time.Unix(lastActivityUnix, 0)
			elapsed := time.Since(lastActivity)

			if elapsed > s.idleTimeout {
				select {
				case s.errorChan <- fmt.Errorf("idle timeout: no output for %v", elapsed):
				default:
					// Error channel full
				}
				return
			}
		}
	}
}

// monitorStopFile monitors .opencode-stop file for stop signals.
func (s *Session) monitorStopFile(ctx context.Context) {
	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-s.stopChan:
			return
		case <-ticker.C:
			if _, err := os.Stat(".opencode-stop"); err == nil {
				select {
				case s.errorChan <- fmt.Errorf("stop signal file detected"):
				default:
					// Error channel full
				}
				return
			}
		}
	}
}

// Stop stops the session gracefully.
func (s *Session) Stop() {
	if s.stopped.CompareAndSwap(false, true) {
		close(s.stopChan)
	}
}

// GetOutput returns the complete output collected so far.
func (s *Session) GetOutput() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.outputBuffer.String()
}

// StreamOutput returns a channel that streams output lines.
// The channel is closed when the session completes.
func (s *Session) StreamOutput() <-chan string {
	return s.outputChan
}

// GetErrorChan returns the error channel for monitoring errors.
func (s *Session) GetErrorChan() <-chan error {
	return s.errorChan
}
