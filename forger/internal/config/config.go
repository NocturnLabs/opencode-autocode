package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

// Config represents the simplified forger.toml configuration
type Config struct {
	Models     ModelsConfig     `toml:"models"`
	Autonomous AutonomousConfig `toml:"autonomous"`
	Paths      PathsConfig      `toml:"paths"`
	UI         UIConfig         `toml:"ui"`
}

// ModelsConfig holds model configuration
type ModelsConfig struct {
	Default    string `toml:"default"`
	Autonomous string `toml:"autonomous"`
}

// AutonomousConfig holds autonomous loop configuration
type AutonomousConfig struct {
	SessionTimeoutMinutes int  `toml:"session_timeout_minutes"`
	IdleTimeoutSeconds    int  `toml:"idle_timeout_seconds"`
	AutoCommit            bool `toml:"auto_commit"`
}

// PathsConfig holds path configuration
type PathsConfig struct {
	AppSpecFile string `toml:"app_spec_file"`
	Database    string `toml:"database"`
}

// UIConfig holds UI configuration
type UIConfig struct {
	ShowProgress bool `toml:"show_progress"`
}

// DefaultConfig returns the default configuration
func DefaultConfig() *Config {
	return &Config{
		Models: ModelsConfig{
			Default:    "opencode/glm-4.7-free",
			Autonomous: "opencode/minimax-m2.1-free",
		},
		Autonomous: AutonomousConfig{
			SessionTimeoutMinutes: 15,
			IdleTimeoutSeconds:    600,
			AutoCommit:            true,
		},
		Paths: PathsConfig{
			AppSpecFile: ".forger/app_spec.md",
			Database:    ".forger/progress.db",
		},
		UI: UIConfig{
			ShowProgress: true,
		},
	}
}

// Load loads the config from the specified path, falling back to defaults
func Load(path string) (*Config, error) {
	cfg := DefaultConfig()

	// If path is empty, use default
	if path == "" {
		path = "forger.toml"
	}

	// Check if file exists
	if _, err := os.Stat(path); os.IsNotExist(err) {
		// Return default config if file doesn't exist
		return cfg, nil
	}

	// Read and parse TOML file
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	if err := toml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	return cfg, nil
}

// Save writes the config to the specified path
func Save(cfg *Config, path string) error {
	if path == "" {
		path = "forger.toml"
	}

	// Create directory if needed
	dir := filepath.Dir(path)
	if dir != "." {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return fmt.Errorf("failed to create config directory: %w", err)
		}
	}

	// Write config file
	f, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("failed to create config file: %w", err)
	}
	defer f.Close()

	encoder := toml.NewEncoder(f)
	if err := encoder.Encode(cfg); err != nil {
		return fmt.Errorf("failed to write config: %w", err)
	}

	return nil
}

// Validate checks if the config is valid
func (cfg *Config) Validate() error {
	if cfg.Models.Default == "" {
		return fmt.Errorf("models.default is required")
	}
	if cfg.Models.Autonomous == "" {
		return fmt.Errorf("models.autonomous is required")
	}
	if cfg.Autonomous.SessionTimeoutMinutes <= 0 {
		return fmt.Errorf("autonomous.session_timeout_minutes must be positive")
	}
	if cfg.Autonomous.IdleTimeoutSeconds < 0 {
		return fmt.Errorf("autonomous.idle_timeout_seconds must be non-negative")
	}
	return nil
}
