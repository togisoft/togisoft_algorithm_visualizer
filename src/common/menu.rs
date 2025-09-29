use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::{event, execute, terminal, ExecutableCommand, QueueableCommand};
use crossterm::cursor::{MoveTo, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, LeaveAlternateScreen, Clear, ClearType};

#[derive(Debug, Clone)]
pub struct MenuOption {
    pub id: u32,
    pub name: String,
    pub category: String,
}

/// Displays an interactive categorized menu for the algorithm visualizer and returns the selected option.
///
/// # Returns
/// A number representing the selected menu option.
pub fn print_menu_banner() -> u32 {
    // Enable raw mode for direct keyboard input handling
    enable_raw_mode().unwrap();

    // Get a handle to standard output
    let mut stdout = stdout();

    // Enter alternate screen and clear it
    execute!(stdout, terminal::EnterAlternateScreen, Clear(ClearType::All)).unwrap();

    // Define menu categories and options
    let categories = vec![
        ("START", vec![
            MenuOption { id: 1, name: "Generate Array List".to_string(), category: "start".to_string() },
        ]),
        ("SEARCH ALGORITHMS", vec![
            MenuOption { id: 2, name: "Linear Search".to_string(), category: "search".to_string() },
            MenuOption { id: 3, name: "Binary Search".to_string(), category: "search".to_string() },
        ]),
        ("SORTING ALGORITHMS", vec![
            MenuOption { id: 4, name: "Bubble Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 5, name: "Bucket Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 6, name: "Cocktail Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 7, name: "Comb Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 8, name: "Counting Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 9, name: "Gnome Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 10, name: "Heap Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 11, name: "Insertion Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 12, name: "Merge Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 13, name: "Pancake Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 14, name: "Quick Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 15, name: "Radix Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 16, name: "Selection Sort".to_string(), category: "sort".to_string() },
            MenuOption { id: 17, name: "Shell Sort".to_string(), category: "search".to_string() },
            MenuOption { id: 18, name: "Tim Sort".to_string(), category: "search".to_string() },
        ]),
        ("⚙️ SETTINGS & OTHERS", vec![
            MenuOption { id: 31, name: "Settings".to_string(), category: "settings".to_string() },
        ]),
    ];

    let mut selected_category = 0usize;
    let mut selected_option = 0usize;

    // Main menu loop
    loop {
        // Get current terminal dimensions
        let (width, height) = size().unwrap();

        // Clear the screen
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Draw Title ---
        draw_title(&mut stdout, width, 2);

        // --- Draw Navigation Instructions ---
        draw_navigation_help(&mut stdout, width, 6);

        // Calculate layout dimensions
        let content_start_y = 10;
        let category_width = 30u16;
        let option_width = 40u16;
        let padding = 4u16;

        let total_content_width = category_width + option_width + padding;
        let start_x = (width.saturating_sub(total_content_width)) / 2;

        let category_x = start_x;
        let option_x = category_x + category_width + padding;

        // --- Draw Categories (Left Side) ---
        draw_category_header(&mut stdout, category_x, content_start_y, "CATEGORIES");

        for (i, (category_name, _)) in categories.iter().enumerate() {
            let y = content_start_y + 3 + (i as u16);
            let is_selected = i == selected_category;
            draw_category_item(&mut stdout, category_name, category_x, y, is_selected, category_width as usize);
        }

        // --- Draw Options (Right Side) ---
        let current_category = &categories[selected_category];
        draw_category_header(&mut stdout, option_x, content_start_y, &current_category.0);

        for (i, option) in current_category.1.iter().enumerate() {
            let y = content_start_y + 3 + (i as u16);
            let is_selected = i == selected_option;
            let option_text = format!("{}. {}", option.id, option.name);
            draw_option_item(&mut stdout, &option_text, option_x, y, is_selected, option_width as usize);
        }

        // --- Draw Selected Option Description ---
        if selected_category < categories.len() && selected_option < categories[selected_category].1.len() {
            let selected_id = categories[selected_category].1[selected_option].id;
            let description = get_option_description(selected_id);
            draw_description(&mut stdout, width, height - 4, &description);
        }

        // --- Draw Border Box ---
        // Calculate dynamic height based on the maximum number of options in any category
        let max_options = categories.iter().map(|(_, opts)| opts.len()).max().unwrap_or(0) as u16;
        let border_height = max_options + 5; // 2 for header, 1 for separator, 2 for padding
        draw_border_box(&mut stdout, start_x - 2, content_start_y - 2, total_content_width + 4, border_height);

        // Reset styling and flush
        stdout.queue(ResetColor).unwrap();
        stdout.flush().unwrap();

        // --- Handle Keyboard Input ---
        if event::poll(Duration::from_millis(100)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Right | KeyCode::Tab => {
                            // Move to options panel
                        },
                        KeyCode::Left => {
                            // Move to categories panel
                        },
                        KeyCode::Down => {
                            if selected_option < categories[selected_category].1.len() - 1 {
                                selected_option += 1;
                            } else {
                                selected_option = 0;
                            }
                        },
                        KeyCode::Up => {
                            if selected_option > 0 {
                                selected_option -= 1;
                            } else {
                                selected_option = categories[selected_category].1.len() - 1;
                            }
                        },
                        KeyCode::Char('s') => {
                            selected_category = (selected_category + 1) % categories.len();
                            selected_option = 0;
                        },
                        KeyCode::Char('w') => {
                            selected_category = if selected_category == 0 {
                                categories.len() - 1
                            } else {
                                selected_category - 1
                            };
                            selected_option = 0;
                        },
                        KeyCode::Enter => {
                            let selected_id = categories[selected_category].1[selected_option].id;
                            cleanup_terminal(&mut stdout);
                            return selected_id;
                        },
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            cleanup_terminal(&mut stdout);
                            return 99; // Exit option
                        },
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            // Allow direct number selection
                            if let Some(digit) = c.to_digit(10) {
                                for (cat_idx, (_, options)) in categories.iter().enumerate() {
                                    for (opt_idx, option) in options.iter().enumerate() {
                                        if option.id == digit {
                                            selected_category = cat_idx;
                                            selected_option = opt_idx;
                                            break;
                                        }
                                    }
                                }
                            }
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

fn draw_title(stdout: &mut std::io::Stdout, width: u16, y: u16) {
    let title = "TOGISOFT ALGORITHM VISUALIZER";
    let title_len = title.len() as u16;
    let x = (width.saturating_sub(title_len + 4)) / 2;

    // Top border
    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
    stdout.queue(Print("╔")).unwrap();
    stdout.queue(Print("═".repeat((title_len + 2) as usize))).unwrap();
    stdout.queue(Print("╗")).unwrap();

    // Title
    stdout.queue(MoveTo(x, y + 1)).unwrap();
    stdout.queue(Print("║ ")).unwrap();
    stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
    stdout.queue(Print(title)).unwrap();
    stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
    stdout.queue(Print(" ║")).unwrap();

    // Bottom border
    stdout.queue(MoveTo(x, y + 2)).unwrap();
    stdout.queue(Print("╚")).unwrap();
    stdout.queue(Print("═".repeat((title_len + 2) as usize))).unwrap();
    stdout.queue(Print("╝")).unwrap();

    stdout.queue(ResetColor).unwrap();
}

fn draw_navigation_help(stdout: &mut std::io::Stdout, width: u16, y: u16) {
    let help_text = "↑↓ Navigate | W/S Categories | Enter Select | Esc/Q Exit";
    let x = (width.saturating_sub(help_text.len() as u16)) / 2;

    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
    stdout.queue(Print(help_text)).unwrap();
    stdout.queue(ResetColor).unwrap();
}

fn draw_category_header(stdout: &mut std::io::Stdout, x: u16, y: u16, title: &str) {
    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetForegroundColor(Color::Magenta)).unwrap();
    stdout.queue(Print(title)).unwrap();

    stdout.queue(MoveTo(x, y + 1)).unwrap();
    stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print("─".repeat(title.chars().count().min(30)))).unwrap();

    stdout.queue(ResetColor).unwrap();
}

