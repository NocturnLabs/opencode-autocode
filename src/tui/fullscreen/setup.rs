use anyhow::Result;
use iocraft::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::types::{InteractiveMode, SetupResult};
use crate::config::Config;
use crate::services::scaffold::{scaffold_custom, scaffold_default};

/// Application phase for the fullscreen TUI
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    #[default]
    SetupChoice,
    ModeSelection,
    Done,
}

#[derive(Props, Default)]
struct InteractiveSetupProps {
    has_existing_config: bool,
    result: Arc<Mutex<Option<SetupResult>>>,
}

#[component]
fn InteractiveSetup(
    props: &InteractiveSetupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();

    let mut selected_mode = hooks.use_state(|| 0usize);
    let mut setup_choice = hooks.use_state(|| 0usize);
    let show_reconfigure = hooks.use_state(|| props.has_existing_config);
    let mut reconfigure_selected = hooks.use_state(|| false);
    let mut phase = hooks.use_state(|| Phase::SetupChoice);
    let mut should_exit = hooks.use_state(|| false);
    let mut result_mode = hooks.use_state(|| None::<usize>);

    let modes = InteractiveMode::all();

    hooks.use_terminal_events(move |event| match event {
        TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
            match phase.get() {
                Phase::SetupChoice => {
                    if show_reconfigure.get() {
                        // Yes/No for reconfigure
                        match code {
                            KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Char('h')
                            | KeyCode::Char('l') => {
                                reconfigure_selected.set(!reconfigure_selected.get());
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                reconfigure_selected.set(true);
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                reconfigure_selected.set(false);
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Enter => {
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                should_exit.set(true);
                            }
                            _ => {}
                        }
                    } else {
                        // Quick start vs Configure
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                if setup_choice.get() > 0 {
                                    setup_choice.set(setup_choice.get() - 1);
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if setup_choice.get() < 1 {
                                    setup_choice.set(setup_choice.get() + 1);
                                }
                            }
                            KeyCode::Enter => {
                                phase.set(Phase::ModeSelection);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                should_exit.set(true);
                            }
                            _ => {}
                        }
                    }
                }
                Phase::ModeSelection => match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected_mode.get() > 0 {
                            selected_mode.set(selected_mode.get() - 1);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected_mode.get() < modes.len() - 1 {
                            selected_mode.set(selected_mode.get() + 1);
                        }
                    }
                    KeyCode::Enter => {
                        result_mode.set(Some(selected_mode.get()));
                        phase.set(Phase::Done);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        should_exit.set(true);
                    }
                    _ => {}
                },
                Phase::Done => {}
            }
        }
        _ => {}
    });

    if should_exit.get() || phase.get() == Phase::Done {
        if phase.get() == Phase::Done {
            let mut res = props.result.lock().unwrap();
            *res = Some(SetupResult {
                mode: Some(modes[selected_mode.get()]),
                should_configure: !props.has_existing_config && setup_choice.get() == 1,
                reconfigure: props.has_existing_config && reconfigure_selected.get(),
            });
        }
        system.exit();
    }

    let _current_mode = modes.get(selected_mode.get()).copied().unwrap_or_default();

    element! {
        View(
            width,
            height,
            flex_direction: FlexDirection::Column,
        ) {
            // Header with progress indicator
            View(
                border_style: BorderStyle::Round,
                border_color: Color::DarkGrey,
                padding: 1,
                margin_bottom: 1,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
            ) {
                Text(
                    content: "OpenCode Forger",
                    weight: Weight::Bold,
                    color: Color::White,
                )
                // Progress indicator
                View(flex_direction: FlexDirection::Row) {
                    Text(
                        content: format!("Step {} of 2", if phase.get() == Phase::SetupChoice { 1 } else { 2 }),
                        color: Color::Grey,
                    )
                    Text(content: "  ", color: Color::DarkGrey)
                    Text(
                        content: if phase.get() == Phase::SetupChoice { "● ○" } else { "● ●" },
                        color: Color::Cyan,
                    )
                }
            }

            // Main content area
            View(
                flex_grow: 1.0,
                padding: 1,
                flex_direction: FlexDirection::Column,
            ) {
                #(match phase.get() {
                    Phase::SetupChoice => {
                        if show_reconfigure.get() {
                            element! {
                                View(flex_direction: FlexDirection::Column) {
                                    Text(
                                        content: "Existing configuration found.",
                                        color: Color::White,
                                        weight: Weight::Bold,
                                    )
                                    View(margin_top: 1) {
                                        Text(content: "Reconfigure? ", color: Color::Grey)
                                        Text(
                                            content: "[Yes]",
                                            color: if reconfigure_selected.get() { Color::White } else { Color::DarkGrey },
                                            weight: if reconfigure_selected.get() { Weight::Bold } else { Weight::Normal },
                                        )
                                        Text(content: " / ", color: Color::DarkGrey)
                                        Text(
                                            content: "[No]",
                                            color: if !reconfigure_selected.get() { Color::White } else { Color::DarkGrey },
                                            weight: if !reconfigure_selected.get() { Weight::Bold } else { Weight::Normal },
                                        )
                                    }
                                    Text(
                                        content: "←/→ toggle, Enter confirm",
                                        color: Color::DarkGrey,
                                    )
                                }
                            }
                        } else {
                            element! {
                                View(flex_direction: FlexDirection::Column) {
                                    Text(
                                        content: "Setup Mode",
                                        weight: Weight::Bold,
                                        color: Color::White,
                                    )
                                    View(margin_top: 1, flex_direction: FlexDirection::Column) {
                                        View {
                                            Text(
                                                content: if setup_choice.get() == 0 { "› " } else { "  " },
                                                color: Color::Cyan,
                                            )
                                            Text(
                                                content: "Quick start (use defaults)",
                                                color: if setup_choice.get() == 0 { Color::White } else { Color::Grey },
                                                weight: if setup_choice.get() == 0 { Weight::Bold } else { Weight::Normal },
                                            )
                                        }
                                        View {
                                            Text(
                                                content: if setup_choice.get() == 1 { "› " } else { "  " },
                                                color: Color::Cyan,
                                            )
                                            Text(
                                                content: "Configure settings first",
                                                color: if setup_choice.get() == 1 { Color::White } else { Color::Grey },
                                                weight: if setup_choice.get() == 1 { Weight::Bold } else { Weight::Normal },
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Phase::ModeSelection => {
                        element! {
                            View(flex_direction: FlexDirection::Column) {
                                Text(
                                    content: "Select Project Mode",
                                    weight: Weight::Bold,
                                    color: Color::White,
                                )
                                // Mode cards in a 2x2 grid-like layout
                                View(margin_top: 2, flex_direction: FlexDirection::Column) {
                                    #(modes.iter().enumerate().map(|(i, mode)| {
                                        let is_selected = i == selected_mode.get();
                                        element! {
                                            View(
                                                border_style: BorderStyle::Round,
                                                border_color: if is_selected { Color::Cyan } else { Color::DarkGrey },
                                                padding: 1,
                                                margin_bottom: 1,
                                                flex_direction: FlexDirection::Column,
                                            ) {
                                                View(flex_direction: FlexDirection::Row) {
                                                    Text(
                                                        content: if is_selected { "› " } else { "  " },
                                                        color: Color::Cyan,
                                                    )
                                                    Text(
                                                        content: mode.label(),
                                                        color: if is_selected { Color::White } else { Color::Grey },
                                                        weight: if is_selected { Weight::Bold } else { Weight::Normal },
                                                    )
                                                }
                                                View(padding_left: 2, margin_top: 0) {
                                                    Text(
                                                        content: mode.description(),
                                                        color: Color::DarkGrey,
                                                    )
                                                }
                                            }
                                        }
                                    }))
                                }
                            }
                        }
                    }
                    Phase::Done => {
                        element! {
                            View {
                                Text(content: "Starting...", color: Color::Green)
                            }
                        }
                    }
                })
            }

            // Footer
            View(
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey,
                padding: 1,
            ) {
                Text(
                    content: "↑↓ Navigate  Enter Select  q Quit",
                    color: Color::DarkGrey,
                )
            }
        }
    }
}

/// Run the fullscreen interactive setup and return the selected mode
pub fn run_fullscreen_setup(has_existing_config: bool) -> Result<SetupResult> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    smol::block_on(async {
        element! {
            InteractiveSetup(has_existing_config: has_existing_config, result: result_clone)
        }
        .fullscreen()
        .await
    })?;

    let res = result.lock().unwrap().take();
    Ok(res.unwrap_or(SetupResult {
        mode: None,
        should_configure: false,
        reconfigure: false,
    }))
}

