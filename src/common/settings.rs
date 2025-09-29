use crossterm::{
    cursor::{MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{stdout, Read, Write};
use std::path::Path;
use std::time::Duration;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    pub speed: u64, // milliseconds
    pub teaching_mode: bool,
    pub last_visualizer: Option<String>, // e.g., "BubbleSort"
}

impl Settings {
    pub fn load() -> Self {
        if Path::new(SETTINGS_FILE).exists() {
            let mut file = File::open(SETTINGS_FILE).expect("Failed to open settings file");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Failed to read settings file");
            serde_json::from_str(&contents).unwrap_or_else(|_| Settings::default())
        } else {
            Settings::default()
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize settings");
        fs::write(SETTINGS_FILE, json).expect("Failed to write settings file");
    }

    /// Displays an interactive settings menu using crossterm and returns the updated Settings.
    pub fn show_settings_menu(mut settings: Settings) -> Settings {
        // Enable raw mode for direct keyboard input handling
        enable_raw_mode().unwrap();
        // Get a handle to standard output
        let mut stdout = stdout();
        // Enter alternate screen and clear it
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All)).unwrap();
        // Track the currently selected menu option (0-based index)
        let mut selection = 0u32;
        // Define settings options
        let options = ["1. Change Speed", "2. Toggle Teaching Mode", "3. Back"];
        // Main settings loop
        loop {
            // Get current terminal dimensions
            let (width, height) = size().unwrap();
            // Clear the screen with default background
            execute!(stdout, Clear(ClearType::All)).unwrap();
            // --- Draw Title ---
            let title = "SETTINGS";
            let title_x = if width > title.len() as u16 {
                (width - title.len() as u16) / 2
            } else {
                0
            };
            let title_y = height / 2 - 6;
            execute!(stdout, MoveTo(title_x, title_y)).unwrap();
            execute!(stdout, SetForegroundColor(Color::Yellow)).unwrap();
            execute!(stdout, SetBackgroundColor(Color::DarkBlue)).unwrap();
            execute!(stdout, Print(title)).unwrap();
            // --- Draw Current Settings ---
            let settings_info_y = title_y + 2;
            let speed_text = format!("Current Speed: {} ms", settings.speed);
            let teaching_text = format!(
                "Teaching Mode: {}",
                if settings.teaching_mode { "ON" } else { "OFF" }
            );
            let last_viz_text = format!(
                "Last Visualizer: {:?}",
                settings.last_visualizer.as_ref().unwrap_or(&"None".to_string())
            );
            execute!(stdout, MoveTo(5, settings_info_y)).unwrap();
            execute!(stdout, SetForegroundColor(Color::Cyan)).unwrap();
            execute!(stdout, Print(&speed_text)).unwrap();
            execute!(stdout, MoveTo(5, settings_info_y + 1)).unwrap();
            execute!(stdout, SetForegroundColor(Color::Cyan)).unwrap();
            execute!(stdout, Print(&teaching_text)).unwrap();
            execute!(stdout, MoveTo(5, settings_info_y + 2)).unwrap();
            execute!(stdout, SetForegroundColor(Color::Cyan)).unwrap();
            execute!(stdout, Print(&last_viz_text)).unwrap();
            // --- Draw Subtitle ---
            let subtitle = "Options";
            let subtitle_x = if width > subtitle.len() as u16 {
                (width - subtitle.len() as u16) / 2
            } else {
                0
            };
            let subtitle_y = settings_info_y + 4;
            execute!(stdout, MoveTo(subtitle_x, subtitle_y)).unwrap();
            execute!(stdout, SetForegroundColor(Color::Cyan)).unwrap();
            execute!(stdout, SetBackgroundColor(Color::Reset)).unwrap();
            execute!(stdout, Print(subtitle)).unwrap();
            // --- Draw Menu Selection Prompt ---
            let menu_select_text = "Select an option (Use ↑/↓ arrows, Enter to select):";
            let menu_select_x = if width > menu_select_text.len() as u16 {
                (width - menu_select_text.len() as u16) / 2
            } else {
                0
            };
            let menu_select_y = subtitle_y + 2;
            execute!(stdout, MoveTo(menu_select_x, menu_select_y)).unwrap();
            execute!(stdout, SetForegroundColor(Color::White)).unwrap();
            execute!(stdout, SetBackgroundColor(Color::Reset)).unwrap();
            execute!(stdout, Print(menu_select_text)).unwrap();
            // --- Draw Menu Options ---
            for (i, option) in options.iter().enumerate() {
                // Calculate position for this option
                let option_x = if width > option.len() as u16 {
                    (width - option.len() as u16) / 2
                } else {
                    0
                };
                let option_y = menu_select_y + 2 + i as u16;
                execute!(stdout, MoveTo(option_x, option_y)).unwrap();
                // Highlight the currently selected option
                if i == selection as usize {
                    execute!(stdout, SetForegroundColor(Color::Black)).unwrap();
                    execute!(stdout, SetBackgroundColor(Color::White)).unwrap();
                } else {
                    execute!(stdout, SetForegroundColor(Color::White)).unwrap();
                    execute!(stdout, SetBackgroundColor(Color::Reset)).unwrap();
                }
                // Print the option with some padding
                execute!(stdout, Print(format!(" {} ", option))).unwrap();
            }
            // Reset all styling to default
            execute!(stdout, ResetColor).unwrap();
            stdout.flush().unwrap();
            // --- Handle Keyboard Input ---
            if poll(Duration::from_millis(100)).unwrap() {
                match read().unwrap() {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Up => {
                                // Move selection up
                                if selection > 0 {
                                    selection -= 1;
                                } else {
                                    // Wrap around to the bottom
                                    selection = options.len() as u32 - 1;
                                }
                            }
                            KeyCode::Down => {
                                // Move selection down (with wrap-around)
                                selection = (selection + 1) % options.len() as u32;
                            }
                            KeyCode::Enter => {
                                // Process selection
                                match selection {
                                    0 => {
                                        // Change Speed - Sub-menu for input
                                        if let Some(speed) = change_speed_menu() {
                                            settings.speed = speed;
                                            settings.save(); // Save immediately
                                        }
                                    }
                                    1 => {
                                        // Toggle Teaching Mode
                                        settings.teaching_mode = !settings.teaching_mode;
                                        settings.save(); // Save immediately
                                    }
                                    2 => {
                                        // Back
                                        execute!(stdout, ResetColor).unwrap();
                                        execute!(stdout, Show, LeaveAlternateScreen).unwrap();
                                        disable_raw_mode().unwrap();
                                        return settings;
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Esc => {
                                // Back/Escape
                                execute!(stdout, ResetColor).unwrap();
                                execute!(stdout, Show, LeaveAlternateScreen).unwrap();
                                disable_raw_mode().unwrap();
                                return settings;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Interactive sub-menu to change speed using crossterm
fn change_speed_menu() -> Option<u64> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    let mut input = String::new();
    let mut cursor_position = 0;
    let fixed_prompt = "Enter speed (100-3000): ";

    loop {
        // Clear the screen
        execute!(stdout, Clear(ClearType::All)).unwrap();

        // Get terminal dimensions
        let (width, height) = match size() {
            Ok(size) => size,
            Err(_) => (80, 24), // Default size if error
        };

        // Draw title
        let title = "CHANGE SPEED (ms, 100-3000)";
        let title_x = (width / 2).saturating_sub(title.len() as u16 / 2);
        execute!(
            stdout,
            MoveTo(title_x, height / 2 - 2),
            SetForegroundColor(Color::Yellow),
            Print(title),
            ResetColor
        )
            .unwrap();

        // Draw input prompt
        let full_prompt = format!("{}{}", fixed_prompt, input);
        let full_prompt_len = full_prompt.len();
        let prompt_x = (width / 2).saturating_sub(full_prompt_len as u16 / 2);
        execute!(
            stdout,
            MoveTo(prompt_x, height / 2),
            SetForegroundColor(Color::White),
            Print(&full_prompt),
            ResetColor
        )
            .unwrap();

        // Position cursor after fixed prompt + cursor position in input
        let cursor_x = prompt_x + fixed_prompt.len() as u16 + cursor_position as u16;
        execute!(stdout, MoveTo(cursor_x, height / 2)).unwrap();

        // Flush output
        stdout.flush().unwrap();

        // Handle input
        if poll(Duration::from_millis(100)).unwrap() {
            match read().unwrap() {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            // Add character to input
                            input.insert(cursor_position, c);
                            cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            // Remove last character
                            if !input.is_empty() && cursor_position > 0 {
                                input.remove(cursor_position - 1);
                                cursor_position -= 1;
                            }
                        }
                        KeyCode::Left => {
                            // Move cursor left
                            if cursor_position > 0 {
                                cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            // Move cursor right
                            if cursor_position < input.len() {
                                cursor_position += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Try to parse input as u64
                            if !input.is_empty() {
                                if let Ok(speed) = input.parse::<u64>() {
                                    if speed >= 100 && speed <= 3000 {
                                        // Valid speed, return it
                                        execute!(stdout, ResetColor).unwrap();
                                        return Some(speed);
                                    }
                                }
                                // Invalid input, clear and continue
                                input.clear();
                                cursor_position = 0;
                            }
                        }
                        KeyCode::Esc => {
                            // Cancel and return None
                            execute!(stdout, ResetColor).unwrap();
                            return None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}