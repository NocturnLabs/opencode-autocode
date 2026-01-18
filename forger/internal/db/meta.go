package db

import (
	"database/sql"
	"time"
)

// Meta represents a metadata entry
type Meta struct {
	Key       string
	Value     string
	UpdatedAt time.Time
}

// MetaRepository handles metadata operations
type MetaRepository struct {
	db *sql.DB
}

// NewMetaRepository creates a new meta repository
func NewMetaRepository(db *sql.DB) *MetaRepository {
	return &MetaRepository{db: db}
}

// Set sets a metadata value
func (r *MetaRepository) Set(key, value string) error {
	_, err := r.db.Exec(`
		INSERT INTO meta (key, value)
		VALUES (?, ?)
		ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = datetime('now')
	`, key, value, value)
	return err
}

// Get retrieves a metadata value by key
func (r *MetaRepository) Get(key string) (string, error) {
	var value string
	err := r.db.QueryRow(`SELECT value FROM meta WHERE key = ?`, key).Scan(&value)
	if err == sql.ErrNoRows {
		return "", nil
	}
	return value, err
}

// GetAll retrieves all metadata
func (r *MetaRepository) GetAll() ([]Meta, error) {
	rows, err := r.db.Query(`SELECT key, value, updated_at FROM meta ORDER BY key`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var meta []Meta
	for rows.Next() {
		var m Meta
		if err := rows.Scan(&m.Key, &m.Value, &m.UpdatedAt); err != nil {
			return nil, err
		}
		meta = append(meta, m)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return meta, nil
}

// Delete deletes a metadata entry by key
func (r *MetaRepository) Delete(key string) error {
	_, err := r.db.Exec(`DELETE FROM meta WHERE key = ?`, key)
	return err
}
