## DESIRED WORKFLOW

- User runs `forger` in the terminal and is greeted with a TUI menu
- The TUI Menu has 4 options:
    - Initialize Project
    - Start Vibe
    - Reset Project
    - Quit
- If Initialize Project is selected, the user is greeted with a TUI for project initialization
    ** This includes a menu with configuration settings for the autonomous loop [saves to a .toml file]

    [models]
    reasoning = "opencode/minimax-m2.1-free"   # Planning/reasoning model
    fixer = "opencode/glm-4.7-free"            # Stuck recovery model
    autonomous = "opencode/grok-code"          # Coding model
    [generation]
    complexity = 0.75  # Discrete: 0.0, 0.25, 0.5, 0.75, 1.0
    [autonomous]
    session_timeout_minutes = 15
    idle_timeout_seconds = 600
    auto_commit = true
    max_iterations = 0
    no_progress_threshold = 5  # Triggers fixer model
    [agent]
    single_feature_focus = true
    [paths]
    app_spec_file = ".forger/app_spec.md"
    database = ".forger/progress.db"
    [notifications]
    webhook_url = ""
    webhook_enabled = false
    bot_token = ""
    channel_id = ""
    [ui]
    show_progress = true
- Once the user has filled all the information in, they press the save button which background processes the scaffolding event this contains the bare minimum to be greated with the appspec generation screen again if failure/user exits. 
- Once the files are confirmed to be scaffolded, the user is greeted with a new menu this menu has 4 options
- Generate Spec Details (AI Generated AppSpec that uses the complexity value for how defined/production ready the project is)
- When the spec is generated the user is prompted with a review menu, they are able to change what they want, accept the spec, or quit the app.
- When the User Accepts the spec, Another scaffolding event for the remaining items happens, this contains the full scaffolding of files that forger provides for the Autonomous Loop.
- Once the files are confirmed to be scaffolded, the user is greeted with a new menu this menu has 4 options
- Start Vibe
- Modify Config
- Quit

- If Start Vibe is selected, the user is met with a pager https://github.com/charmbracelet/bubbletea/tree/main/examples/pager , this allows them to monitor the output of OpenCode, the pager will update in real time as OpenCode works on the project. 
