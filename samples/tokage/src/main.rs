mod app;
mod test;
mod ui;

use anyhow::{Context, Result};
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the test configuration file (YAML or TOML)
    #[arg(short, long)]
    config: std::path::PathBuf,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Load and parse the configuration (YAML or TOML)
    let config = test::load_config(&args.config)
        .with_context(|| format!("failed to load config from `{}`", args.config.display()))?;
    
    // Run all tests
    let test_results = test::run_tests(&config.tests)?;
    
    // Display results in TUI
    start_ui(test_results)?;
    
    Ok(())
}

fn start_ui(test_results: Vec<test::TestResult>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(test_results);
    
    // Start the main loop
    loop {
        terminal.draw(|frame| {
            ui::render_ui::<CrosstermBackend<io::Stdout>>(frame, &app);
        })?;
        
        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('?') => app.toggle_help(),
                KeyCode::Down | KeyCode::Char('j') => {
                    if !app.show_help {
                        app.next()
                    }
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    if !app.show_help {
                        app.previous()
                    }
                },
                KeyCode::Right | KeyCode::Char('l') => {
                    if !app.show_help {
                        app.next_tab()
                    }
                },
                KeyCode::Left | KeyCode::Char('h') => {
                    if !app.show_help {
                        app.previous_tab()
                    }
                },
                KeyCode::Esc => {
                    if app.show_help {
                        app.toggle_help();
                    }
                },
                _ => {}
            }
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;
    
    Ok(())
} 