fn draw_category_item(
    stdout: &mut std::io::Stdout,
    category: &str,
    x: u16,
    y: u16,
    is_selected: bool,
    max_width: usize,
) {
    stdout.queue(MoveTo(x, y)).unwrap();

    if is_selected {
        stdout.queue(SetForegroundColor(Color::Black)).unwrap();
        stdout.queue(SetBackgroundColor(Color::White)).unwrap();
        stdout.queue(Print("▶ ")).unwrap();
    } else {
        stdout.queue(SetForegroundColor(Color::White)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
        stdout.queue(Print("  ")).unwrap();
    }

    // Truncate if too long
    let display_text = if category.chars().count() > max_width - 3 {
        format!("{}...", category.chars().take(max_width - 6).collect::<String>())
    } else {
        category.to_string()
    };

    stdout.queue(Print(&display_text)).unwrap();

    if is_selected {
        // Fill remaining space with background color
        let remaining = max_width.saturating_sub(display_text.chars().count() + 2);
        stdout.queue(Print(" ".repeat(remaining))).unwrap();
    }

    stdout.queue(ResetColor).unwrap();
}

fn draw_option_item(
    stdout: &mut std::io::Stdout,
    option: &str,
    x: u16,
    y: u16,
    is_selected: bool,
    max_width: usize,
) {
    stdout.queue(MoveTo(x, y)).unwrap();

    if is_selected {
        stdout.queue(SetForegroundColor(Color::Black)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Green)).unwrap();
        stdout.queue(Print("● ")).unwrap();
    } else {
        stdout.queue(SetForegroundColor(Color::White)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
        stdout.queue(Print("  ")).unwrap();
    }

    // Truncate if too long
    let display_text = if option.chars().count() > max_width - 3 {
        format!("{}...", option.chars().take(max_width - 6).collect::<String>())
    } else {
        option.to_string()
    };

    stdout.queue(Print(&display_text)).unwrap();

    if is_selected {
        // Fill remaining space with background color
        let remaining = max_width.saturating_sub(display_text.chars().count() + 2);
        stdout.queue(Print(" ".repeat(remaining))).unwrap();
    }

    stdout.queue(ResetColor).unwrap();
}

fn draw_description(stdout: &mut std::io::Stdout, width: u16, y: u16, description: &str) {
    let max_desc_width = (width as usize).saturating_sub(10);
    let wrapped_desc = if description.len() > max_desc_width {
        format!("{}...", &description[..max_desc_width.saturating_sub(3)])
    } else {
        description.to_string()
    };

    let x = (width.saturating_sub(wrapped_desc.len() as u16)) / 2;

    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
    stdout.queue(SetBackgroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print(format!(" {} ", wrapped_desc))).unwrap();
    stdout.queue(ResetColor).unwrap();
}

fn draw_border_box(stdout: &mut std::io::Stdout, x: u16, y: u16, width: u16, height: u16) {
    // Top border
    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print("┌")).unwrap();
    stdout.queue(Print("─".repeat((width - 2) as usize))).unwrap();
    stdout.queue(Print("┐")).unwrap();

    // Side borders
    for i in 1..height - 1 {
        stdout.queue(MoveTo(x, y + i)).unwrap();
        stdout.queue(Print("│")).unwrap();
        stdout.queue(MoveTo(x + width - 1, y + i)).unwrap();
        stdout.queue(Print("│")).unwrap();
    }

    // Bottom border
    stdout.queue(MoveTo(x, y + height - 1)).unwrap();
    stdout.queue(Print("└")).unwrap();
    stdout.queue(Print("─".repeat((width - 2) as usize))).unwrap();
    stdout.queue(Print("┘")).unwrap();

    stdout.queue(ResetColor).unwrap();
}