pub fn run_interactive(output_dir: &Path, use_subagents: bool) -> Result<()> {
    let config_path = output_dir.join(".forger/config.toml");
    let has_existing_config = config_path.exists();

    // Run fullscreen TUI for mode selection
    let result = run_fullscreen_setup(has_existing_config)?;

    if result.mode.is_none() {
        println!("Cancelled.");
        return Ok(());
    }

    // Handle configuration
    let config = if result.reconfigure || result.should_configure {
        crate::config_tui::run_config_tui(Some(output_dir))?
    } else {
        Config::load(Some(output_dir)).unwrap_or_default()
    };

    // Execute the selected mode
    match result.mode {
        Some(InteractiveMode::Generated) => {
            crate::tui::generated::run_generated_mode(output_dir, &config, use_subagents)
        }
        Some(InteractiveMode::Manual) => crate::tui::manual::run_manual_mode(output_dir),
        Some(InteractiveMode::FromSpecFile) => run_from_spec_file_mode(output_dir),
        Some(InteractiveMode::Default) => run_default_mode(output_dir),
        None => unreachable!(),
    }
}

fn run_from_spec_file_mode(output_dir: &Path) -> Result<()> {
    use crate::tui::prompts::input;
    println!("\n📁 Spec File Mode");
    let spec_path = input("Path to spec file", Some("app_spec.md"))?;

    let spec_path = std::path::PathBuf::from(&spec_path);
    if !spec_path.exists() {
        println!("❌ Spec file not found: {}", spec_path.display());
        return Ok(());
    }

    scaffold_custom(output_dir, &spec_path)?;
    println!("\n✅ Project scaffolded from spec file!");
    Ok(())
}

fn run_default_mode(output_dir: &Path) -> Result<()> {
    use crate::tui::prompts::confirm;

    println!("\n⚡ Default Mode");
    println!("Using the built-in default specification.");

    if confirm("Scaffold project with default spec?", true)? {
        scaffold_default(output_dir)?;
        println!("\n✅ Project scaffolded with default spec!");
    } else {
        println!("❌ Cancelled.");
    }
    Ok(())
}
