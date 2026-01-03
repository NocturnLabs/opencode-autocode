# Example: Properly granular feature INSERTs

# DON'T: One vague feature

opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Implement the game', 0, 'cargo build')"

# DO: Separate testable features (5-15 minimum)

opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Hero entity spawns and renders', 0, 'cargo test test_hero_spawn')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Hero moves upward automatically', 0, 'cargo test test_hero_movement')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Weapon fires projectiles', 0, 'cargo test test_weapon_firing')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Enemies spawn and move', 0, 'cargo test test_enemy_spawn')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Collision detection works', 0, 'cargo test test_collision')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Database persists scores', 0, 'cargo test test_persistence')"
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('style', 'UI displays score', 0, 'cargo test test_ui')"

# Rules:

# - Each feature = ONE testable behavior

# - Use real test commands (not just 'cargo build')

# - Mix 'functional' and 'style' categories
