// TUI Menu System...
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{Event, KeyCode, poll, read, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

#[derive(PartialEq)]
enum MenuItem {
    Create,
    Read,
    Update,
    Delete,
    Quit,
}

pub fn terminal_test() -> Result<(), io::Error> {
    // Enable raw mode for terminal input
    enable_raw_mode()?;

    // Initialize the terminal backend
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Set up the initial menu state
    let mut current_menu_item = MenuItem::Create;

    // Main loop
    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ])
                .split(size);

            // Render menu items
            let menu_items = vec![
                ("Create", MenuItem::Create),
                ("Read", MenuItem::Read),
                ("Update", MenuItem::Update),
                ("Delete", MenuItem::Delete),
                ("Quit", MenuItem::Quit),
            ];

            let selected_index = menu_items
                .iter()
                .position(|(_, item)| item == &current_menu_item).unwrap();

            for (i, (text, _)) in menu_items.iter().enumerate() {
                let item = Paragraph::new(text.to_string())
                    .block(Block::default().borders(Borders::ALL))
                    .style(if i == selected_index { tui::style::Style::default().fg(tui::style::Color::Yellow) } else { tui::style::Style::default() });
                f.render_widget(item, chunks[i]);
            }
        })?;

        // Handle user input
        if poll(Duration::from_millis(100))?{
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        return Ok(());
                    }
                    KeyCode::Down => {
                        current_menu_item = match current_menu_item {
                            MenuItem::Create => MenuItem::Read,
                            MenuItem::Read => MenuItem::Update,
                            MenuItem::Update => MenuItem::Delete,
                            MenuItem::Delete => MenuItem::Quit,
                            MenuItem::Quit => MenuItem::Quit,
                        };
                    }
                    KeyCode::Up => {
                        current_menu_item = match current_menu_item {
                            MenuItem::Create => MenuItem::Create,
                            MenuItem::Read => MenuItem::Create,
                            MenuItem::Update => MenuItem::Read,
                            MenuItem::Delete => MenuItem::Update,
                            MenuItem::Quit => MenuItem::Delete,
                        };
                    }
                    KeyCode::Enter => {
                        match current_menu_item {
                            MenuItem::Create => {
                                println!("Create");
                            }
                            MenuItem::Read => {
                                println!("Read");
                            }
                            MenuItem::Update => {
                                println!("Update");
                            }
                            MenuItem::Delete => {
                                println!("Delete");
                            }
                            MenuItem::Quit => {
                                disable_raw_mode()?;
                                return Ok(());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}