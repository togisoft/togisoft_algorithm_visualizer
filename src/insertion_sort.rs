use crate::array_manager::ArrayData;
use crate::helper::{cleanup_terminal};
use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen}, ExecutableCommand,
    QueueableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;
use crate::enums::SelectionState;

/// Visualizes the insertion sort algorithm step-by-step with interactive controls
pub struct InsertionSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, shifting, sorted)
    current_i: usize,          // Current outer loop index (element to insert)
    current_j: usize,          // Current inner loop index (position being compared)
    key: u32,                  // Current key element being inserted
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of shifts made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete
    phase: InsertionPhase,     // Current phase of the insertion sort algorithm
}

/// Represents the different phases of the insertion sort algorithm
#[derive(Clone, Copy, PartialEq)]
enum InsertionPhase {
    SelectingElement,    // Selecting the next element to insert
    SearchingPosition,   // Comparing and shifting elements to find the correct position
    InsertingElement,    // Inserting the element at its correct position
    MoveToNext,          // Moving to the next element
}

impl InsertionSortVisualizer {
    /// Creates a new InsertionSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();

        // If the array has 0 or 1 elements, it's already sorted
        if len <= 1 {
            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Sorted; len],
                current_i: 1,
                current_j: 0,
                key: 0,
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: true,
                phase: InsertionPhase::MoveToNext,
            }
        } else {
            // Initialize with the first element marked as sorted
            let mut states = vec![SelectionState::Normal; len];
            if len > 0 {
                states[0] = SelectionState::Sorted; // First element is always sorted
            }
            Self {
                original_array: array.clone(),
                array,
                states,
                current_i: 1,
                current_j: 0,
                key: 0,
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: false,
                phase: InsertionPhase::SelectingElement,
            }
        }
    }

    /// Main loop: handles rendering, input, and stepping through the sort
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
                                    (self.speed.as_millis() as u64).saturating_sub(50).max(100)
                                );
                            },
                            KeyCode::Char('-') => {
                                // -: Decrease speed (increase delay)
                                self.speed = Duration::from_millis(
                                    (self.speed.as_millis() as u64 + 100).min(3000)
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

    /// Renders the current state of the visualization to the terminal
    fn draw(&mut self, stdout: &mut std::io::Stdout) {
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "TOGISOFT INSERTION SORT VISUALIZER";
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

    /// Draws the array as a series of bars, with colors indicating their state
    fn draw_array(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let array_start_y = 5;
        let max_value = *self.array.iter().max().unwrap_or(&1) as f64;
        let array_len = self.array.len();
        if array_len == 0 {
            return;
        }

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
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow), // Key element
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta), // Being compared
                SelectionState::Selected => (Color::White, Color::DarkBlue), // Current position
                SelectionState::Swapping => (Color::Red, Color::DarkRed), // Being shifted
                SelectionState::PartitionLeft | SelectionState::PartitionRight => (Color::Blue, Color::DarkBlue),
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
            ("Key Element", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Position", Color::White),
            ("Shifting", Color::Red),
            ("Sorted", Color::Green),
        ];
        let legend_start_x = (width as usize - 90) / 2;
        for (i, (label, color)) in legend_items.iter().enumerate() {
            let x = legend_start_x + (i % 3) * 30;
            let y = legend_y + (i / 3);
            stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
            stdout.queue(SetForegroundColor(*color)).unwrap();
            stdout.queue(Print("██")).unwrap();
            stdout.queue(ResetColor).unwrap();
            stdout.queue(Print(format!(" {}", label))).unwrap();
        }
    }

    /// Draws statistics such as comparisons, shifts, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(8);
        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Shifts: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Current Index: {}", if self.current_i < self.array.len() { self.current_i.to_string() } else { "Done".to_string() }),
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

    /// Draws the control instructions and current status
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

    /// Draws information about the current operation
    fn draw_current_operation(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        if self.completed {
            let message = "✓ Array is now sorted using Insertion Sort!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            let operation = match self.phase {
                InsertionPhase::SelectingElement => {
                    if self.current_i < self.array.len() {
                        format!("Step {}/{}: Selecting key element {} (value: {})",
                                self.current_i, self.array.len() - 1, self.current_i, self.array[self.current_i])
                    } else {
                        "Selecting element...".to_string()
                    }
                },
                InsertionPhase::SearchingPosition => {
                    if self.current_j < self.array.len() && self.current_j + 1 < self.array.len() {
                        format!("Comparing key {} with element {} (value: {})",
                                self.key, self.current_j, self.array[self.current_j])
                    } else {
                        format!("Finding correct position for key {}", self.key)
                    }
                },
                InsertionPhase::InsertingElement => {
                    format!("Inserting key {} at position {}", self.key, self.current_j + 1)
                },
                InsertionPhase::MoveToNext => {
                    if self.current_i < self.array.len() {
                        format!("Element {} positioned correctly, moving to next", self.key)
                    } else {
                        "Insertion sort completed!".to_string()
                    }
                },
            };

            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    /// Performs a single step of the insertion sort algorithm
    /// Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset all non-sorted states
        for (i, state) in self.states.iter_mut().enumerate() {
            match *state {
                SelectionState::Sorted => {}
                _ => {
                    if i < self.current_i {
                        *state = SelectionState::Sorted;
                    } else {
                        *state = SelectionState::Normal;
                    }
                }
            }
        }

        match self.phase {
            InsertionPhase::SelectingElement => {
                if self.current_i >= self.array.len() {
                    return false; // Done sorting
                }

                // Select the key element
                self.key = self.array[self.current_i];
                self.states[self.current_i] = SelectionState::CurrentMin;
                self.current_j = if self.current_i > 0 { self.current_i - 1 } else { 0 };

                if self.current_i == 0 {
                    // First element is already sorted
                    self.states[0] = SelectionState::Sorted;
                    self.current_i += 1;
                    return self.current_i < self.array.len();
                }

                self.phase = InsertionPhase::SearchingPosition;
                true
            },
            InsertionPhase::SearchingPosition => {
                // Compare key with current element
                if self.current_j < self.array.len() {
                    self.states[self.current_j] = SelectionState::Comparing;
                    self.comparisons += 1;

                    if self.array[self.current_j] > self.key {
                        // Need to shift this element right
                        self.states[self.current_j] = SelectionState::Swapping;
                        if self.current_j + 1 < self.array.len() {
                            self.array[self.current_j + 1] = self.array[self.current_j];
                            self.swaps += 1;
                        }

                        if self.current_j > 0 {
                            self.current_j -= 1;
                        } else {
                            // Reached the beginning, insert here
                            self.phase = InsertionPhase::InsertingElement;
                        }
                    } else {
                        // Found correct position (after current element)
                        self.current_j += 1;
                        self.phase = InsertionPhase::InsertingElement;
                    }
                } else {
                    self.phase = InsertionPhase::InsertingElement;
                }
                true
            },
            InsertionPhase::InsertingElement => {
                // Insert the key at current_j position
                if self.current_j < self.array.len() {
                    self.array[self.current_j] = self.key;
                    self.states[self.current_j] = SelectionState::Selected;
                }

                self.phase = InsertionPhase::MoveToNext;
                true
            },
            InsertionPhase::MoveToNext => {
                // Move to next element
                self.current_i += 1;
                if self.current_i >= self.array.len() {
                    return false; // Done sorting
                }

                self.phase = InsertionPhase::SelectingElement;
                true
            },
        }
    }

    /// Resets the visualization to its initial state
    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];

        if len > 0 {
            self.states[0] = SelectionState::Sorted; // First element is always sorted
        }

        self.current_i = if len <= 1 { len } else { 1 };
        self.current_j = 0;
        self.key = 0;
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = len <= 1;
        self.phase = if len <= 1 { InsertionPhase::MoveToNext } else { InsertionPhase::SelectingElement };
    }

    /// Marks all elements as sorted (used when sorting is complete)
    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    /// Calculates the progress of the sorting as a percentage
    fn get_progress(&self) -> f64 {
        if self.array.len() <= 1 {
            100.0
        } else {
            ((self.current_i as f64) / (self.array.len() as f64) * 100.0).min(100.0)
        }
    }
}

/// Entry point for the insertion sort visualization
pub fn insertion_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = InsertionSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
