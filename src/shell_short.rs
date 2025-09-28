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

/// Represents the different phases of the shell sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum ShellPhase {
    StartingGap,        // Starting a new gap size
    InsertionSorting,   // Performing insertion sort with current gap
    ComparingElements,  // Comparing elements during insertion sort
    ShiftingElement,    // Shifting an element to make space
    InsertingElement,   // Inserting an element in its correct position
    GapComplete,        // Completed sorting with current gap
    Done,               // Sorting is complete
}

/// Visualizes the shell sort algorithm step-by-step with interactive controls
pub struct ShellSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of shifts made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete

    // Shell Sort specific fields
    gap: usize,                // Current gap size
    current_group: usize,      // Current group being processed
    current_index: usize,      // Current index being processed
    insertion_index: usize,    // Index where element will be inserted
    comparing_index: usize,    // Index of element being compared
    key: u32,                  // Current element being inserted
    phase: ShellPhase,         // Current phase of the shell sort algorithm
    gap_sequence: Vec<usize>,  // Sequence of gap sizes (Knuth sequence)
    gap_sequence_index: usize, // Index of current gap in the sequence
}

impl ShellSortVisualizer {
    /// Creates a new ShellSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();

        // Generate Knuth gap sequence: h = 3h + 1
        let mut gap_sequence = Vec::new();
        let mut gap = 1;
        while gap < len {
            gap_sequence.push(gap);
            gap = gap * 3 + 1;
        }
        gap_sequence.reverse(); // Start with largest gap

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
                speed: Duration::from_millis(600),
                completed: true,
                gap: 1,
                current_group: 0,
                current_index: 0,
                insertion_index: 0,
                comparing_index: 0,
                key: 0,
                phase: ShellPhase::Done,
                gap_sequence,
                gap_sequence_index: 0,
            }
        } else {
            // Initialize with the first gap from the sequence
            let initial_gap = if gap_sequence.is_empty() { 1 } else { gap_sequence[0] };

            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Normal; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(600),
                completed: false,
                gap: initial_gap,
                current_group: 0,
                current_index: initial_gap,
                insertion_index: 0,
                comparing_index: 0,
                key: 0,
                phase: ShellPhase::StartingGap,
                gap_sequence,
                gap_sequence_index: 0,
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
        let title = "TOGISOFT SHELL SORT VISUALIZER";
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

        // --- Gap Visualization ---
        self.draw_gap_info(stdout, width, height);

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
        let max_bar_height = (height as usize).saturating_sub(18).min(15);
        for (i, &value) in self.array.iter().enumerate() {
            let bar_height = ((value as f64 / max_value) * max_bar_height as f64) as usize + 1;
            let x = start_x + i * (bar_width + spacing);

            // Choose color based on state and gap highlighting
            let (fg_color, bg_color) = match self.states[i] {
                SelectionState::Normal => {
                    // Highlight gap groups with subtle coloring
                    if self.gap > 1 && i % self.gap == self.current_group {
                        (Color::Cyan, Color::DarkBlue) // Current gap group
                    } else {
                        (Color::Cyan, Color::Reset)
                    }
                },
                SelectionState::Sorted => (Color::Green, Color::DarkGreen),
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow), // Key element
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta), // Being compared
                SelectionState::Selected => (Color::White, Color::DarkBlue), // Current position in insertion
                SelectionState::Swapping => (Color::Red, Color::DarkRed), // Being shifted
                SelectionState::PartitionLeft => (Color::Blue, Color::DarkBlue),
                SelectionState::PartitionRight => (Color::AnsiValue(208), Color::DarkBlue),
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

        // Draw gap connections for current group
        if self.gap > 1 && !self.completed {
            let group_color = match self.current_group {
                0 => Color::Blue,
                1 => Color::Green,
                2 => Color::Red,
                3 => Color::Yellow,
                _ => Color::Magenta,
            };

            // Draw lines connecting elements in same gap group
            for i in (self.current_group..array_len).step_by(self.gap) {
                if i + self.gap < array_len {
                    let x1 = start_x + i * (bar_width + spacing) + bar_width / 2;
                    let x2 = start_x + (i + self.gap) * (bar_width + spacing) + bar_width / 2;
                    let line_y = array_start_y + max_bar_height + 3;

                    stdout.queue(MoveTo(x1 as u16, line_y as u16)).unwrap();
                    stdout.queue(SetForegroundColor(group_color)).unwrap();

                    // Draw a line
                    for x in x1..=x2 {
                        stdout.queue(MoveTo(x as u16, line_y as u16)).unwrap();
                        if x == x1 || x == x2 {
                            stdout.queue(Print("•")).unwrap();
                        } else if (x - x1) % 3 == 0 {
                            stdout.queue(Print("-")).unwrap();
                        }
                    }
                    stdout.queue(ResetColor).unwrap();
                }
            }
        }

        // --- Legend ---
        let legend_y = array_start_y + max_bar_height + 5;
        let legend_items = [
            ("Normal", Color::Cyan),
            ("Gap Group", Color::DarkBlue),
            ("Key", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Position", Color::White),
            ("Shifting", Color::Red),
            ("Sorted", Color::Green),
        ];
        let legend_start_x = (width as usize - 100) / 2;
        for (i, (label, color)) in legend_items.iter().enumerate() {
            let x = legend_start_x + (i % 4) * 22;
            let y = legend_y + (i / 4);
            stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
            stdout.queue(SetForegroundColor(*color)).unwrap();
            stdout.queue(Print("██")).unwrap();
            stdout.queue(ResetColor).unwrap();
            stdout.queue(Print(format!(" {}", label))).unwrap();
        }
    }

    /// Draws information about the current gap and sequence
    fn draw_gap_info(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let gap_y = height.saturating_sub(14);

        // Current gap info
        let gap_info = format!("Gap: {} | Group: {} | Current Index: {} | Insertion Point: {}",
                               self.gap,
                               self.current_group,
                               self.current_index,
                               self.insertion_index);
        let info_x = (width.saturating_sub(gap_info.len() as u16)) / 2;
        stdout.queue(MoveTo(info_x, gap_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(gap_info)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // Gap sequence visualization
        let sequence_y = gap_y + 1;
        let mut sequence_str = "Gap Sequence: ".to_string();
        for (i, &gap) in self.gap_sequence.iter().enumerate() {
            if i == self.gap_sequence_index && !self.completed {
                sequence_str.push_str(&format!("[{}] ", gap));
            } else {
                sequence_str.push_str(&format!("{} ", gap));
            }
        }
        let seq_x = (width.saturating_sub(sequence_str.len() as u16)) / 2;
        stdout.queue(MoveTo(seq_x, sequence_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(sequence_str)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    /// Draws statistics such as comparisons, shifts, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(11);
        let phase_str = match self.phase {
            ShellPhase::StartingGap => "Starting Gap",
            ShellPhase::InsertionSorting => "Insertion Sort",
            ShellPhase::ComparingElements => "Comparing",
            ShellPhase::ShiftingElement => "Shifting",
            ShellPhase::InsertingElement => "Inserting",
            ShellPhase::GapComplete => "Gap Complete",
            ShellPhase::Done => "Done",
        };

        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Shifts: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let x = 5 + (i % 3) * 30;
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
        let op_y = height.saturating_sub(9);

        if self.completed {
            let message = "✓ Array is now sorted using Shell Sort!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, op_y)).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            let operation = match self.phase {
                ShellPhase::StartingGap => {
                    format!("Starting gap-{} insertion sort for group {}", self.gap, self.current_group)
                },
                ShellPhase::InsertionSorting => {
                    if self.current_index < self.array.len() {
                        format!("Gap-{} sort: processing element {} (value: {})",
                                self.gap, self.current_index, self.array[self.current_index])
                    } else {
                        format!("Gap-{} insertion sorting", self.gap)
                    }
                },
                ShellPhase::ComparingElements => {
                    if self.insertion_index < self.array.len() && self.comparing_index < self.array.len() {
                        format!("Comparing key {} with element at {} (value: {})",
                                self.key, self.comparing_index, self.array[self.comparing_index])
                    } else {
                        "Comparing elements...".to_string()
                    }
                },
                ShellPhase::ShiftingElement => {
                    if self.comparing_index < self.array.len() {
                        format!("Shifting element {} (value: {}) {} positions right",
                                self.comparing_index, self.array[self.comparing_index], self.gap)
                    } else {
                        "Shifting element...".to_string()
                    }
                },
                ShellPhase::InsertingElement => {
                    format!("Inserting key {} at position {}", self.key, self.insertion_index)
                },
                ShellPhase::GapComplete => {
                    format!("Gap-{} sorting completed, moving to next gap", self.gap)
                },
                ShellPhase::Done => {
                    "Shell sort completed!".to_string()
                },
            };

            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, op_y)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    /// Performs a single step of the shell sort algorithm
    /// Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset states to normal except sorted
        for state in self.states.iter_mut() {
            match *state {
                SelectionState::Sorted => {}
                _ => *state = SelectionState::Normal,
            }
        }

        match self.phase {
            ShellPhase::StartingGap => {
                if self.gap_sequence_index < self.gap_sequence.len() {
                    // Set the current gap
                    self.gap = self.gap_sequence[self.gap_sequence_index];
                    self.current_group = 0;
                    self.current_index = self.gap;
                    self.phase = ShellPhase::InsertionSorting;
                    true
                } else {
                    // All gaps processed, sorting complete
                    self.phase = ShellPhase::Done;
                    false
                }
            },
            ShellPhase::InsertionSorting => {
                if self.current_index < self.array.len() {
                    // Start insertion sort for current element
                    self.key = self.array[self.current_index];
                    self.insertion_index = self.current_index;
                    self.comparing_index = if self.current_index >= self.gap {
                        self.current_index - self.gap
                    } else {
                        0
                    };

                    // Highlight current element
                    self.states[self.current_index] = SelectionState::CurrentMin;

                    if self.insertion_index >= self.gap {
                        self.phase = ShellPhase::ComparingElements;
                    } else {
                        // No need to sort, move to next element
                        self.current_index += 1;

                        // Skip to next element in the same group
                        while self.current_index < self.array.len() &&
                            self.current_index % self.gap != self.current_group {
                            self.current_index += 1;
                        }

                        if self.current_index >= self.array.len() {
                            self.phase = ShellPhase::GapComplete;
                        }
                    }
                    true
                } else {
                    // All elements in this gap processed
                    self.phase = ShellPhase::GapComplete;
                    true
                }
            },
            ShellPhase::ComparingElements => {
                if self.insertion_index >= self.gap && self.comparing_index < self.array.len() {
                    // Highlight elements being compared
                    self.states[self.comparing_index] = SelectionState::Comparing;
                    self.comparisons += 1;

                    if self.array[self.comparing_index] > self.key {
                        // Need to shift this element
                        self.phase = ShellPhase::ShiftingElement;
                    } else {
                        // Found correct position
                        self.phase = ShellPhase::InsertingElement;
                    }
                } else {
                    // Found correct position
                    self.phase = ShellPhase::InsertingElement;
                }
                true
            },
            ShellPhase::ShiftingElement => {
                if self.comparing_index < self.array.len() && self.insertion_index < self.array.len() {
                    // Highlight elements being shifted
                    self.states[self.comparing_index] = SelectionState::Swapping;
                    self.states[self.insertion_index] = SelectionState::Swapping;

                    // Shift element to the right
                    self.array[self.insertion_index] = self.array[self.comparing_index];
                    self.swaps += 1;

                    self.insertion_index = self.comparing_index;

                    if self.comparing_index >= self.gap {
                        self.comparing_index -= self.gap;
                        self.phase = ShellPhase::ComparingElements;
                    } else {
                        // Reached the beginning of the group
                        self.phase = ShellPhase::InsertingElement;
                    }
                } else {
                    // Reached the beginning of the group
                    self.phase = ShellPhase::InsertingElement;
                }
                true
            },
            ShellPhase::InsertingElement => {
                if self.insertion_index < self.array.len() {
                    // Highlight position where element will be inserted
                    self.states[self.insertion_index] = SelectionState::Selected;

                    // Insert the key at its correct position
                    self.array[self.insertion_index] = self.key;

                    // Move to next element in the same gap group
                    self.current_index += 1;

                    // Skip to next element in the same group
                    while self.current_index < self.array.len() &&
                        self.current_index % self.gap != self.current_group {
                        self.current_index += 1;
                    }

                    if self.current_index >= self.array.len() {
                        // All elements in this group processed
                        self.phase = ShellPhase::GapComplete;
                    } else {
                        // Process next element
                        self.phase = ShellPhase::InsertionSorting;
                    }
                } else {
                    // All elements in this group processed
                    self.phase = ShellPhase::GapComplete;
                }
                true
            },
            ShellPhase::GapComplete => {
                // Move to next group
                self.current_group += 1;

                if self.current_group < self.gap {
                    // Start next group in the same gap
                    self.current_index = self.current_group + self.gap;
                    self.phase = ShellPhase::InsertionSorting;
                } else {
                    // Move to next gap
                    self.gap_sequence_index += 1;

                    if self.gap_sequence_index < self.gap_sequence.len() {
                        // Start with next gap
                        self.phase = ShellPhase::StartingGap;
                    } else {
                        // All gaps processed, sorting complete
                        self.phase = ShellPhase::Done;
                        return false;
                    }
                }
                true
            },
            ShellPhase::Done => false,
        }
    }

    /// Resets the visualization to its initial state
    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
        self.current_group = 0;
        self.current_index = 0;
        self.insertion_index = 0;
        self.comparing_index = 0;
        self.key = 0;
        self.gap_sequence_index = 0;

        if len <= 1 {
            self.completed = true;
            self.states = vec![SelectionState::Sorted; len];
            self.phase = ShellPhase::Done;
            self.gap = 1;
        } else {
            // Initialize with the first gap from the sequence
            self.gap = if self.gap_sequence.is_empty() { 1 } else { self.gap_sequence[0] };
            self.current_index = self.gap;
            self.phase = ShellPhase::StartingGap;
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
        if self.array.len() <= 1 || self.gap_sequence.is_empty() {
            100.0
        } else {
            let total_gaps = self.gap_sequence.len() as f64;
            let completed_gaps = self.gap_sequence_index as f64;
            let current_gap_progress = if self.gap > 0 {
                (self.current_group as f64) / (self.gap as f64)
            } else {
                0.0
            };

            ((completed_gaps + current_gap_progress) / total_gaps * 100.0).min(100.0)
        }
    }
}

/// Entry point for the shell sort visualization
pub fn shell_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = ShellSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
