<project_specification>
<project_name>OpenArena - LLM Benchmarking & Battle Platform</project_name>

  <overview>
    Build a comprehensive clone of lmarena.ai (Chatbot Arena). The application serves as a crowdsourced
    benchmarking platform for Large Language Models (LLMs). It features a "Blind Battle" mode where users
    prompt two anonymous models simultaneously and vote on the better response, a detailed ELO-based
    leaderboard, and a "Side-by-Side" direct comparison mode. The system must support both external APIs
    (OpenAI, Anthropic) and local model inference (Ollama) to allow benchmarking of local open-source models
    against proprietary giants.
  </overview>

<technology_stack>
<frontend>
<framework>React with Vite</framework>
<styling>Tailwind CSS (via CDN) + DaisyUI</styling>
<state_management>Zustand (lightweight global state)</state_management>
<visualization>Recharts (for ELO history graphs)</visualization>
<markdown>React Markdown with remark-gfm</markdown>
<animations>Framer Motion (for voting interactions)</animations>
<port>Only launch on port 5173</port>
</frontend>
<backend>
<runtime>Node.js with Express (or Bun with Hono for better performance)</runtime>
<database>SQLite with better-sqlite3 (migratable to PostgreSQL)</database>
<model_orchestration>Vercel AI SDK</model_orchestration>
<caching>In-memory caching for battle session state</caching>
</backend>
</technology_stack>

  <prerequisites>
    <environment_setup>
      - .env with provider keys (OPENAI_API_KEY, ANTHROPIC_API_KEY)
      - OLLAMA_HOST configured for local model discovery
      - Backend code in /server directory
      - Use bun for package management
    </environment_setup>
  </prerequisites>

<core_features>
<arena_mode_blind_battle> - Split-screen chat interface (Model A vs Model B) - Models are anonymized (e.g., "Model A" and "Model B") - Single prompt input sends to both models simultaneously - Parallel streaming of responses - Voting mechanism enabled only after generation completes - Voting options: "A is better", "B is better", "Tie", "Both Bad" - Reveal model names only after vote is cast - "Regenerate" is disabled in blind mode to preserve fairness - Multi-turn conversation support before voting
</arena_mode_blind_battle>

    <direct_chat_side_by_side>
      - User selects specific models for comparison (e.g., Llama3 vs GPT-4o)
      - Non-blind interface (model names visible)
      - Independent parameter tuning (Temperature, Max Tokens)
      - Useful for prompt engineering and specific debugging
    </direct_chat_side_by_side>

    <leaderboard_system>
      - Global ranking table based on ELO rating system
      - Sortable columns: Rank, Model, Arena Elo, 95% CI, Votes, License
      - Categories filtering (Coding, Creative Writing, Hard Prompts)
      - Last updated timestamp
      - ELO history graph per model
      - Medals/Badges for top 3 models
    </leaderboard_system>

    <model_management>
      - Adapter pattern for different providers (Ollama, OpenAI, Anthropic, Google)
      - Auto-discovery of local Ollama models
      - Model metadata display (Parameter count, License, Context Window)
      - Enable/Disable specific models from the rotation
    </model_management>

    <analysis_dashboard>
      - Win-rate matrices (Heatmap of Model X vs Model Y)
      - Length bias analysis (Does the longer answer always win?)
      - Topic modeling of prompts
      - Export battle data (JSON/CSV) for dataset creation
    </analysis_dashboard>

</core_features>

<database_schema>
<tables>
<models> - id, slug, display_name, provider (ollama/openai/etc) - parameter_count, license, is_active - current_elo, battle_count - created_at
</models>

      <battles>
        - id, session_id, user_ip (hashed)
        - model_a_id, model_b_id
        - winner_model_id (nullable for ties)
        - outcome (a_won, b_won, tie, both_bad)
        - created_at
        - prompt_category (inferred)
      </battles>

      <turns>
        - id, battle_id, turn_number
        - prompt_content
        - response_a_content, response_b_content
        - response_a_time_ms, response_b_time_ms
        - response_a_tokens, response_b_tokens
      </turns>

      <elo_snapshots>
        - id, model_id, score, date
        - ranking_position
      </elo_snapshots>

      <categories>
        - id, name (e.g., "Coding", "Creative")
        - description
      </categories>
    </tables>

