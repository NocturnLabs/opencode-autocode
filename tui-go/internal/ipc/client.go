// Package ipc provides the client for communicating with the Rust engine.
package ipc

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"sync"
)

// Client handles bidirectional JSON-RPC communication with the Rust engine.
type Client struct {
	reader   *bufio.Reader
	writer   io.Writer
	mu       sync.Mutex
	debug    bool
	handlers map[string]EventHandler
}

// EventHandler is a callback function for handling incoming events.
type EventHandler func(payload json.RawMessage) error

// NewClient creates a new IPC client using stdin/stdout.
func NewClient() *Client {
	return &Client{
		reader:   bufio.NewReader(os.Stdin),
		writer:   os.Stdout,
		debug:    os.Getenv("OPENCODE_RPC_DEBUG") == "1",
		handlers: make(map[string]EventHandler),
	}
}

// NewClientWithIO creates a new IPC client with custom reader/writer (for testing).
func NewClientWithIO(reader io.Reader, writer io.Writer) *Client {
	return &Client{
		reader:   bufio.NewReader(reader),
		writer:   writer,
		debug:    os.Getenv("OPENCODE_RPC_DEBUG") == "1",
		handlers: make(map[string]EventHandler),
	}
}

// SetDebug enables or disables debug logging.
func (c *Client) SetDebug(enabled bool) {
	c.debug = enabled
}

// OnEvent registers a handler for a specific event type.
func (c *Client) OnEvent(name string, handler EventHandler) {
	c.handlers[name] = handler
}

// SendCommand sends a command to the Rust engine.
func (c *Client) SendCommand(name string, payload interface{}) error {
	msg := Message{
		ProtocolVersion: ProtocolVersion,
		Direction:       DirectionGoToRust,
		Type:            MessageTypeCommand,
		Name:            name,
		Payload:         payload,
	}
	return c.send(msg)
}

// send marshals and writes a message followed by a newline.
func (c *Client) send(msg Message) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	data, err := json.Marshal(msg)
	if err != nil {
		return fmt.Errorf("failed to marshal message: %w", err)
	}

	if c.debug {
		fmt.Fprintf(os.Stderr, "[IPC DEBUG] OUT: %s\n", string(data))
	}

	if _, err := c.writer.Write(append(data, '\n')); err != nil {
		return fmt.Errorf("failed to write message: %w", err)
	}

	return nil
}

// ReadMessage reads and parses a single message from the input stream.
func (c *Client) ReadMessage() (*Message, error) {
	line, err := c.reader.ReadBytes('\n')
	if err != nil {
		return nil, err
	}

	if c.debug {
		fmt.Fprintf(os.Stderr, "[IPC DEBUG] IN: %s", string(line))
	}

	var msg Message
	if err := json.Unmarshal(line, &msg); err != nil {
		return nil, fmt.Errorf("failed to parse message: %w", err)
	}

	// Validate protocol version
	if msg.ProtocolVersion != "" && msg.ProtocolVersion != ProtocolVersion {
		return nil, fmt.Errorf(
			"protocol version mismatch: expected %s, got %s. Please ensure both binaries are from the same release",
			ProtocolVersion,
			msg.ProtocolVersion,
		)
	}

	return &msg, nil
}

// ReadLoop continuously reads messages and dispatches to registered handlers.
// It returns when EOF is reached or an unrecoverable error occurs.
func (c *Client) ReadLoop(msgChan chan<- *Message) error {
	for {
		msg, err := c.ReadMessage()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		msgChan <- msg
	}
}

// ParsePayload unmarshals a raw JSON payload into the specified type.
func ParsePayload[T any](payload interface{}) (*T, error) {
	if payload == nil {
		return nil, fmt.Errorf("payload is nil")
	}

	// Handle case where payload is already json.RawMessage
	if raw, ok := payload.(json.RawMessage); ok {
		var result T
		if err := json.Unmarshal(raw, &result); err != nil {
			return nil, fmt.Errorf("failed to parse payload: %w", err)
		}
		return &result, nil
	}

	// Handle case where payload was unmarshaled as map[string]interface{}
	data, err := json.Marshal(payload)
	if err != nil {
		return nil, fmt.Errorf("failed to re-marshal payload: %w", err)
	}

	var result T
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, fmt.Errorf("failed to parse payload: %w", err)
	}

	return &result, nil
}
