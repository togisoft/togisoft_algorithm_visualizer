use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, size},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Attribute, SetAttribute},
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
};
use std::io::{stdout, Write};
use std::time::Duration;
use rand::Rng;
use crate::helper::cleanup_terminal;

// Represents a single array with metadata for visualization and management
#[derive(Debug, Clone)]
pub struct ArrayData {
    pub data: Vec<u32>,      // The actual array data
    pub name: String,        // User-defined name for the array
    pub size: usize,         // Number of elements in the array
    pub min_value: u32,      // Minimum value in the array
    pub max_value: u32,      // Maximum value in the array
}

impl ArrayData {
    // Constructs a new ArrayData instance, calculating min/max values
    pub fn new(data: Vec<u32>, name: String) -> Self {
        let size = data.len();
        let min_value = *data.iter().min().unwrap_or(&0);
        let max_value = *data.iter().max().unwrap_or(&0);
        Self {
            data,
            name,
            size,
            min_value,
            max_value,
        }
    }
}

// Manages a collection of arrays and tracks the currently selected array
pub struct ArrayManager {
    arrays: Vec<ArrayData>,          // Collection of all arrays
    selected_index: Option<usize>,  // Index of the currently selected array (if any)
}

impl ArrayManager {
    // Initializes a new, empty ArrayManager
    pub fn new() -> Self {
        Self {
            arrays: Vec::new(),
            selected_index: None,
        }
    }

    // Adds a new array to the manager
    pub fn add_array(&mut self, array_data: ArrayData) {
        self.arrays.push(array_data);
    }

    // Returns an immutable reference to the currently selected array
    pub fn get_selected_array(&self) -> Option<&ArrayData> {
        if let Some(index) = self.selected_index {
            self.arrays.get(index)
        } else {
            None
        }
    }

    // Returns a mutable reference to the currently selected array
    pub fn get_selected_array_mut(&mut self) -> Option<&mut ArrayData> {
        if let Some(index) = self.selected_index {
            self.arrays.get_mut(index)
        } else {
            None
        }
    }

    // Removes an array at the specified index and updates selection if necessary
    pub fn remove_array(&mut self, index: usize) {
        if index < self.arrays.len() {
            self.arrays.remove(index);
            if let Some(selected) = self.selected_index {
                if selected == index {
                    self.selected_index = None;  // Deselect if the removed array was selected
                } else if selected > index {
                    self.selected_index = Some(selected - 1);  // Adjust selection index
                }
            }
        }
    }
}

