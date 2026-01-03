# Example: Vibe loop workflow phases

Phase 1: auto-init
└─ Runs if database is empty
└─ Populates features, creates conductor context

Phase 2: auto-context
└─ Runs if .conductor/ doesn't exist
└─ Creates product.md, tech_stack.md

Phase 3: auto-continue (active track)
└─ Runs if tracks/<name>/plan.md has incomplete tasks
└─ Works on next task in the plan

Phase 4: completion check
└─ If all features pass → DONE!

Phase 5: auto-continue (new feature)
└─ Picks next failing feature from database
└─ Implements using @coder subagent
