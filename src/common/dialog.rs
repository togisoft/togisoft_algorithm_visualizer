use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, size},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Attribute, SetAttribute},
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
};
use std::io::{stdout, Write};
use std::time::Duration;

// Represents the type of dialog, which affects colors, icons, and default titles
#[derive(Clone, Copy)]
pub enum DialogType {
    Info,      // Informational dialog (blue)
    Warning,   // Warning dialog (yellow)
    Error,     // Error dialog (red)
    Success,   // Success dialog (green)
    Question,  // Question dialog (magenta)
}

impl DialogType {
    // Returns the colors and icon associated with each dialog type
    fn get_colors(&self) -> (Color, Color, &str) {
        match self {
            DialogType::Info => (Color::Blue, Color::Cyan, "ℹ"),
            DialogType::Warning => (Color::Yellow, Color::DarkYellow, "⚠"),
            DialogType::Error => (Color::Red, Color::DarkRed, "✖"),
            DialogType::Success => (Color::Green, Color::DarkGreen, "✓"),
            DialogType::Question => (Color::Magenta, Color::DarkMagenta, "?"),
        }
    }

    // Returns the default title for each dialog type
    fn get_title(&self) -> &str {
        match self {
            DialogType::Info => "INFORMATION",
            DialogType::Warning => "WARNING",
            DialogType::Error => "ERROR",
            DialogType::Success => "SUCCESS",
            DialogType::Question => "QUESTION",
        }
    }
}

// A customizable dialog box for user interaction
pub struct Dialog {
    dialog_type: DialogType,    // Type of dialog (affects colors and icon)
    title: String,              // Custom title for the dialog
    message: Vec<String>,       // Message content, split into lines
    buttons: Vec<String>,       // Buttons to display (e.g., "OK", "Cancel")
    selected_button: usize,     // Index of the currently selected button
    width: u16,                 // Width of the dialog box
    height: u16,                // Height of the dialog box
}

impl Dialog {
    // Creates a new dialog with the given type, title, and message
    pub fn new(dialog_type: DialogType, title: &str, message: &str) -> Self {
        let message_lines: Vec<String> = message
            .lines()
            .map(|line| line.to_string())
            .collect();
        let max_line_length = message_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
            .max(title.len());
        let width = (max_line_length + 8).min(60) as u16;
        // Height +1 for extra controls space
        let height = (message_lines.len() + 9) as u16;
        Self {
            dialog_type,
            title: title.to_string(),
            message: message_lines,
            buttons: vec!["OK".to_string()],
            selected_button: 0,
            width,
            height,
        }
    }

    // Sets custom buttons for the dialog
    pub fn with_buttons(mut self, buttons: Vec<&str>) -> Self {
        self.buttons = buttons.into_iter().map(|s| s.to_string()).collect();
        self.selected_button = 0;
        self
    }

