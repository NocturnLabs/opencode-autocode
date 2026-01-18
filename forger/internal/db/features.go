package db

import (
	"database/sql"
	"time"
)

// Feature represents a feature in the database
type Feature struct {
	ID                  int
	Category            string
	Description         string
	Passes              int
	VerificationCommand string
	LastError           sql.NullString
	CreatedAt           time.Time
	UpdatedAt           time.Time
}

// FeatureStep represents a verification step for a feature
type FeatureStep struct {
	ID        int
	FeatureID int
	StepOrder int
	StepText  string
}

// FeatureRepository handles feature database operations
type FeatureRepository struct {
	db *sql.DB
}

// NewFeatureRepository creates a new feature repository
func NewFeatureRepository(db *sql.DB) *FeatureRepository {
	return &FeatureRepository{db: db}
}

// Create creates a new feature
func (r *FeatureRepository) Create(feature *Feature) error {
	result, err := r.db.Exec(`
		INSERT INTO features (category, description, passes, verification_command, last_error)
		VALUES (?, ?, ?, ?, ?)
	`, feature.Category, feature.Description, feature.Passes, feature.VerificationCommand, feature.LastError)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}

	feature.ID = int(id)
	return nil
}

// GetByID retrieves a feature by ID
func (r *FeatureRepository) GetByID(id int) (*Feature, error) {
	var f Feature
	var lastError sql.NullString

	err := r.db.QueryRow(`
		SELECT id, category, description, passes, verification_command, last_error, created_at, updated_at
		FROM features WHERE id = ?
	`, id).Scan(
		&f.ID, &f.Category, &f.Description, &f.Passes, &f.VerificationCommand,
		&lastError, &f.CreatedAt, &f.UpdatedAt,
	)
	if err != nil {
		return nil, err
	}

	f.LastError = lastError
	return &f, nil
}

// GetAll retrieves all features
func (r *FeatureRepository) GetAll() ([]Feature, error) {
	rows, err := r.db.Query(`
		SELECT id, category, description, passes, verification_command, last_error, created_at, updated_at
		FROM features ORDER BY id
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var features []Feature
	for rows.Next() {
		var f Feature
		var lastError sql.NullString

		if err := rows.Scan(
			&f.ID, &f.Category, &f.Description, &f.Passes, &f.VerificationCommand,
			&lastError, &f.CreatedAt, &f.UpdatedAt,
		); err != nil {
			return nil, err
		}

		f.LastError = lastError
		features = append(features, f)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return features, nil
}

// Update updates an existing feature
func (r *FeatureRepository) Update(feature *Feature) error {
	_, err := r.db.Exec(`
		UPDATE features
		SET category = ?, description = ?, passes = ?, verification_command = ?, last_error = ?
		WHERE id = ?
	`, feature.Category, feature.Description, feature.Passes, feature.VerificationCommand,
		feature.LastError, feature.ID)
	return err
}

// Delete deletes a feature by ID
func (r *FeatureRepository) Delete(id int) error {
	_, err := r.db.Exec(`DELETE FROM features WHERE id = ?`, id)
	return err
}

// IncrementPasses increments the pass count for a feature
func (r *FeatureRepository) IncrementPasses(id int) error {
	_, err := r.db.Exec(`
		UPDATE features SET passes = passes + 1 WHERE id = ?
	`, id)
	return err
}

// UpdateError updates the last error for a feature
func (r *FeatureRepository) UpdateError(id int, errorMsg string) error {
	_, err := r.db.Exec(`
		UPDATE features SET last_error = ? WHERE id = ?
	`, errorMsg, id)
	return err
}

// GetPassingCount returns the number of passing features
func (r *FeatureRepository) GetPassingCount() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM features WHERE passes > 0`).Scan(&count)
	return count, err
}

// GetFailingCount returns the number of failing features
func (r *FeatureRepository) GetFailingCount() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM features WHERE passes = 0`).Scan(&count)
	return count, err
}

// AddStep adds a verification step to a feature
func (r *FeatureRepository) AddStep(step *FeatureStep) error {
	result, err := r.db.Exec(`
		INSERT INTO feature_steps (feature_id, step_order, step_text)
		VALUES (?, ?, ?)
	`, step.FeatureID, step.StepOrder, step.StepText)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}

	step.ID = int(id)
	return nil
}

// GetSteps retrieves all steps for a feature
func (r *FeatureRepository) GetSteps(featureID int) ([]FeatureStep, error) {
	rows, err := r.db.Query(`
		SELECT id, feature_id, step_order, step_text
		FROM feature_steps WHERE feature_id = ? ORDER BY step_order
	`, featureID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var steps []FeatureStep
	for rows.Next() {
		var s FeatureStep
		if err := rows.Scan(&s.ID, &s.FeatureID, &s.StepOrder, &s.StepText); err != nil {
			return nil, err
		}
		steps = append(steps, s)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return steps, nil
}
