package db

import (
	"database/sql"
	"time"
)

// Knowledge represents a knowledge base entry
type Knowledge struct {
	Key         string
	Value       string
	Category    string
	Description sql.NullString
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

// KnowledgeRepository handles knowledge base operations
type KnowledgeRepository struct {
	db *sql.DB
}

// NewKnowledgeRepository creates a new knowledge repository
func NewKnowledgeRepository(db *sql.DB) *KnowledgeRepository {
	return &KnowledgeRepository{db: db}
}

// Create creates a new knowledge entry
func (r *KnowledgeRepository) Create(knowledge *Knowledge) error {
	_, err := r.db.Exec(`
		INSERT INTO knowledge (key, value, category, description)
		VALUES (?, ?, ?, ?)
	`, knowledge.Key, knowledge.Value, knowledge.Category, knowledge.Description)
	return err
}

// GetByKey retrieves a knowledge entry by key
func (r *KnowledgeRepository) GetByKey(key string) (*Knowledge, error) {
	var k Knowledge
	var description sql.NullString

	err := r.db.QueryRow(`
		SELECT key, value, category, description, created_at, updated_at
		FROM knowledge WHERE key = ?
	`, key).Scan(
		&k.Key, &k.Value, &k.Category, &description,
		&k.CreatedAt, &k.UpdatedAt,
	)
	if err != nil {
		return nil, err
	}

	k.Description = description
	return &k, nil
}

// GetAll retrieves all knowledge entries
func (r *KnowledgeRepository) GetAll() ([]Knowledge, error) {
	rows, err := r.db.Query(`
		SELECT key, value, category, description, created_at, updated_at
		FROM knowledge ORDER BY category, key
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var knowledge []Knowledge
	for rows.Next() {
		var k Knowledge
		var description sql.NullString

		if err := rows.Scan(
			&k.Key, &k.Value, &k.Category, &description,
			&k.CreatedAt, &k.UpdatedAt,
		); err != nil {
			return nil, err
		}

		k.Description = description
		knowledge = append(knowledge, k)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return knowledge, nil
}

// Update updates an existing knowledge entry
func (r *KnowledgeRepository) Update(knowledge *Knowledge) error {
	_, err := r.db.Exec(`
		UPDATE knowledge
		SET value = ?, category = ?, description = ?
		WHERE key = ?
	`, knowledge.Value, knowledge.Category, knowledge.Description, knowledge.Key)
	return err
}

// Delete deletes a knowledge entry by key
func (r *KnowledgeRepository) Delete(key string) error {
	_, err := r.db.Exec(`DELETE FROM knowledge WHERE key = ?`, key)
	return err
}
