use crate::array_manager::ArrayData;
use crate::enums::SelectionState;
use crate::helper::cleanup_terminal;
use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen}, ExecutableCommand,
    QueueableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;

/// Represents the different phases of the quick sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum QuickPhase {
    ChoosingPivot,      // Selecting the pivot element
    PartitioningLeft,   // Moving the left pointer and comparing with pivot
    PartitioningRight,  // Moving the right pointer and comparing with pivot
    SwappingElements,   // Swapping elements at left and right pointers
    SwappingWithPivot,  // Swapping the pivot with its final position
    DonePartition,      // Partitioning is complete
}

/// Visualizes the quick sort algorithm step-by-step with interactive controls
pub struct QuickSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of swaps made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete

    // QuickSort specific fields
    stack: Vec<(usize, usize)>, // Stack of (low, high) pairs to process
    low: usize,                // Lower bound of the current subarray
    high: usize,               // Upper bound of the current subarray
    pivot_index: usize,        // Index of the pivot element
    left: usize,               // Left pointer for partitioning
    right: usize,              // Right pointer for partitioning
    phase: QuickPhase,         // Current phase of the quick sort algorithm
}

impl QuickSortVisualizer {
    /// Creates a new QuickSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();

        // If the array has 0 or 1 elements, it's already sorted
        if len <= 1 {
            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Sorted; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: true,
                stack: Vec::new(),
                low: 0,
                high: 0,
                pivot_index: 0,
                left: 0,
                right: 0,
                phase: QuickPhase::DonePartition,
            }
        } else {
            // Initialize with the full array range
            let mut stack = Vec::new();
            stack.push((0, len - 1));

            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Normal; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: false,
                stack,
                low: 0,
                high: len - 1,
                pivot_index: len - 1, // Start with last element as pivot
                left: 0,
                right: 0,
                phase: QuickPhase::ChoosingPivot,
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
        let title = "TOGISOFT QUICK SORT VISUALIZER";
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
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow), // Pivot
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta),
                SelectionState::Selected => (Color::White, Color::DarkBlue),
                SelectionState::Swapping => (Color::Red, Color::DarkRed),
                SelectionState::PartitionLeft => (Color::Blue, Color::DarkBlue), // Left pointer
                SelectionState::PartitionRight => (Color::AnsiValue(208), Color::DarkBlue), // Right pointer (orange)
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
            ("Pivot", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Left Ptr", Color::Blue),
            ("Right Ptr", Color::AnsiValue(208)), // Orange
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ];
        let legend_start_x = (width as usize - 100) / 2;
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

    /// Draws statistics such as comparisons, swaps, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(8);
        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Swaps: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Stack Size: {}", self.stack.len()),
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
            let message = "✓ Array is now sorted!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            let operation = match self.phase {
                QuickPhase::ChoosingPivot => {
                    format!("Partition [{}..{}]: Choosing pivot at index {}",
                            self.low, self.high, self.pivot_index)
                },
                QuickPhase::PartitioningLeft => {
                    if self.left < self.array.len() && self.pivot_index < self.array.len() {
                        format!("Partition [{}..{}]: left={} ({}) <= pivot {}?",
                                self.low, self.high, self.left, self.array[self.left], self.array[self.pivot_index])
                    } else {
                        format!("Partition [{}..{}]: Moving left pointer", self.low, self.high)
                    }
                },
                QuickPhase::PartitioningRight => {
                    if self.right < self.array.len() && self.pivot_index < self.array.len() {
                        format!("Partition [{}..{}]: right={} ({}) > pivot {}?",
                                self.low, self.high, self.right, self.array[self.right], self.array[self.pivot_index])
                    } else {
                        format!("Partition [{}..{}]: Moving right pointer", self.low, self.high)
                    }
                },
                QuickPhase::SwappingElements => {
                    if self.left < self.array.len() && self.right < self.array.len() {
                        format!("Swapping left={} ({}) with right={} ({})",
                                self.left, self.array[self.left], self.right, self.array[self.right])
                    } else {
                        "Swapping elements".to_string()
                    }
                },
                QuickPhase::SwappingWithPivot => {
                    format!("Final swap: pivot at {} with left={}", self.pivot_index, self.left)
                },
                QuickPhase::DonePartition => {
                    format!("Moving to next subarray")
                },
            };

            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, height.saturating_sub(6))).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    /// Performs a single step of the quick sort algorithm
    /// Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset states to normal except sorted
        for (i, state) in self.states.iter_mut().enumerate() {
            match *state {
                SelectionState::Sorted => {}
                _ => *state = SelectionState::Normal,
            }
        }

        match self.phase {
            QuickPhase::ChoosingPivot => {
                // Get a new range from the stack
                if let Some((l, h)) = self.stack.pop() {
                    self.low = l;
                    self.high = h;

                    // Single element range, mark as sorted
                    if self.low >= self.high {
                        if self.low == self.high && self.low < self.array.len() {
                            self.states[self.low] = SelectionState::Sorted;
                        }
                        self.phase = QuickPhase::ChoosingPivot;
                        return self.step(); // Move to next range
                    }

                    // Choose pivot (last element)
                    self.pivot_index = self.high;
                    self.states[self.pivot_index] = SelectionState::CurrentMin;

                    // Initialize pointers
                    self.left = self.low;
                    if self.high > 0 {
                        self.right = self.high - 1;
                    } else {
                        self.right = 0;
                    }

                    self.phase = QuickPhase::PartitioningLeft;
                } else {
                    // Stack is empty, algorithm is complete
                    return false;
                }
                true
            },
            QuickPhase::PartitioningLeft => {
                if self.left <= self.right {
                    self.states[self.left] = SelectionState::PartitionLeft;
                    self.comparisons += 1;

                    // Move left pointer if element is less than or equal to pivot
                    if self.array[self.left] <= self.array[self.pivot_index] {
                        self.left += 1;
                    } else {
                        // Element is greater than pivot, move to right pointer
                        self.phase = QuickPhase::PartitioningRight;
                    }
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::PartitioningRight => {
                if self.left <= self.right {
                    self.states[self.right] = SelectionState::PartitionRight;
                    self.comparisons += 1;

                    // Move right pointer if element is greater than pivot
                    if self.array[self.right] > self.array[self.pivot_index] {
                        if self.right > 0 {
                            self.right -= 1;
                        } else {
                            self.phase = QuickPhase::SwappingWithPivot;
                        }
                    } else {
                        // Element is less than or equal to pivot, swap with left
                        self.phase = QuickPhase::SwappingElements;
                    }
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::SwappingElements => {
                if self.left <= self.right {
                    self.states[self.left] = SelectionState::Swapping;
                    self.states[self.right] = SelectionState::Swapping;

                    // Swap elements at left and right pointers
                    self.array.swap(self.left, self.right);
                    self.swaps += 1;

                    // Move pointers
                    self.left += 1;
                    if self.right > 0 {
                        self.right -= 1;
                    }

                    self.phase = QuickPhase::PartitioningLeft;
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::SwappingWithPivot => {
                self.states[self.pivot_index] = SelectionState::Swapping;
                self.states[self.left] = SelectionState::Swapping;

                // Swap pivot with left pointer (final position)
                if self.left != self.pivot_index {
                    self.array.swap(self.pivot_index, self.left);
                    self.swaps += 1;
                }

                let pivot_final_pos = self.left;
                self.states[pivot_final_pos] = SelectionState::Sorted;

                // Push new subarrays to stack (larger subarray first)
                if pivot_final_pos + 1 <= self.high {
                    self.stack.push((pivot_final_pos + 1, self.high));
                }
                if self.low < pivot_final_pos && pivot_final_pos > 0 {
                    self.stack.push((self.low, pivot_final_pos - 1));
                }

                self.phase = QuickPhase::ChoosingPivot;
                true
            },
            QuickPhase::DonePartition => {
                self.phase = QuickPhase::ChoosingPivot;
                true
            },
        }
    }

    /// Resets the visualization to its initial state
    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.stack = Vec::new();
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
        self.pivot_index = 0;
        self.left = 0;
        self.right = 0;

        if len <= 1 {
            self.completed = true;
            self.states = vec![SelectionState::Sorted; len];
            self.phase = QuickPhase::DonePartition;
        } else {
            self.stack.push((0, len - 1));
            self.low = 0;
            self.high = len - 1;
            self.phase = QuickPhase::ChoosingPivot;
        }
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
            let sorted_count = self.states.iter().filter(|&&s| s == SelectionState::Sorted).count() as f64;
            (sorted_count / self.array.len() as f64 * 100.0).min(100.0)
        }
    }
}

/// Entry point for the quick sort visualization
pub fn quick_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = QuickSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