// Main screen for array management: handles UI rendering and user input
pub fn array_management_screen(manager: &mut ArrayManager) -> bool {
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();

    let mut menu_selection = 0usize;    // Tracks which menu option is highlighted
    let mut array_selection = 0usize;   // Tracks which array is highlighted (for array-specific operations)

    loop {
        // Clear screen and draw UI
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "TOGISOFT ARRAY MANAGER";
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        let title_y = 2;
        stdout.queue(MoveTo(title_x, title_y)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Menu Options ---
        let menu_options = [
            "1. Generate New Random Array",
            "2. Select Array for Sorting",
            "3. View Array Details",
            "4. Delete Array",
            "5. Back to Main Menu"
        ];
        let menu_y = title_y + 3;
        for (i, option) in menu_options.iter().enumerate() {
            let option_x = (width.saturating_sub(option.len() as u16)) / 2;
            stdout.queue(MoveTo(option_x, menu_y + i as u16)).unwrap();
            if i == menu_selection {
                // Highlight selected menu option
                stdout.queue(SetForegroundColor(Color::Black)).unwrap();
                stdout.queue(SetBackgroundColor(Color::White)).unwrap();
            } else {
                stdout.queue(SetForegroundColor(Color::White)).unwrap();
                stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
            }
            stdout.queue(Print(format!(" {} ", option))).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        // --- Arrays List Section ---
        let arrays_title = "Available Arrays:";
        let arrays_y = menu_y + menu_options.len() as u16 + 2;
        stdout.queue(MoveTo(5, arrays_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(Print(arrays_title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        if manager.arrays.is_empty() {
            // Show placeholder if no arrays exist
            let no_arrays_msg = "No arrays created yet. Generate one to get started!";
            let msg_x = (width.saturating_sub(no_arrays_msg.len() as u16)) / 2;
            stdout.queue(MoveTo(msg_x, arrays_y + 2)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(no_arrays_msg)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            // List all arrays with metadata and preview
            for (i, array_data) in manager.arrays.iter().enumerate() {
                let y_pos = arrays_y + 2 + i as u16;
                let array_info = format!(
                    "{}: \"{}\" [Size: {}, Range: {}-{}]",
                    i + 1,
                    array_data.name,
                    array_data.size,
                    array_data.min_value,
                    array_data.max_value
                );
                stdout.queue(MoveTo(8, y_pos)).unwrap();

                // Highlight if this array is selected for sorting
                if manager.selected_index == Some(i) {
                    stdout.queue(SetForegroundColor(Color::Green)).unwrap();
                    stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
                    stdout.queue(Print("► [SELECTED] ")).unwrap();
                    stdout.queue(ResetColor).unwrap();
                }

                // Highlight if this array is currently being navigated
                if (menu_selection >= 1 && menu_selection <= 3) && i == array_selection {
                    stdout.queue(SetBackgroundColor(Color::DarkGrey)).unwrap();
                    stdout.queue(SetForegroundColor(Color::White)).unwrap();
                } else {
                    stdout.queue(SetForegroundColor(Color::White)).unwrap();
                }
                stdout.queue(Print(array_info)).unwrap();
                stdout.queue(ResetColor).unwrap();

                // Show preview of array data
                let preview = display_array_preview(&array_data.data);
                stdout.queue(MoveTo(12, y_pos + 1)).unwrap();
                stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                stdout.queue(Print(preview)).unwrap();
                stdout.queue(ResetColor).unwrap();
            }
        }

        // --- Instructions ---
        let instructions = if (menu_selection >= 1 && menu_selection <= 3) && !manager.arrays.is_empty() {
            vec![
                "Use ↑/↓ to select array, ENTER to choose",
                "Press LEFT arrow to go back to menu",
                "Press ESC to cancel",
            ]
        } else {
            vec![
                "Use ↑/↓ arrows to navigate menu",
                "Press ENTER to select option",
                "Press ESC to go back",
            ]
        };
        let inst_y = height.saturating_sub(instructions.len() as u16 + 2);
        for (i, instruction) in instructions.iter().enumerate() {
            let inst_x = (width.saturating_sub(instruction.len() as u16)) / 2;
            stdout.queue(MoveTo(inst_x, inst_y + i as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(*instruction)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        stdout.flush().unwrap();

        // --- Handle Input ---
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Up => {
                            if (menu_selection >= 1 && menu_selection <= 3) && !manager.arrays.is_empty() {
                                // Navigate array list
                                array_selection = if array_selection > 0 {
                                    array_selection - 1
                                } else {
                                    manager.arrays.len() - 1
                                };
                            } else {
                                // Navigate menu
                                menu_selection = if menu_selection > 0 {
                                    menu_selection - 1
                                } else {
                                    menu_options.len() - 1
                                };
                            }
                        },
                        KeyCode::Down => {
                            if (menu_selection >= 1 && menu_selection <= 3) && !manager.arrays.is_empty() {
                                // Navigate array list
                                array_selection = (array_selection + 1) % manager.arrays.len();
                            } else {
                                // Navigate menu
                                menu_selection = (menu_selection + 1) % menu_options.len();
                            }
                        },
                        KeyCode::Left => {
                            // Exit array selection mode
                            if menu_selection >= 1 && menu_selection <= 3 {
                                menu_selection = if menu_selection > 0 {
                                    menu_selection - 1
                                } else {
                                    menu_options.len() - 1
                                };
                            }
                        },
                        KeyCode::Right => {
                            // Move forward in menu
                            menu_selection = (menu_selection + 1) % menu_options.len();
                        },
                        KeyCode::Enter => {
                            match menu_selection {
                                0 => {
                                    // Generate New Random Array
                                    if let Some(array) = generate_random_array_dialog() {
                                        manager.add_array(array);
                                    }
                                },
                                1 => {
                                    // Select Array for Sorting
                                    if !manager.arrays.is_empty() {
                                        manager.selected_index = Some(array_selection);
                                        show_selection_confirmation(&manager.arrays[array_selection]);
                                    }
                                },
                                2 => {
                                    // View Array Details
                                    if !manager.arrays.is_empty() {
                                        show_array_details(&manager.arrays[array_selection]);
                                    }
                                },
                                3 => {
                                    // Delete Array
                                    if !manager.arrays.is_empty() {
                                        if confirm_delete(&manager.arrays[array_selection]) {
                                            manager.remove_array(array_selection);
                                            if array_selection >= manager.arrays.len() && !manager.arrays.is_empty() {
                                                array_selection = manager.arrays.len() - 1;
                                            }
                                        }
                                    }
                                },
                                4 => {
                                    // Back to Main Menu
                                    cleanup_terminal();
                                    return false;
                                },
                                _ => {}
                            }
                        },
                        KeyCode::Esc => {
                            cleanup_terminal();
                            return false;
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

// Dialog for generating a new random array: prompts for size and name
fn generate_random_array_dialog() -> Option<ArrayData> {
    let mut stdout = stdout();
    let mut input_string = String::new();   // Stores array size input
    let mut name_string = String::new();    // Stores array name input
    let mut input_mode = 0;                 // 0: size input, 1: name input
    let mut cursor_pos = 0usize;

    loop {
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "Generate New Random Array";
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, height / 2 - 8)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Size Input ---
        let size_label = "Array Size (2-50):";
        stdout.queue(MoveTo(width / 2 - 28, height / 2 - 5)).unwrap();
        stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
        stdout.queue(Print(size_label)).unwrap();
        stdout.queue(ResetColor).unwrap();
        draw_input_box(&mut stdout, width / 2 - 10, height / 2 - 4, 20, &input_string, cursor_pos, input_mode == 0);

        // --- Name Input ---
        let name_label = "Array Name:";
        stdout.queue(MoveTo(width / 2 - 28, height / 2 - 2)).unwrap();
        stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
        stdout.queue(Print(name_label)).unwrap();
        stdout.queue(ResetColor).unwrap();
        draw_input_box(&mut stdout, width / 2 - 10, height / 2 - 1, 20, &name_string, if input_mode == 1 { cursor_pos } else { 0 }, input_mode == 1);

        // --- Instructions ---
        let instructions = [
            "Press TAB to switch between fields",
            "Press ENTER when ready to generate",
            "Press ESC to cancel"
        ];
        for (i, instruction) in instructions.iter().enumerate() {
            let inst_x = (width.saturating_sub(instruction.len() as u16)) / 2;
            stdout.queue(MoveTo(inst_x, height / 2 + 2 + i as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(*instruction)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        stdout.flush().unwrap();

        // --- Handle Input ---
        if poll(Duration::from_millis(50)).unwrap_or(false) {
            match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Tab => {
                            // Switch between size and name fields
                            input_mode = (input_mode + 1) % 2;
                            cursor_pos = if input_mode == 0 {
                                input_string.len()
                            } else {
                                name_string.len()
                            };
                        },
                        KeyCode::Char(c) => {
                            // Handle character input
                            if input_mode == 0 && c.is_ascii_digit() && input_string.len() < 2 {
                                input_string.insert(cursor_pos, c);
                                cursor_pos += 1;
                            } else if input_mode == 1 && name_string.len() < 18 {
                                name_string.insert(cursor_pos, c);
                                cursor_pos += 1;
                            }
                        },
                        KeyCode::Backspace => {
                            // Handle backspace
                            if input_mode == 0 && cursor_pos > 0 {
                                cursor_pos -= 1;
                                input_string.remove(cursor_pos);
                            } else if input_mode == 1 && cursor_pos > 0 {
                                cursor_pos -= 1;
                                name_string.remove(cursor_pos);
                            }
                        },
                        KeyCode::Enter => {
                            // Generate array if input is valid
                            if let Ok(size) = input_string.trim().parse::<usize>() {
                                if size >= 2 && size <= 50 {
                                    let array_name = if name_string.trim().is_empty() {
                                        format!("Array_{}", size)
                                    } else {
                                        name_string.trim().to_string()
                                    };
                                    let mut rng = rand::rng();
                                    let data: Vec<u32> = (0..size)
                                        .map(|_| rng.random_range(1..=100))
                                        .collect();
                                    return Some(ArrayData::new(data, array_name));
                                }
                            }
                        },
                        KeyCode::Esc => {
                            return None;
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

// Renders an input box with border, content, and cursor
fn draw_input_box(stdout: &mut std::io::Stdout, x: u16, y: u16, width: u16, text: &str, cursor_pos: usize, active: bool) {
    // Draw border
    let border_color = if active { Color::Yellow } else { Color::White };
    let bg_color = if active { Color::White } else { Color::DarkGrey };
    stdout.queue(MoveTo(x.saturating_sub(1), y.saturating_sub(1))).unwrap();
    stdout.queue(SetForegroundColor(border_color)).unwrap();
    stdout.queue(Print("┌".to_string() + &"─".repeat(width as usize) + "┐")).unwrap();
    stdout.queue(MoveTo(x.saturating_sub(1), y)).unwrap();
    stdout.queue(Print("│")).unwrap();
    stdout.queue(MoveTo(x + width, y)).unwrap();
    stdout.queue(Print("│")).unwrap();
    stdout.queue(MoveTo(x.saturating_sub(1), y + 1)).unwrap();
    stdout.queue(Print("└".to_string() + &"─".repeat(width as usize) + "┘")).unwrap();

    // Draw content
    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.queue(SetBackgroundColor(bg_color)).unwrap();
    stdout.queue(SetForegroundColor(Color::Black)).unwrap();
    stdout.queue(Print(format!("{:<width$}", text, width = width as usize))).unwrap();

    // Draw cursor
    if active && cursor_pos < width as usize {
        stdout.queue(MoveTo(x + cursor_pos as u16, y)).unwrap();
        stdout.queue(SetBackgroundColor(Color::Yellow)).unwrap();
        let cursor_char = if cursor_pos < text.len() {
            text.chars().nth(cursor_pos).unwrap_or(' ')
        } else {
            ' '
        };
        stdout.queue(Print(cursor_char)).unwrap();
    }
    stdout.queue(ResetColor).unwrap();
}

// Shows a confirmation message after selecting an array for sorting
fn show_selection_confirmation(array_data: &ArrayData) {
    let mut stdout = stdout();
    let (width, height) = size().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();

    // --- Title ---
    let title = "Array Selected for Sorting!";
    let title_x = (width.saturating_sub(title.len() as u16)) / 2;
    stdout.queue(MoveTo(title_x, height / 2 - 3)).unwrap();
    stdout.queue(SetForegroundColor(Color::Green)).unwrap();
    stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
    stdout.queue(Print(title)).unwrap();
    stdout.queue(ResetColor).unwrap();

    // --- Info ---
    let info = format!("Selected: \"{}\" (Size: {})", array_data.name, array_data.size);
    let info_x = (width.saturating_sub(info.len() as u16)) / 2;
    stdout.queue(MoveTo(info_x, height / 2 - 1)).unwrap();
    stdout.queue(SetForegroundColor(Color::White)).unwrap();
    stdout.queue(Print(info)).unwrap();
    stdout.queue(ResetColor).unwrap();

    // --- Instruction ---
    let instruction = "Press any key to continue...";
    let inst_x = (width.saturating_sub(instruction.len() as u16)) / 2;
    stdout.queue(MoveTo(inst_x, height / 2 + 1)).unwrap();
    stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print(instruction)).unwrap();
    stdout.queue(ResetColor).unwrap();
    stdout.flush().unwrap();

    // Wait for keypress
    loop {
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(_) = read() {
                break;
            }
        }
    }
}

// Displays detailed information about an array
fn show_array_details(array_data: &ArrayData) {
    let mut stdout = stdout();
    let (width, height) = size().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();

    // --- Title ---
    let title = format!("Array Details: \"{}\"", array_data.name);
    let title_x = (width.saturating_sub(title.len() as u16)) / 2;
    stdout.queue(MoveTo(title_x, height / 2 - 8)).unwrap();
    stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
    stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
    stdout.queue(Print(title)).unwrap();
    stdout.queue(ResetColor).unwrap();

    // --- Details ---
    let details = [
        format!("Size: {}", array_data.size),
        format!("Min Value: {}", array_data.min_value),
        format!("Max Value: {}", array_data.max_value),
        format!("Range: {} - {}", array_data.min_value, array_data.max_value),
    ];
    for (i, detail) in details.iter().enumerate() {
        let detail_x = (width.saturating_sub(detail.len() as u16)) / 2;
        stdout.queue(MoveTo(detail_x, height / 2 - 5 + i as u16)).unwrap();
        stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
        stdout.queue(Print(detail)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // --- Array Content ---
    let array_title = "Array Content:";
    let array_title_x = (width.saturating_sub(array_title.len() as u16)) / 2;
    stdout.queue(MoveTo(array_title_x, height / 2)).unwrap();
    stdout.queue(SetForegroundColor(Color::Green)).unwrap();
    stdout.queue(Print(array_title)).unwrap();
    stdout.queue(ResetColor).unwrap();

    let content = display_array_full(&array_data.data, width as usize - 4);
    for (i, line) in content.iter().enumerate() {
        let line_x = (width.saturating_sub(line.len() as u16)) / 2;
        stdout.queue(MoveTo(line_x, height / 2 + 2 + i as u16)).unwrap();
        stdout.queue(SetForegroundColor(Color::White)).unwrap();
        stdout.queue(Print(line)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // --- Instruction ---
    let instruction = "Press any key to continue...";
    let inst_x = (width.saturating_sub(instruction.len() as u16)) / 2;
    stdout.queue(MoveTo(inst_x, height - 2)).unwrap();
    stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print(instruction)).unwrap();
    stdout.queue(ResetColor).unwrap();
    stdout.flush().unwrap();

    // Wait for keypress
    loop {
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(_) = read() {
                break;
            }
        }
    }
}

// Prompts for confirmation before deleting an array
fn confirm_delete(array_data: &ArrayData) -> bool {
    let mut stdout = stdout();
    let (width, height) = size().unwrap();

    loop {
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "Confirm Deletion";
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, height / 2 - 4)).unwrap();
        stdout.queue(SetForegroundColor(Color::Red)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Question ---
        let question = format!("Delete array \"{}\"?", array_data.name);
        let question_x = (width.saturating_sub(question.len() as u16)) / 2;
        stdout.queue(MoveTo(question_x, height / 2 - 2)).unwrap();
        stdout.queue(SetForegroundColor(Color::White)).unwrap();
        stdout.queue(Print(question)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Options ---
        let options = "Press Y to confirm, N to cancel";
        let options_x = (width.saturating_sub(options.len() as u16)) / 2;
        stdout.queue(MoveTo(options_x, height / 2)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(options)).unwrap();
        stdout.queue(ResetColor).unwrap();
        stdout.flush().unwrap();

        // --- Handle Input ---
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => return true,
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

// Returns a short preview of the array for the list view
fn display_array_preview(arr: &[u32]) -> String {
    if arr.len() <= 8 {
        format!("[{}]", arr.iter().map(|x| format!("{:2}", x)).collect::<Vec<_>>().join(", "))
    } else {
        let preview: Vec<String> = arr.iter().take(6).map(|x| format!("{:2}", x)).collect();
        format!("[{}, ... +{} more]", preview.join(", "), arr.len() - 6)
    }
}

// Returns the full array content, split into lines if necessary
fn display_array_full(arr: &[u32], max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::from("[");
    for (i, value) in arr.iter().enumerate() {
        let value_str = if i == arr.len() - 1 {
            format!("{:2}]", value)
        } else {
            format!("{:2}, ", value)
        };
        if current_line.len() + value_str.len() > max_width && current_line.len() > 1 {
            lines.push(current_line);
            current_line = format!(" {}", value_str);
        } else {
            current_line.push_str(&value_str);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

