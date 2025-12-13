//! Interactive spec editor using ratatui
//!
//! A TUI application for editing project specifications with
//! tree navigation, editing, and live preview.

use anyhow::Result;
use console::style;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::Path;

/// Sections of the spec that can be edited
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SpecSection {
    ProjectName,
    Overview,
    TechStack,
    Features,
    Database,
    ApiEndpoints,
    SuccessCriteria,
}

impl SpecSection {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectName => "Project Name",
            Self::Overview => "Overview",
            Self::TechStack => "Technology Stack",
            Self::Features => "Core Features",
            Self::Database => "Database Schema",
            Self::ApiEndpoints => "API Endpoints",
            Self::SuccessCriteria => "Success Criteria",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::ProjectName,
            Self::Overview,
            Self::TechStack,
            Self::Features,
            Self::Database,
            Self::ApiEndpoints,
            Self::SuccessCriteria,
        ]
    }
}

/// Application state
struct App {
    sections: Vec<SpecSection>,
    list_state: ListState,
    content: std::collections::HashMap<SpecSection, String>,
    editing: bool,
    edit_buffer: String,
    cursor_pos: usize,
    should_quit: bool,
    status_message: String,
}

impl App {
    fn new() -> Self {
        let sections = SpecSection::all();
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut content = std::collections::HashMap::new();
        content.insert(SpecSection::ProjectName, String::new());
        content.insert(SpecSection::Overview, String::new());
        content.insert(SpecSection::TechStack, "React, Node.js".to_string());
        content.insert(SpecSection::Features, String::new());
        content.insert(SpecSection::Database, String::new());
        content.insert(SpecSection::ApiEndpoints, String::new());
        content.insert(SpecSection::SuccessCriteria, String::new());

        Self {
            sections,
            list_state,
            content,
            editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,
            should_quit: false,
            status_message: "j/k: navigate | Enter: edit | q: quit | s: save".to_string(),
        }
    }

    fn selected_section(&self) -> Option<&SpecSection> {
        self.list_state.selected().map(|i| &self.sections[i])
    }

    fn next(&mut self) {
        if self.editing {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.sections.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.editing {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sections.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn start_editing(&mut self) {
        if let Some(section) = self.selected_section().cloned() {
            self.edit_buffer = self.content.get(&section).cloned().unwrap_or_default();
            self.cursor_pos = self.edit_buffer.len();
            self.editing = true;
            self.status_message = "Esc: cancel | Ctrl+S: save section".to_string();
        }
    }

    fn save_edit(&mut self) {
        if let Some(section) = self.selected_section().cloned() {
            self.content.insert(section, self.edit_buffer.clone());
            self.editing = false;
            self.status_message = "Saved! | j/k: navigate | Enter: edit | q: quit | s: save".to_string();
        }
    }

    fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.status_message = "j/k: navigate | Enter: edit | q: quit | s: save".to_string();
    }

    fn handle_edit_input(&mut self, c: char) {
        self.edit_buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    fn handle_edit_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.edit_buffer.remove(self.cursor_pos);
        }
    }

    fn handle_edit_newline(&mut self) {
        self.edit_buffer.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
    }

    fn generate_spec(&self) -> String {
        let project_name = self.content.get(&SpecSection::ProjectName).map(|s| s.as_str()).unwrap_or("Untitled");
        let overview = self.content.get(&SpecSection::Overview).map(|s| s.as_str()).unwrap_or("");
        let tech_stack = self.content.get(&SpecSection::TechStack).map(|s| s.as_str()).unwrap_or("");
        let features = self.content.get(&SpecSection::Features).map(|s| s.as_str()).unwrap_or("");
        let database = self.content.get(&SpecSection::Database).map(|s| s.as_str()).unwrap_or("");
        let api = self.content.get(&SpecSection::ApiEndpoints).map(|s| s.as_str()).unwrap_or("");
        let criteria = self.content.get(&SpecSection::SuccessCriteria).map(|s| s.as_str()).unwrap_or("");

        format!(r#"<project_specification>
<project_name>{}</project_name>

<overview>
{}
</overview>

<technology_stack>
{}
</technology_stack>

<core_features>
{}
</core_features>

<database_schema>
{}
</database_schema>

<api_endpoints>
{}
</api_endpoints>

<success_criteria>
{}
</success_criteria>
</project_specification>"#,
            project_name, overview, tech_stack, features, database, api, criteria
        )
    }
}

/// Run the interactive spec editor
pub fn run_editor(output_dir: &Path) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if app.editing {
                match (key.code, key.modifiers) {
                    (KeyCode::Esc, _) => app.cancel_edit(),
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => app.save_edit(),
                    (KeyCode::Enter, _) => app.handle_edit_newline(),
                    (KeyCode::Backspace, _) => app.handle_edit_backspace(),
                    (KeyCode::Char(c), _) => app.handle_edit_input(c),
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Enter => app.start_editing(),
                    KeyCode::Char('s') => {
                        // Save and scaffold
                        let spec = app.generate_spec();
                        
                        // Restore terminal first
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;

                        // Validate and scaffold
                        println!("\n{}", style("─── Validating Specification ───").cyan().bold());
                        let validation = crate::validation::validate_spec(&spec)?;
                        validation.print();

                        if validation.is_valid {
                            crate::scaffold::scaffold_with_spec_text(output_dir, &spec)?;
                            println!("\n{}", style("✅ Project scaffolded successfully!").green().bold());
                        } else {
                            println!("\n{}", style("⚠️  Spec has validation errors. Fix them before scaffolding.").yellow());
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    // Create layout: sidebar | editor | preview
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ])
        .split(main_chunks[0]);

    // Sidebar - section list
    let items: Vec<ListItem> = app
        .sections
        .iter()
        .map(|s| {
            let style = if app.editing && app.selected_section() == Some(s) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(s.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Sections ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, content_chunks[0], &mut app.list_state);

    // Editor pane
    let editor_title = if app.editing { " Editor [EDITING] " } else { " Editor " };
    let editor_content = if app.editing {
        format!("{}▏", &app.edit_buffer) // Show cursor
    } else {
        app.selected_section()
            .and_then(|s| app.content.get(s))
            .cloned()
            .unwrap_or_else(|| "(empty)".to_string())
    };

    let editor = Paragraph::new(editor_content)
        .block(
            Block::default()
                .title(editor_title)
                .borders(Borders::ALL)
                .border_style(if app.editing {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(editor, content_chunks[1]);

    // Preview pane
    let preview_spec = app.generate_spec();
    let preview_lines: Vec<Line> = preview_spec
        .lines()
        .take(30)
        .map(|line| {
            if line.starts_with('<') && line.ends_with('>') {
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
            } else {
                Line::from(line)
            }
        })
        .collect();

    let preview = Paragraph::new(Text::from(preview_lines))
        .block(Block::default().title(" Preview ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    f.render_widget(preview, content_chunks[2]);

    // Status bar
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default());

    f.render_widget(status, main_chunks[1]);
}
