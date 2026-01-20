//! Terminal User Interface
//!
//! Interactive TUI for browsing and cleaning artifacts.
//!
//! This module provides a full-screen terminal interface using Ratatui.

pub mod app;
pub mod event;
pub mod ui;

pub use app::{App, AppState, ProjectEntry};
pub use event::{Action, Event, EventHandler};

use crate::error::Result;
use crate::trash::{delete_path, DeleteMethod};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// Run the TUI application
pub fn run(paths: Vec<PathBuf>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(paths);

    // Create event handler with faster tick rate for smooth animations
    let events = EventHandler::new(Duration::from_millis(50));

    // Main loop
    let result = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::render(app, frame))?;

        // Handle events
        let event = events.next().map_err(|e| crate::error::DevSweepError::Other(e.to_string()))?;
        match event {
            Event::Key(key) => {
                // Handle search mode separately
                if app.is_searching {
                    match key.code {
                        KeyCode::Esc => app.end_search(),
                        KeyCode::Enter => app.end_search(),
                        KeyCode::Backspace => app.search_pop(),
                        KeyCode::Char(c) => app.search_push(c),
                        _ => {}
                    }
                    continue;
                }

                // Handle help popup
                if app.show_help {
                    app.show_help = false;
                    continue;
                }

                // Convert key to action
                let action = Action::from_key(key);

                // Handle state-specific actions
                match app.state {
                    AppState::Scanning => {
                        // Only allow quit during scanning
                        if matches!(action, Action::Quit) {
                            app.should_quit = true;
                        }
                    }
                    AppState::Confirming => match action {
                        Action::Confirm => {
                            let paths = app.confirm_delete();
                            let (success, failed, freed) = delete_paths(&paths);
                            app.deletion_complete(success, failed, freed);
                        }
                        Action::Cancel | Action::Quit => {
                            app.cancel_delete();
                        }
                        _ => {}
                    },
                    AppState::Ready => match action {
                        // Menu navigation
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::Up | Action::ScrollUp => app.menu_up(),
                        Action::Down | Action::ScrollDown => app.menu_down(),
                        Action::ToggleExpand | Action::Scan | Action::Confirm => {
                            // Enter key or 's' key starts scan
                            app.start_scan();
                        }
                        Action::Help => app.toggle_help(),
                        _ => {}
                    },
                    AppState::Results | AppState::CacheResults | AppState::CleanerResults | AppState::Error(_) => match action {
                        Action::Quit => {
                            app.should_quit = true;
                        }
                        Action::Up => app.select_up(),
                        Action::Down => app.select_down(),
                        Action::PageUp => app.page_up(10),
                        Action::PageDown => app.page_down(10),
                        Action::Top => app.go_top(),
                        Action::Bottom => app.go_bottom(),
                        Action::ToggleSelect => app.toggle_select(),
                        Action::ToggleExpand => app.toggle_expand(),
                        Action::Expand => app.expand(),
                        Action::Collapse => app.collapse(),
                        Action::SelectAll => app.select_all(),
                        Action::DeselectAll => app.deselect_all(),
                        Action::Delete => app.request_delete(),
                        Action::Help => app.toggle_help(),
                        Action::Scan | Action::Refresh => {
                            app.start_scan();
                        }
                        Action::Search => app.start_search(),
                        Action::NextTab => app.next_tab(),
                        Action::PrevTab => app.prev_tab(),
                        Action::ScrollUp => app.scroll_up(),
                        Action::ScrollDown => app.scroll_down(),
                        Action::Back => {
                            app.go_back();
                        }
                        Action::Cancel => {
                            // Esc - go back to menu (or clear search if active)
                            if !app.search_query.is_empty() {
                                app.search_query.clear();
                                app.filter_by_tab();
                            } else {
                                app.go_back();
                            }
                        }
                        _ => {}
                    },
                    AppState::Cleaning => {
                        // No actions during cleaning
                    }
                }
            }
            Event::Tick => {
                // Check for scan updates on every tick
                if app.state == AppState::Scanning {
                    app.check_scan_progress();
                }
            }
            Event::Resize(_, _) => {
                // Terminal will redraw on next iteration
            }
            Event::Mouse(mouse) => {
                // Handle mouse events (scroll wheel)
                let action = Action::from_mouse(&mouse);
                match action {
                    Action::ScrollUp => {
                        app.select_up();
                        app.select_up();
                        app.select_up();
                    }
                    Action::ScrollDown => {
                        app.select_down();
                        app.select_down();
                        app.select_down();
                    }
                    _ => {}
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Delete paths and return (success_count, fail_count, bytes_freed)
fn delete_paths(paths: &[PathBuf]) -> (usize, usize, u64) {
    let mut success = 0;
    let mut failed = 0;
    let mut freed = 0u64;

    for path in paths {
        // Get size before deletion
        let size = if path.is_dir() {
            dir_size(path)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };

        // Delete using trash
        match delete_path(path, DeleteMethod::Trash) {
            Ok(_) => {
                success += 1;
                freed += size;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    (success, failed, freed)
}

/// Calculate directory size
fn dir_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
