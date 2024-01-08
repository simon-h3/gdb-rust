mod types;  // Import the types module
mod disk;

extern crate ncurses;

use ncurses::*;

const TITLE: &str = r#"
            ___  ____   __   ____  _  _    ____   __  ____  __   ____   __   ____  ____ 
           / __)(  _ \ / _\ (  _ \/ )( \  (    \ / _\(_  _)/ _\ (  _ \ / _\ / ___)(  __)
          ( (_ \ )   //    \ ) __/) __ (   ) D (/    \ )( /    \ ) _ (/    \\___ \ ) _) 
           \___/(__\_)\_/\_/(__)  \_)(_/  (____/\_/\_/(__)\_/\_/(____/\_/\_/(____/(____)
        "#;

fn db_test() {
    types::print_struct_info();

    println!("Format: {:?}", disk::format_disk(10));
    println!("Header: {:?}", disk::print_header());

    // println!("Block 1: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Nodes: {:?}", disk::test_nodes());

    // println!("Block 2: {:?}", disk::print_block(24));

    // disk::print_first_empty();

    println!("Relationships: {:?}", disk::test_relationships());
    // let n = disk::print_block(24);

    println!("blocks: {:?}", disk::print_all_blocks());

    println!("Header 2: {:?}", disk::print_header());
}

fn ncurses_menu(){
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    
    // Print title
    // mvprintw(0,0, TITLE);
    // mvprintw(5,0," - Exit (q) - ");
    refresh();

    // Define menu options
    let choices = ["! TEST !", "-> Read", "-> Create", "-> Update", "-> Delete", "-> Views"];
    let mut highlight = 0;

    loop {
        mvprintw(0,0, TITLE);
        mvprintw(6,4," # Exit (q) # ");
        // Print menu options
        for i in 0..choices.len() {
            if i == highlight {
                attron(A_REVERSE());
            }
            mvprintw((i + 8) as i32, 5, choices[i]);
            attroff(A_REVERSE());
        }

        // Get user input
        let ch = getch();

        match ch {
            KEY_UP => {
                if highlight > 0 {
                    highlight -= 1;
                }
            }
            KEY_DOWN => {
                if highlight < choices.len() - 1 {
                    highlight += 1;
                }
            }
            10 => {
                // User pressed Enter
                clear();
                // printw(&format!("You selected: {}", choices[highlight]));

                match highlight {
                    0 => {
                        db_test();
                    }
                    1 => {
                        // Read
                        mvprintw(0,0, "READ");
                        refresh();
                        getch();
                    }
                    2 => {
                        // Create
                        mvprintw(0,0, "CREATE");
                        refresh();
                        getch();
                    }
                    3 => {
                        // Update
                        mvprintw(0,0, "UPDATE");
                        refresh();
                        getch();
                    }
                    4 => {
                        // Delete
                        mvprintw(0,0, "DELETE");
                        refresh();
                        getch();
                    }
                    5 => {
                        // Views
                        mvprintw(0,0, "VIEWS");
                        refresh();
                        getch();
                    }
                    _ => {}
                }

                refresh();
                getch(); // Wait for a key press
                clear();
                refresh();
            }
            _ => {}
        }

        // Exit loop when 'q' is pressed
        if ch == 'q' as i32 {
            break;
        }
    }

    // Clean up and exit
    endwin();
}

fn main() {
    db_test();
}
