use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::{event, execute, terminal, ExecutableCommand, QueueableCommand};
use crossterm::cursor::{MoveTo, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, LeaveAlternateScreen, Clear, ClearType};

/// Displays an interactive menu for the algorithm visualizer and returns the selected option.
///
/// # Returns
/// A number representing the selected menu option (1-10).
/// 1-9 correspond to different algorithms, 10 corresponds to Exit.
pub fn print_menu_banner() -> u32 {
    // Enable raw mode for direct keyboard input handling
    enable_raw_mode().unwrap();

    // Get a handle to standard output
    let mut stdout = stdout();

    // Enter alternate screen and clear it
    execute!(stdout, terminal::EnterAlternateScreen, Clear(ClearType::All)).unwrap();

    // Track the currently selected menu option (0-based index)
    let mut selection = 0u32;

    // Define all menu options
    let options = [
        "1. Generate Random Array",
        "2. Bubble Sort",
        "3. Selection Sort",
        "4. Insertion Sort",
        "5. Quick Sort",
        "6. Merge Sort",
        "7. Heap Sort",
        "8. Shell Sort",
        "9. Radix Sort",
        "10. Exit"
    ];

    // Main menu loop
    loop {
        // Get current terminal dimensions
        let (width, height) = size().unwrap();

        // Clear the screen with default background
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Draw Title ---
        let title = "TOGISOFT ALGORITHM VISUALIZER";
        let title_x = if width > title.len() as u16 {
            // Center the title horizontally
            (width - title.len() as u16) / 2
        } else {
            // If terminal is too narrow, start at position 0
            0
        };
        let title_y = height / 2 - 8; // Position title 8 rows above center

        stdout.queue(MoveTo(title_x, title_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
        stdout.queue(Print(title)).unwrap();

        // --- Draw Subtitle ---
        let subtitle = "Menu";
        let subtitle_x = if width > subtitle.len() as u16 {
            // Center the subtitle horizontally
            (width - subtitle.len() as u16) / 2
        } else {
            // If terminal is too narrow, start at position 0
            0
        };
        let subtitle_y = title_y + 2; // Position subtitle 2 rows below title

        stdout.queue(MoveTo(subtitle_x, subtitle_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
        stdout.queue(Print(subtitle)).unwrap();

        // --- Draw Menu Selection Prompt ---
        let menu_select_text = "Select an option (Use ↑/↓ arrows, Enter to select):";
        let menu_select_x = if width > menu_select_text.len() as u16 {
            // Center the prompt horizontally
            (width - menu_select_text.len() as u16) / 2
        } else {
            // If terminal is too narrow, start at position 0
            0
        };
        let menu_select_y = subtitle_y + 2; // Position prompt 2 rows below subtitle

        stdout.queue(MoveTo(menu_select_x, menu_select_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::White)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
        stdout.queue(Print(menu_select_text)).unwrap();

        // --- Draw Menu Options ---
        for (i, option) in options.iter().enumerate() {
            // Calculate position for this option
            let option_x = if width > option.len() as u16 {
                // Center the option horizontally
                (width - option.len() as u16) / 2
            } else {
                // If terminal is too narrow, start at position 0
                0
            };
            let option_y = menu_select_y + 2 + i as u16; // Position each option 2 rows below the prompt, then incrementally

            stdout.queue(MoveTo(option_x, option_y)).unwrap();

            // Highlight the currently selected option
            if i == selection as usize {
                stdout.queue(SetForegroundColor(Color::Black)).unwrap();
                stdout.queue(SetBackgroundColor(Color::White)).unwrap();
            } else {
                // Normal styling for unselected options
                stdout.queue(SetForegroundColor(Color::White)).unwrap();
                stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
            }

            // Print the option with some padding
            stdout.queue(Print(format!(" {} ", option))).unwrap();
        }

        // Reset all styling to default
        stdout.queue(ResetColor).unwrap();
        stdout.flush().unwrap();

        // --- Handle Keyboard Input ---
        if event::poll(Duration::from_millis(100)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Up => {
                            // Move selection up
                            if selection > 0 {
                                selection -= 1;
                            } else {
                                // Wrap around to the bottom
                                selection = options.len() as u32 - 1;
                            }
                        },
                        KeyCode::Down => {
                            // Move selection down (with wrap-around)
                            selection = (selection + 1) % options.len() as u32;
                        },
                        KeyCode::Enter => {
                            // Return the selected option (1-based index)
                            stdout.queue(ResetColor).unwrap();
                            execute!(stdout, Show, LeaveAlternateScreen).unwrap();
                            disable_raw_mode().unwrap();
                            return selection + 1;
                        },
                        KeyCode::Esc | KeyCode::Char('q') => {
                            // Exit the application (return Exit option)
                            stdout.queue(ResetColor).unwrap();
                            execute!(stdout, Show, LeaveAlternateScreen).unwrap();
                            disable_raw_mode().unwrap();
                            return 10; // Exit option
                        },
                        _ => {
                            // Ignore other keys
                        }
                    }
                },
                _ => {
                    // Ignore non-key events
                }
            }
        }
    }
}
