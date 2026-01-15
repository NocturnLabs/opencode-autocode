// Package main is the entry point for the opencode-forger-tui executable.
// This TUI client communicates with the Rust engine via JSON-RPC over stdin/stdout.
package main

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/yumlabs-tools/opencode-forger/tui-go/internal/ipc"
	"github.com/yumlabs-tools/opencode-forger/tui-go/internal/ui"
)

// Version is set at build time via -ldflags.
var Version = "dev"

func main() {
	// Handle --version flag
	if len(os.Args) > 1 && (os.Args[1] == "--version" || os.Args[1] == "-v") {
		fmt.Printf("opencode-forger-tui %s (protocol %s)\n", Version, ipc.ProtocolVersion)
		os.Exit(0)
	}

	// Handle --help flag
	if len(os.Args) > 1 && (os.Args[1] == "--help" || os.Args[1] == "-h") {
		printHelp()
		os.Exit(0)
	}

	// Check for headless mode (for testing)
	headless := os.Getenv("OPENCODE_TUI_HEADLESS") == "1"
	if headless {
		runHeadless()
		return
	}

	// Create IPC client
	client := ipc.NewClient()

	// Create and run the Bubble Tea program
	model := ui.NewModel(client)
	p := tea.NewProgram(model, tea.WithAltScreen())

	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "Error running TUI: %v\n", err)
		os.Exit(1)
	}
}

func printHelp() {
	fmt.Println(`opencode-forger-tui - Interactive TUI for OpenCode Forger

USAGE:
    opencode-forger-tui [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --version    Print version information

ENVIRONMENT VARIABLES:
    OPENCODE_RPC_DEBUG=1     Enable IPC debug logging to stderr
    OPENCODE_TUI_HEADLESS=1  Run in headless mode (for testing)

DESCRIPTION:
    This is the interactive TUI frontend for OpenCode Forger. It is normally
    launched automatically by the main opencode-forger binary when running
    in interactive mode (--interactive).

    The TUI communicates with the Rust engine via JSON-RPC-style messages
    over stdin/stdout. This allows the UI to be developed and tested
    independently from the core engine logic.

PROTOCOL:
    Messages are newline-delimited JSON (NDJSON) with the following envelope:
    {
      "protocol_version": "1.0.0",
      "direction": "rust->go" | "go->rust",
      "type": "event" | "command",
      "name": "<message_name>",
      "payload": { ... }
    }`)
}

// runHeadless runs in headless mode for integration testing.
// It reads/writes IPC messages without rendering a TUI.
func runHeadless() {
	client := ipc.NewClient()
	msgChan := make(chan *ipc.Message, 100)

	// Start reading messages
	go func() {
		if err := client.ReadLoop(msgChan); err != nil {
			fmt.Fprintf(os.Stderr, "IPC error: %v\n", err)
		}
		close(msgChan)
	}()

	// Process messages until EOF
	for msg := range msgChan {
		if msg == nil {
			continue
		}

		// In headless mode, auto-respond to prompts
		switch msg.Name {
		case ipc.EventEngineReady:
			fmt.Fprintf(os.Stderr, "[HEADLESS] Engine ready\n")

		case ipc.EventModeList:
			// Auto-select first mode
			if err := client.SendCommand(ipc.CommandSelectMode, ipc.SelectModePayload{
				ModeID: "generated",
			}); err != nil {
				fmt.Fprintf(os.Stderr, "[HEADLESS] Error sending command: %v\n", err)
			}

		case ipc.EventUserPrompt:
			payload, _ := ipc.ParsePayload[ipc.UserPromptPayload](msg.Payload)
			if payload != nil && len(payload.Options) > 0 {
				if err := client.SendCommand(ipc.CommandHandleSelection, ipc.HandleSelectionPayload{
					PromptID: payload.PromptID,
					Value:    payload.Options[0],
					Index:    0,
				}); err != nil {
					fmt.Fprintf(os.Stderr, "[HEADLESS] Error sending command: %v\n", err)
				}
			}

		case ipc.EventFinished:
			fmt.Fprintf(os.Stderr, "[HEADLESS] Finished\n")
			return

		case ipc.EventError:
			payload, _ := ipc.ParsePayload[ipc.ErrorPayload](msg.Payload)
			if payload != nil {
				fmt.Fprintf(os.Stderr, "[HEADLESS] Error: %s\n", payload.Message)
				if payload.Fatal {
					os.Exit(1)
				}
			}
		}
	}
}