</database_schema>

<api_endpoints_summary>
<battle_orchestration> - POST /api/battle/init (Starts session, selects 2 random models) - POST /api/battle/chat (Stream response from both models) - POST /api/battle/vote (Submit vote, reveals identities)
</battle_orchestration>

    <stats_leaderboard>
      - GET /api/leaderboard (Get current standings)
      - GET /api/stats/matrix (Win-rate heatmap data)
      - GET /api/stats/models/:id/history (Elo graph data)
    </stats_leaderboard>

    <models>
      - GET /api/models (List all available models)
      - POST /api/models/sync (Refresh local Ollama models)
    </models>

    <admin>
      - GET /api/admin/battles/export
      - POST /api/admin/recalculate-elo
    </admin>

</api_endpoints_summary>

<ui_layout>
<main_navigation> - Top Navbar: "Arena (Battle)", "Leaderboard", "About", "GitHub" - Dark/Light mode toggle - API Key configuration modal (client-side storage option)
</main_navigation>

    <arena_view>
      - Top: System Instructions / Battle Rules
      - Center: Split pane (50/50 width)
        - Left: Model A Response area
        - Right: Model B Response area
      - Bottom:
        - Voting Bar (Buttons: A is Better, B is Better, Tie, Both Bad)
        - Input Textarea (Auto-expanding)
        - Send Button
    </arena_view>

    <leaderboard_view>
      - Filters/Tabs (Overall, Coding, Hard Prompts)
      - Data Table with sticky header
      - Collapsible row details (show model description/license)
    </leaderboard_view>

</ui_layout>

<design_system>
<color_palette> - Primary: Indigo/Blue (Trust, Data) - Model A Accent: Blue-500 - Model B Accent: Red-500 (Distinct colors to differentiate streams) - Winner Highlight: Gold/Yellow glow - Background: Neutral slate
</color_palette>

    <components>
      <chat_bubble>
        - Distinct styling for User vs Models
        - Markdown rendering
        - LaTeX support
        - Code block highlighting
      </chat_bubble>

      <vote_buttons>
        - Large, clear click targets
        - Hover effects
        - Disabled state during generation
      </vote_buttons>
    </components>

</design_system>

<elo_algorithm_spec>
<methodology> - Bradley-Terry model for pairwise comparisons - Bootstrapping (computing confidence intervals) - K-factor configuration (volatility of rating changes) - Separate ratings for different languages or categories if tagged
</methodology>
</elo_algorithm_spec>

<implementation_steps>
<step number="1">
<title>Backend & Model Abstraction</title>
<tasks> - Set up Express/Hono server with SQLite - Create unified `ModelInterface` class - Implement `OllamaProvider` and `OpenAIProvider` - Create SSE (Server Sent Events) handler for dual streams
</tasks>
</step>

    <step number="2">
      <title>Arena UI Skeleton</title>
      <tasks>
        - Build split-pane layout
        - Implement dual-streaming frontend logic
        - Create "Masking" logic (hide model names until vote)
        - Build voting component
      </tasks>
    </step>

    <step number="3">
      <title>Battle Logic & State</title>
      <tasks>
        - Implement battle session tracking
        - Handle edge cases (one model fails, one is slow)
        - Persist battle turns to database
        - Implement vote submission endpoint
      </tasks>
    </step>

    <step number="4">
      <title>Leaderboard & ELO</title>
      <tasks>
        - Create ELO calculation script/service
        - Build Leaderboard table with sorting
        - Implement Recharts for history graphing
      </tasks>
    </step>

</implementation_steps>

<success_criteria>
<functionality> - Dual streaming works without "crosstalk" or lag - Local Ollama models can battle GPT-4 - ELO ratings update correctly after matches - Mobile view stacks models vertically instead of split-pane
</functionality>
<performance> - Initial load under 1.5s - Streaming start latency under 500ms (for local models)
</performance>
</success_criteria>
</project_specification>
