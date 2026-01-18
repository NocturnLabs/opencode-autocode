package opencode

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"time"
)

// OutputHandler handles streaming output from OpenCode CLI
type OutputHandler interface {
	OnOutput(line string)
	OnError(error string)
	OnComplete()
}

// Client represents OpenCode CLI wrapper
type Client struct {
	binPath string
	model   string
	timeout time.Duration
}

// New creates a new OpenCode client
func New(binPath string) *Client {
	return &Client{
		binPath: binPath,
		timeout: 30 * time.Minute,
	}
}

// SetModel sets the default model
func (c *Client) SetModel(model string) {
	c.model = model
}

// SetTimeout sets the operation timeout
func (c *Client) SetTimeout(timeout time.Duration) {
	c.timeout = timeout
}

// Run executes an OpenCode command and streams output
func (c *Client) Run(command string, prompt string, handler OutputHandler) error {
	// Build command
	args := []string{
		"run",
		"--command", command,
		"--model", c.getModel(),
	}

	// Create command
	cmd := exec.Command(c.binPath, args...)

	// Set up stdin
	cmd.Stdin = strings.NewReader(prompt)

	// Capture stdout and stderr combined
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to create stdout pipe: %w", err)
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return fmt.Errorf("failed to create stderr pipe: %w", err)
	}

	// Start command
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start opencode: %w", err)
	}

	// Stream output
	go c.streamOutput(stdout, handler)
	go c.streamOutput(stderr, handler)

	// Wait for command completion with timeout
	done := make(chan error, 1)
	go func() {
		done <- cmd.Wait()
	}()

	select {
	case err := <-done:
		if err != nil {
			handler.OnError(fmt.Sprintf("Command failed: %v", err))
			return err
		}
		handler.OnComplete()
		return nil
	case <-time.After(c.timeout):
		cmd.Process.Kill()
		handler.OnError(fmt.Sprintf("Command timed out after %v", c.timeout))
		return fmt.Errorf("command timed out after %v", c.timeout)
	}
}

// RunSimple executes an OpenCode command and returns combined output
func (c *Client) RunSimple(command string, prompt string) (string, error) {
	var stdout, stderr bytes.Buffer

	args := []string{
		"run",
		"--command", command,
		"--model", c.getModel(),
	}

	cmd := exec.Command(c.binPath, args...)
	cmd.Stdin = strings.NewReader(prompt)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		return stdout.String(), fmt.Errorf("opencode failed: %w\nstderr: %s", err, stderr.String())
	}

	return stdout.String(), nil
}

// streamOutput reads from a reader and sends to handler
func (c *Client) streamOutput(r io.Reader, handler OutputHandler) {
	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		handler.OnOutput(scanner.Text())
	}
}

// getModel returns the model to use (default or custom)
func (c *Client) getModel() string {
	if c.model != "" {
		return c.model
	}
	return "opencode/glm-4.7-free" // Default
}

// CheckInstallation verifies opencode CLI is available
func (c *Client) CheckInstallation() error {
	cmd := exec.Command(c.binPath, "--version")
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("opencode CLI not found or not executable: %w", err)
	}
	return nil
}

// FindBinary attempts to find the opencode binary
func FindBinary() (string, error) {
	// Check common paths
	paths := []string{
		"opencode",
		"/usr/local/bin/opencode",
		"/usr/bin/opencode",
	}

	for _, path := range paths {
		if _, err := os.Stat(path); err == nil {
			return path, nil
		}
	}

	return "", fmt.Errorf("opencode CLI not found. Please install it first")
}