    // Displays the dialog and returns the index of the selected button
    pub fn show(&mut self) -> usize {
        let mut stdout = stdout();
        let original_raw_mode = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);
        if !original_raw_mode {
            enable_raw_mode().unwrap();
        }
        let use_alt_screen = !original_raw_mode;
        if use_alt_screen {
            stdout.execute(EnterAlternateScreen).unwrap();
        }
        let result = self.run_dialog_loop(&mut stdout);
        if use_alt_screen {
            stdout.execute(LeaveAlternateScreen).unwrap();
        }
        if !original_raw_mode {
            disable_raw_mode().unwrap();
        }
        result
    }

    // Optional: Prints controls hint after the dialog (outside the box)
    fn print_controls_after(&self, stdout: &mut std::io::Stdout) {
        let controls = "←/→ TAB nav, ENTER select, ESC cancel";
        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
        stdout.execute(SetForegroundColor(Color::DarkGrey)).unwrap();
        print!("{}", controls);
        stdout.flush().unwrap();
        println!(); // Move to the next line
    }

    // Main loop for handling user input and rendering the dialog
    fn run_dialog_loop(&mut self, stdout: &mut std::io::Stdout) -> usize {
        loop {
            self.draw(stdout);
            if poll(Duration::from_millis(50)).unwrap_or(false) {
                match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Left => {
                                // Navigate left between buttons
                                if self.selected_button > 0 {
                                    self.selected_button -= 1;
                                } else {
                                    self.selected_button = self.buttons.len() - 1;
                                }
                            },
                            KeyCode::Right => {
                                // Navigate right between buttons
                                self.selected_button = (self.selected_button + 1) % self.buttons.len();
                            },
                            KeyCode::Tab => {
                                // Tab to cycle through buttons
                                self.selected_button = (self.selected_button + 1) % self.buttons.len();
                            },
                            KeyCode::Enter => {
                                // Enter to select the current button
                                return self.selected_button;
                            },
                            KeyCode::Esc => {
                                // ESC to cancel (selects the last button)
                                return self.buttons.len().saturating_sub(1);
                            },
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                // Quick 'Yes' for yes/no dialogs
                                if self.buttons.iter().any(|b| b.to_lowercase().contains("yes")) {
                                    return 0;
                                }
                            },
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                // Quick 'No' for yes/no dialogs
                                if self.buttons.iter().any(|b| b.to_lowercase().contains("no")) {
                                    return 1;
                                }
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Renders the dialog to the terminal
    fn draw(&self, stdout: &mut std::io::Stdout) {
        let (term_width, term_height) = size().unwrap();
        // Don't clear if overlaying on existing content
        if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) {
            stdout.execute(Clear(ClearType::All)).unwrap();
        }
        let (primary_color, _, icon) = self.dialog_type.get_colors();

        // Calculate dialog position (centered)
        let dialog_x = (term_width.saturating_sub(self.width)) / 2;
        let dialog_y = (term_height.saturating_sub(self.height)) / 2;

        // Draw background/border
        self.draw_border(stdout, dialog_x, dialog_y, primary_color);

        // Draw title with icon
        let title_with_icon = format!("{} {} {}", icon, self.dialog_type.get_title(), icon);
        let title_x = dialog_x + (self.width.saturating_sub(title_with_icon.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, dialog_y + 1)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(SetForegroundColor(primary_color)).unwrap();
        stdout.queue(Print(title_with_icon)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // Draw custom title if provided
        if !self.title.is_empty() && self.title != self.dialog_type.get_title() {
            let custom_title_x = dialog_x + (self.width.saturating_sub(self.title.len() as u16)) / 2;
            stdout.queue(MoveTo(custom_title_x, dialog_y + 2)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(&self.title)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        // Draw message
        let message_start_y = dialog_y + if self.title.is_empty() || self.title == self.dialog_type.get_title() { 3 } else { 4 };
        for (i, line) in self.message.iter().enumerate() {
            let line_x = dialog_x + (self.width.saturating_sub(line.len() as u16)) / 2;
            stdout.queue(MoveTo(line_x, message_start_y + i as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(line)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        // Draw buttons
        let buttons_y = dialog_y + self.height - 4;
        let total_button_width: usize = self.buttons.iter().map(|b| b.len() + 4).sum::<usize>() + (self.buttons.len().saturating_sub(1) * 2);
        let buttons_start_x = dialog_x + (self.width.saturating_sub(total_button_width as u16)) / 2;
        let mut current_x = buttons_start_x;
        for (i, button) in self.buttons.iter().enumerate() {
            let button_text = format!(" {} ", button);
            stdout.queue(MoveTo(current_x, buttons_y)).unwrap();
            if i == self.selected_button {
                stdout.queue(SetBackgroundColor(primary_color)).unwrap();
                stdout.queue(SetForegroundColor(Color::Black)).unwrap();
                stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            } else {
                stdout.queue(SetForegroundColor(primary_color)).unwrap();
                stdout.queue(SetBackgroundColor(Color::Reset)).unwrap();
            }
            stdout.queue(Print("[ ".to_string() + &button_text + " ]")).unwrap();
            stdout.queue(ResetColor).unwrap();
            current_x += (button_text.len() + 4) as u16 + 2; // Button + brackets + spacing
        }

        // Draw controls hint
        let controls = "←/→ TAB nav, ENTER select, ESC cancel";
        let controls_x = dialog_x + (self.width.saturating_sub(controls.len() as u16)) / 2;
        stdout.queue(MoveTo(controls_x, dialog_y + self.height - 3)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(controls)).unwrap();
        stdout.queue(ResetColor).unwrap();

        stdout.flush().unwrap();
    }

    // Draws the border of the dialog box
    fn draw_border(&self, stdout: &mut std::io::Stdout, x: u16, y: u16, color: Color) {
        stdout.queue(SetForegroundColor(color)).unwrap();
        // Top border
        stdout.queue(MoveTo(x, y)).unwrap();
        stdout.queue(Print("╔".to_string() + &"═".repeat((self.width - 2) as usize) + "╗")).unwrap();
        // Side borders
        for i in 1..self.height - 1 {
            stdout.queue(MoveTo(x, y + i)).unwrap();
            stdout.queue(Print("║")).unwrap();
            stdout.queue(MoveTo(x + self.width - 1, y + i)).unwrap();
            stdout.queue(Print("║")).unwrap();
        }
        // Bottom border
        stdout.queue(MoveTo(x, y + self.height - 1)).unwrap();
        stdout.queue(Print("╚".to_string() + &"═".repeat((self.width - 2) as usize) + "╝")).unwrap();
        stdout.queue(ResetColor).unwrap();
    }
}

// Convenience functions for common dialog types
pub fn show_info(title: &str, message: &str) {
    Dialog::new(DialogType::Info, title, message).show();
}

pub fn show_warning(title: &str, message: &str) {
    Dialog::new(DialogType::Warning, title, message).show();
}

pub fn show_error(title: &str, message: &str) {
    Dialog::new(DialogType::Error, title, message).show();
}

pub fn show_success(title: &str, message: &str) {
    Dialog::new(DialogType::Success, title, message).show();
}

pub fn show_question(title: &str, message: &str, buttons: Vec<&str>) -> usize {
    Dialog::new(DialogType::Question, title, message)
        .with_buttons(buttons)
        .show()
}

// Specific helper functions for common use cases
pub fn show_no_array_selected() {
    show_warning(
        "No Array Selected",
        "Please select an array first from the Array Manager.\n\nGo to 'Generate/Select Array' option to create\nor select an array for sorting."
    );
}

pub fn confirm_exit() -> bool {
    let result = show_question(
        "Confirm Exit",
        "Are you sure you want to exit the application?",
        vec!["Yes", "No"]
    );
    result == 0
}

pub fn confirm_reset_array() -> bool {
    let result = show_question(
        "Confirm Reset",
        "This will reset the array to its original state.\nAll progress will be lost.\n\nContinue?",
        vec!["Yes", "No"]
    );
    result == 0
}

pub fn show_sorting_completed(algorithm: &str, comparisons: u32, swaps: u32, time_ms: u64) {
    let message = format!(
        "Algorithm: {}\n\nStatistics:\n• Comparisons: {}\n• Swaps: {}\n• Time: {}ms\n\nThe array has been successfully sorted!",
        algorithm, comparisons, swaps, time_ms
    );
    show_success("Sorting Completed", &message);
}

pub fn show_array_info(array_name: &str, size: usize, min_val: u32, max_val: u32) {
    let message = format!(
        "Array: {}\n\nDetails:\n• Size: {} elements\n• Range: {} - {}\n• Ready for sorting",
        array_name, size, min_val, max_val
    );
    show_info("Array Information", &message);
}

pub fn show_algorithm_info(algorithm: &str) -> bool {
    let (description, complexity) = match algorithm {
        "Bubble Sort" => (
            "Bubble Sort repeatedly compares adjacent elements\nand swaps them if they are in wrong order.\n\nPros: Simple to understand and implement\nCons: Inefficient for large datasets",
            "Time: O(n²) | Space: O(1)"
        ),
        "Quick Sort" => (
            "Quick Sort picks a pivot element and partitions\nthe array around it, then recursively sorts.\n\nPros: Generally fast, in-place sorting\nCons: Worst case O(n²) performance",
            "Time: O(n log n) avg | Space: O(log n)"
        ),
        "Merge Sort" => (
            "Merge Sort divides array into halves, sorts them\nrecursively, then merges sorted halves.\n\nPros: Stable, consistent O(n log n)\nCons: Requires additional memory",
            "Time: O(n log n) | Space: O(n)"
        ),
        _ => (
            "Information about this algorithm\nwill be displayed here.",
            "Complexity information"
        )
    };
    let message = format!("{}\n\nComplexity:\n{}\n\nWould you like to proceed with this algorithm?", description, complexity);
    let result = show_question(
        &format!("{} Information", algorithm),
        &message,
        vec!["Start Sorting", "Go Back"]
    );
    result == 0
}

pub fn show_loading(message: &str) {
    show_info("Please Wait", &format!("{}...", message));
}

// Progress dialog for long operations
pub struct ProgressDialog {
    title: String,             // Title of the progress dialog
    current_step: usize,       // Current step number
    total_steps: usize,        // Total number of steps
    current_message: String,   // Current message to display
}

impl ProgressDialog {
    // Creates a new progress dialog
    pub fn new(title: &str, total_steps: usize) -> Self {
        Self {
            title: title.to_string(),
            current_step: 0,
            total_steps,
            current_message: String::new(),
        }
    }

    // Updates the progress and message, then redraws
    pub fn update(&mut self, step: usize, message: &str) {
        self.current_step = step;
        self.current_message = message.to_string();
        self.draw();
    }

    // Renders the progress dialog
    fn draw(&self) {
        let progress = if self.total_steps > 0 {
            (self.current_step as f64 / self.total_steps as f64 * 100.0) as usize
        } else {
            0
        };
        let progress_bar = "█".repeat(progress / 2) + &"░".repeat(50 - progress / 2);
        let message = format!(
            "{}\n\nProgress: {}/{} ({}%)\n[{}]\n\n{}",
            self.title,
            self.current_step,
            self.total_steps,
            progress,
            progress_bar,
            self.current_message
        );
        show_info("Working", &message);
    }
}
