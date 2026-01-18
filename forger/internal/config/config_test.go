package config

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDefaultConfig(t *testing.T) {
	cfg := DefaultConfig()

	assert.Equal(t, "opencode/glm-4.7-free", cfg.Models.Default)
	assert.Equal(t, "opencode/minimax-m2.1-free", cfg.Models.Autonomous)
	assert.Equal(t, 15, cfg.Autonomous.SessionTimeoutMinutes)
	assert.Equal(t, 600, cfg.Autonomous.IdleTimeoutSeconds)
	assert.True(t, cfg.Autonomous.AutoCommit)
	assert.Equal(t, ".forger/app_spec.md", cfg.Paths.AppSpecFile)
	assert.Equal(t, ".forger/progress.db", cfg.Paths.Database)
	assert.True(t, cfg.UI.ShowProgress)
}

func TestConfigValidate(t *testing.T) {
	tests := []struct {
		name    string
		cfg     *Config
		wantErr bool
	}{
		{
			name:    "valid config",
			cfg:     DefaultConfig(),
			wantErr: false,
		},
		{
			name: "missing default model",
			cfg: func() *Config {
				c := DefaultConfig()
				c.Models.Default = ""
				return c
			}(),
			wantErr: true,
		},
		{
			name: "missing autonomous model",
			cfg: func() *Config {
				c := DefaultConfig()
				c.Models.Autonomous = ""
				return c
			}(),
			wantErr: true,
		},
		{
			name: "negative session timeout",
			cfg: func() *Config {
				c := DefaultConfig()
				c.Autonomous.SessionTimeoutMinutes = -1
				return c
			}(),
			wantErr: true,
		},
		{
			name: "zero idle timeout is allowed",
			cfg: func() *Config {
				c := DefaultConfig()
				c.Autonomous.IdleTimeoutSeconds = 0
				return c
			}(),
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.cfg.Validate()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}
