use crate::array_manager::ArrayData;
use crate::enums::SelectionState;
use crate::helper::cleanup_terminal;
use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen}, ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;

// Visualizes the bubble sort algorithm step-by-step with interactive controls
pub struct BubbleSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    current_i: usize,          // Current outer loop index (pass number)
    current_j: usize,          // Current inner loop index (comparison index)
    sorted_count: usize,       // Number of elements already sorted
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of swaps made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete
}

impl BubbleSortVisualizer {
    // Initializes a new BubbleSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();
        Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            current_i: 0,
            current_j: 0,
            sorted_count: 0,
            comparisons: 0,
            swaps: 0,
            is_running: false,
            is_paused: false,
            speed: Duration::from_millis(500),
            completed: false,
        }
    }

    // Main loop: handles rendering, input, and stepping through the sort
    pub fn run_visualization(&mut self) {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(EnterAlternateScreen).unwrap();

        loop {
            self.draw(&mut stdout);

            // --- Handle Input ---
            if poll(Duration::from_millis(50)).unwrap_or(false) {
                match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Char(' ') => {
                                // Space: Toggle play/pause or restart if completed
                                if self.completed {
                                    self.reset();
                                } else if self.is_running {
                                    self.is_paused = !self.is_paused;
                                } else {
                                    self.is_running = true;
                                    self.is_paused = false;
                                }
                            },
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                // R: Reset the visualization
                                self.reset();
                            },
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // S: Step through the sort manually
                                if !self.completed && !self.is_running {
                                    self.step();
                                }
                            },
                            KeyCode::Char('+') => {
                                // +: Increase speed (decrease delay)
                                self.speed = Duration::from_millis(
                                    (self.speed.as_millis() as u64).saturating_sub(50).max(50)
                                );
                            },
                            KeyCode::Char('-') => {
                                // -: Decrease speed (increase delay)
                                self.speed = Duration::from_millis(
                                    (self.speed.as_millis() as u64 + 50).min(2000)
                                );
                            },
                            KeyCode::Esc => {
                                // ESC: Exit the visualization
                                cleanup_terminal();
                                return;
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // --- Auto Step ---
            // Automatically step if running and not paused
            if self.is_running && !self.is_paused && !self.completed {
                std::thread::sleep(self.speed);
                if !self.step() {
                    self.is_running = false;
                    self.completed = true;
                    self.mark_all_sorted();
                }
            }
        }
    }

    // Renders the current state of the visualization to the terminal
    fn draw(&mut self, stdout: &mut std::io::Stdout) {
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "TOGISOFT BUBBLE SORT VISUALIZER";
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, 1)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Array Visualization ---
        self.draw_array(stdout, width, height);

        // --- Statistics ---
        self.draw_statistics(stdout, width, height);

        // --- Controls ---
        self.draw_controls(stdout, width, height);

        // --- Current Operation Info ---
        self.draw_current_operation(stdout, width, height);

        stdout.flush().unwrap();
    }

    // Draws the array as a series of bars, with colors indicating their state
    fn draw_array(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let array_start_y = 5;
        let max_value = *self.array.iter().max().unwrap_or(&1) as f64;
        let array_len = self.array.len();

        // Calculate bar dimensions and spacing
        let available_width = (width as usize).saturating_sub(4);
        let bar_width = if available_width / array_len >= 3 {
            3
        } else if available_width / array_len >= 2 {
            2
        } else {
            1
        };
        let spacing = if bar_width >= 2 { 1 } else { 0 };
        let total_width_needed = array_len * bar_width + (array_len - 1) * spacing;
        let start_x = (width as usize - total_width_needed) / 2;

        // Draw bars
        let max_bar_height = (height as usize).saturating_sub(15).min(20);
        for (i, &value) in self.array.iter().enumerate() {
            let bar_height = ((value as f64 / max_value) * max_bar_height as f64) as usize + 1;
            let x = start_x + i * (bar_width + spacing);

            // Choose color based on element state
            let (fg_color, bg_color) = match self.states[i] {
                SelectionState::Normal => (Color::Cyan, Color::Reset),
                SelectionState::Sorted => (Color::Green, Color::DarkGreen),
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow),
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta),
                SelectionState::Selected => (Color::White, Color::DarkBlue),
                SelectionState::Swapping => (Color::Red, Color::DarkRed),
                SelectionState::PartitionLeft | SelectionState::PartitionRight => (Color::Cyan, Color::Reset),
            };

            // Draw the bar from bottom up
            for h in 0..bar_height {
                let y = array_start_y + max_bar_height - h;
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                stdout.queue(SetForegroundColor(fg_color)).unwrap();
                stdout.queue(SetBackgroundColor(bg_color)).unwrap();
                if bar_width == 1 {
                    stdout.queue(Print("█")).unwrap();
                } else {
                    stdout.queue(Print("█".repeat(bar_width))).unwrap();
                }
                stdout.queue(ResetColor).unwrap();
            }

            // Draw value below bar
            let value_str = value.to_string();
            let value_x = x + (bar_width.saturating_sub(value_str.len())) / 2;
            stdout.queue(MoveTo(value_x as u16, (array_start_y + max_bar_height + 1) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(value_str)).unwrap();
            stdout.queue(ResetColor).unwrap();

            // Draw index below value
            let index_str = i.to_string();
            let index_x = x + (bar_width.saturating_sub(index_str.len())) / 2;
            stdout.queue(MoveTo(index_x as u16, (array_start_y + max_bar_height + 2) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(index_str)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        // --- Legend ---
        let legend_y = array_start_y + max_bar_height + 4;
        let legend_items = [
            ("Normal", Color::Cyan),
            ("Comparing", Color::Yellow),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ];
        let legend_start_x = (width as usize - 50) / 2;
        for (i, (label, color)) in legend_items.iter().enumerate() {
            let x = legend_start_x + i * 12;
            stdout.queue(MoveTo(x as u16, legend_y as u16)).unwrap();
            stdout.queue(SetForegroundColor(*color)).unwrap();
            stdout.queue(Print("██")).unwrap();
            stdout.queue(ResetColor).unwrap();
            stdout.queue(Print(format!(" {}", label))).unwrap();
        }
    }

    // Draws statistics such as comparisons, swaps, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(8);
        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Swaps: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Progress: {:.1}%", self.get_progress()),
        ];
        for (i, stat) in stats.iter().enumerate() {
            let x = 5 + (i % 3) * 25;
            let y = stats_y + (i / 3) as u16;
            stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
            stdout.queue(Print(stat)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Draws the control instructions and current status
    fn draw_controls(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let controls_y = height.saturating_sub(4);
        let status = if self.completed {
            "COMPLETED!"
        } else if self.is_running && !self.is_paused {
            "RUNNING..."
        } else if self.is_paused {
            "PAUSED"
        } else {
            "READY"
        };

        // Status
        stdout.queue(MoveTo(5, controls_y)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        let status_color = match status {
            "COMPLETED!" => Color::Green,
            "RUNNING..." => Color::Yellow,
            "PAUSED" => Color::Red,
            _ => Color::White,
        };
        stdout.queue(SetForegroundColor(status_color)).unwrap();
        stdout.queue(Print(format!("Status: {}", status))).unwrap();
        stdout.queue(ResetColor).unwrap();

        // Controls
        let controls = if self.completed {
            "SPACE: Restart | R: Reset | ESC: Exit"
        } else {
            "SPACE: Start/Pause | S: Step | R: Reset | +/-: Speed | ESC: Exit"
        };
        let controls_x = (width.saturating_sub(controls.len() as u16)) / 2;
        stdout.queue(MoveTo(controls_x, controls_y + 1)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(controls)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws information about the current operation (e.g., which elements are being compared)
    fn draw_current_operation(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        if self.completed {
            let message = "✓ Array is now sorted!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else if self.current_i < self.array.len() {
            let operation = if self.current_j < self.array.len() - 1 - self.current_i {
                format!(
                    "Pass {}: Comparing array[{}] ({}) with array[{}] ({})",
                    self.current_i + 1,
                    self.current_j,
                    self.array[self.current_j],
                    self.current_j + 1,
                    self.array[self.current_j + 1]
                )
            } else {
                format!("Pass {} completed", self.current_i + 1)
            };
            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Performs a single step of the bubble sort algorithm
    // Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset states (except for sorted elements)
        for state in &mut self.states {
            if *state != SelectionState::Sorted {
                *state = SelectionState::Normal;
            }
        }

        let n = self.array.len();
        if self.current_i >= n - 1 {
            return false; // Sorting complete
        }

        if self.current_j < n - 1 - self.current_i {
            // Mark elements being compared
            self.states[self.current_j] = SelectionState::Comparing;
            self.states[self.current_j + 1] = SelectionState::Comparing;
            self.comparisons += 1;

            // Check if swap is needed
            if self.array[self.current_j] > self.array[self.current_j + 1] {
                // Mark as swapping
                self.states[self.current_j] = SelectionState::Swapping;
                self.states[self.current_j + 1] = SelectionState::Swapping;
                // Perform swap
                self.array.swap(self.current_j, self.current_j + 1);
                self.swaps += 1;
            }
            self.current_j += 1;
        } else {
            // Mark the last element of this pass as sorted
            self.states[n - 1 - self.current_i] = SelectionState::Sorted;
            self.sorted_count += 1;
            self.current_i += 1;
            self.current_j = 0;
        }
        true
    }

    // Resets the visualization to its initial state
    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.current_i = 0;
        self.current_j = 0;
        self.sorted_count = 0;
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
    }

    // Marks all elements as sorted (used when sorting is complete)
    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    // Calculates the progress of the sorting as a percentage
    fn get_progress(&self) -> f64 {
        let total_comparisons_needed = (self.array.len() * (self.array.len() - 1)) / 2;
        if total_comparisons_needed == 0 {
            100.0
        } else {
            (self.comparisons as f64 / total_comparisons_needed as f64 * 100.0).min(100.0)
        }
    }
}

// Entry point for the bubble sort visualization
pub fn bubble_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = BubbleSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