/// Helper function to clean up terminal state before exiting
fn cleanup_terminal(stdout: &mut std::io::Stdout) {
    stdout.queue(ResetColor).unwrap();
    execute!(stdout, Show, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}

/// Returns the category name for a given option ID
pub fn get_category_for_option(option_id: u32) -> String {
    match option_id {
        1..=10 => "search".to_string(),
        11..=20 => "sort".to_string(),
        21..=30 => "games".to_string(),
        31..=98 => "settings".to_string(),
        99 => "exit".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Returns a human-readable description for a given option ID
pub fn get_option_description(option_id: u32) -> String {
    match option_id {
        1 => "Generate a random array of numbers for algorithm testing and visualization".to_string(),
        2 => "Visualize linear search - searches elements one by one from start to end".to_string(),
        3 => "Visualize binary search - efficient search in sorted arrays using divide and conquer".to_string(),
        4 => "Visualize bubble sort - repeatedly swaps adjacent elements if they're in wrong order".to_string(),
        5 => "Visualize bucket sort - distributes elements into buckets then sorts each bucket".to_string(),
        6 => "Visualize cocktail sort - bidirectional bubble sort that sorts in both directions".to_string(),
        7 => "Visualize comb sort - improved bubble sort with gap sequence shrinking".to_string(),
        8 => "Visualize counting sort - counts occurrences of each element for non-comparison sorting".to_string(),
        9 => "Visualize gnome sort - simple sorting algorithm similar to insertion sort".to_string(),
        10 => "Visualize heap sort - uses binary heap data structure for efficient sorting".to_string(),
        11 => "Visualize insertion sort - builds sorted array one element at a time".to_string(),
        12 => "Visualize merge sort - divide and conquer algorithm that merges sorted subarrays".to_string(),
        13 => "Visualize pancake sort - sorts by flipping prefix of array like pancakes".to_string(),
        14 => "Visualize quick sort - efficient divide and conquer sorting with pivot selection".to_string(),
        15 => "Visualize radix sort - sorts integers by processing individual digits".to_string(),
        16 => "Visualize selection sort - finds minimum element and places it at beginning".to_string(),
        17 => "Visualize shell sort - generalized insertion sort with diminishing gaps".to_string(),
        18 => "Visualize tim sort - hybrid stable sorting algorithm derived from merge sort".to_string(),
        31 => "Configure application settings - speed, colors, array size, and display options".to_string(),
        99 => "Exit the application and return to terminal".to_string(),
        _ => "Unknown option - please select a valid menu item".to_string(),
    }
}