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

/// Represents the different phases of the radix sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum RadixPhase {
    StartingDigit,      // Starting a new digit pass
    CountingOccurrences, // Counting occurrences of each digit
    CalculatingPositions, // Calculating positions for each digit
    PlacingElements,    // Placing elements in their correct positions
    CopyingBack,        // Copying elements back to the main array
    NextDigit,          // Moving to the next digit
    Done,               // Sorting is complete
}

/// Visualizes the radix sort algorithm step-by-step with interactive controls
pub struct RadixSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    temp_array: Vec<u32>,      // Temporary array used during sorting
    states: Vec<SelectionState>, // Visual state of each element
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of moves made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete

    // Radix Sort specific fields
    current_digit: u32,       // Current digit position being processed (1=ones, 2=tens, etc.)
    max_digits: u32,          // Maximum number of digits in any number
    radix: u32,               // Base (usually 10 for decimal numbers)
    count: Vec<u32>,          // Count array for digits 0-9
    original_count: Vec<u32>, // Store original counts for visualization
    current_index: usize,     // Current index being processed
    current_element: u32,     // Current element being processed
    current_digit_value: u32, // Current digit value being processed
    phase: RadixPhase,         // Current phase of the radix sort algorithm
    buckets: Vec<Vec<u32>>,   // Visual buckets for demonstration
}

impl RadixSortVisualizer {
    /// Creates a new RadixSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();

        // Find maximum number to determine number of digits
        let max_num = *array.iter().max().unwrap_or(&0);
        let max_digits = if max_num == 0 { 1 } else { Self::count_digits(max_num) };

        // If the array has 0 or 1 elements, it's already sorted
        if len <= 1 {
            Self {
                original_array: array.clone(),
                array,
                temp_array: vec![0; len],
                states: vec![SelectionState::Sorted; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(600),
                completed: true,
                current_digit: 0,
                max_digits,
                radix: 10,
                count: vec![0; 10],
                original_count: vec![0; 10],
                current_index: 0,
                current_element: 0,
                current_digit_value: 0,
                phase: RadixPhase::Done,
                buckets: vec![Vec::new(); 10],
            }
        } else {
            // Initialize for sorting
            Self {
                original_array: array.clone(),
                array,
                temp_array: vec![0; len],
                states: vec![SelectionState::Normal; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(600),
                completed: false,
                current_digit: 1, // Start with ones place
                max_digits,
                radix: 10,
                count: vec![0; 10],
                original_count: vec![0; 10],
                current_index: 0,
                current_element: 0,
                current_digit_value: 0,
                phase: RadixPhase::StartingDigit,
                buckets: vec![Vec::new(); 10],
            }
        }
    }

    /// Counts the number of digits in a number
    fn count_digits(mut num: u32) -> u32 {
        if num == 0 { return 1; }
        let mut digits = 0;
        while num > 0 {
            digits += 1;
            num /= 10;
        }
        digits
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
        let title = "TOGISOFT RADIX SORT VISUALIZER";
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

        // --- Digit Place Info and Buckets ---
        self.draw_digit_info(stdout, width, height);

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
        let total_width_needed = array_len * bar_width + (array_len.saturating_sub(1)) * spacing;
        let start_x = ((width as usize).saturating_sub(total_width_needed)) / 2;

        // Draw bars
        let max_bar_height = (height as usize).saturating_sub(18).min(15);
        for (i, &value) in self.array.iter().enumerate() {
            let bar_height = if max_value > 0.0 {
                ((value as f64 / max_value) * max_bar_height as f64) as usize + 1
            } else {
                1
            };
            let x = start_x + i * (bar_width + spacing);

            // Get current digit of this number
            let current_digit_of_element = self.get_digit(value, self.current_digit);

            // Choose color based on state and current digit
            let (fg_color, bg_color) = match self.states[i] {
                SelectionState::Normal => {
                    // Color based on current digit value
                    let digit_color = Self::get_digit_color(current_digit_of_element);
                    (digit_color, Color::Reset)
                },
                SelectionState::Sorted => (Color::Green, Color::DarkGreen),
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow), // Current element being processed
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta), // Being counted
                SelectionState::Selected => (Color::White, Color::DarkBlue), // Being placed
                SelectionState::Swapping => (Color::Red, Color::DarkRed), // Being moved
                SelectionState::PartitionLeft => (Color::Blue, Color::DarkBlue),
                SelectionState::PartitionRight => (Color::AnsiValue(208), Color::DarkBlue),
            };

            // Draw the bar from bottom up
            for h in 0..bar_height {
                let y = array_start_y + max_bar_height - h;
                if y < height as usize {
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
            }

            // Draw value below bar
            let value_str = value.to_string();
            let value_x = x + (bar_width.saturating_sub(value_str.len())) / 2;
            let value_y = array_start_y + max_bar_height + 1;
            if value_y < height as usize {
                stdout.queue(MoveTo(value_x as u16, value_y as u16)).unwrap();
                stdout.queue(SetForegroundColor(Color::White)).unwrap();
                stdout.queue(Print(value_str)).unwrap();
                stdout.queue(ResetColor).unwrap();
            }

            // Highlight current digit in the number
            let digit_str = current_digit_of_element.to_string();
            let digit_x = x + (bar_width.saturating_sub(digit_str.len())) / 2;
            let digit_y = array_start_y + max_bar_height + 2;
            if digit_y < height as usize {
                stdout.queue(MoveTo(digit_x as u16, digit_y as u16)).unwrap();
                stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
                stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
                stdout.queue(SetBackgroundColor(Color::DarkRed)).unwrap();
                stdout.queue(Print(digit_str)).unwrap();
                stdout.queue(ResetColor).unwrap();
            }

            // Draw index below
            let index_str = i.to_string();
            let index_x = x + (bar_width.saturating_sub(index_str.len())) / 2;
            let index_y = array_start_y + max_bar_height + 3;
            if index_y < height as usize {
                stdout.queue(MoveTo(index_x as u16, index_y as u16)).unwrap();
                stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                stdout.queue(Print(index_str)).unwrap();
                stdout.queue(ResetColor).unwrap();
            }
        }

        // --- Legend ---
        let legend_y = array_start_y + max_bar_height + 5;
        if legend_y + 1 < height as usize {
            let legend_items = [
                ("Normal", Color::Cyan),
                ("Being Counted", Color::Magenta),
                ("Being Placed", Color::White),
                ("Being Moved", Color::Red),
                ("Sorted", Color::Green),
            ];
            let legend_start_x = (width as usize - 80) / 2;
            for (i, (label, color)) in legend_items.iter().enumerate() {
                let x = legend_start_x + i * 16;
                let y = legend_y;
                if y < height as usize && x + 14 < width as usize {
                    stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                    stdout.queue(SetForegroundColor(*color)).unwrap();
                    stdout.queue(Print("██")).unwrap();
                    stdout.queue(ResetColor).unwrap();
                    stdout.queue(Print(format!(" {}", label))).unwrap();
                }
            }
        }
    }

    /// Returns a color for a digit (0-9) for visualization
    fn get_digit_color(digit: u32) -> Color {
        match digit {
            0 => Color::AnsiValue(240), // Dark grey
            1 => Color::AnsiValue(196), // Red
            2 => Color::AnsiValue(202), // Orange
            3 => Color::AnsiValue(226), // Yellow
            4 => Color::AnsiValue(46),  // Green
            5 => Color::AnsiValue(51),  // Cyan
            6 => Color::AnsiValue(21),  // Blue
            7 => Color::AnsiValue(93),  // Purple
            8 => Color::AnsiValue(201), // Magenta
            9 => Color::AnsiValue(255), // White
            _ => Color::Cyan,
        }
    }

    /// Draws information about the current digit place and buckets
    fn draw_digit_info(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let digit_y = height.saturating_sub(14);
        if digit_y >= height {
            return;
        }

        // Current digit place info
        let digit_place_name = match self.current_digit {
            1 => "Ones Place",
            2 => "Tens Place",
            3 => "Hundreds Place",
            4 => "Thousands Place",
            5 => "Ten Thousands Place",
            _ => "Higher Digit Place",
        };

        let digit_info = format!("Current Pass: {} | Element: {} | Digit Value: {} | Index: {}",
                                 digit_place_name,
                                 if self.current_index < self.array.len() { self.array[self.current_index] } else { 0 },
                                 self.current_digit_value,
                                 self.current_index);

        let info_x = (width.saturating_sub(digit_info.len() as u16)) / 2;
        stdout.queue(MoveTo(info_x, digit_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(digit_info)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // Buckets visualization
        let bucket_y = digit_y + 2;
        if bucket_y + 8 < height {
            let bucket_title = format!("Buckets (by {} digit):", digit_place_name.to_lowercase());
            stdout.queue(MoveTo(5, bucket_y)).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
            stdout.queue(Print(bucket_title)).unwrap();
            stdout.queue(ResetColor).unwrap();

            // Draw buckets 0-9
            let bucket_width = ((width as usize).saturating_sub(10)) / 10;
            for digit in 0..10 {
                let x = 5 + digit * bucket_width;
                let y = bucket_y + 1;
                if y >= height {
                    break;
                }

                // Bucket header
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
                let digit_color = Self::get_digit_color(digit as u32);
                stdout.queue(SetForegroundColor(digit_color)).unwrap();
                stdout.queue(Print(format!("[{}]", digit))).unwrap();
                stdout.queue(ResetColor).unwrap();

                // Bucket contents
                let bucket_contents = if digit < self.buckets.len() {
                    &self.buckets[digit]
                } else {
                    &Vec::new()
                };

                for (i, &value) in bucket_contents.iter().enumerate() {
                    let content_y = y + 2 + i as u16;
                    if content_y >= height {
                        break;
                    }

                    if i < 6 { // Limit display
                        stdout.queue(MoveTo(x as u16, content_y as u16)).unwrap();
                        stdout.queue(SetForegroundColor(digit_color)).unwrap();
                        stdout.queue(Print(format!("{}", value))).unwrap();
                        stdout.queue(ResetColor).unwrap();
                    } else if i == 6 {
                        stdout.queue(MoveTo(x as u16, content_y as u16)).unwrap();
                        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
                        stdout.queue(Print("...")).unwrap();
                        stdout.queue(ResetColor).unwrap();
                        break;
                    }
                }
            }
        }

        // Count array visualization
        let count_y = height.saturating_sub(6);
        if count_y < height {
            stdout.queue(MoveTo(5, count_y)).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
            stdout.queue(Print("Count Array: ")).unwrap();
            stdout.queue(ResetColor).unwrap();

            for (digit, &count) in self.count.iter().enumerate() {
                let digit_color = Self::get_digit_color(digit as u32);
                stdout.queue(SetForegroundColor(digit_color)).unwrap();
                stdout.queue(Print(format!("[{}:{:2}] ", digit, count))).unwrap();
                stdout.queue(ResetColor).unwrap();
            }
        }
    }

    /// Draws statistics such as comparisons, moves, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(11);
        if stats_y >= height {
            return;
        }

        let phase_str = match self.phase {
            RadixPhase::StartingDigit => "Starting Digit",
            RadixPhase::CountingOccurrences => "Counting",
            RadixPhase::CalculatingPositions => "Calculating Positions",
            RadixPhase::PlacingElements => "Placing Elements",
            RadixPhase::CopyingBack => "Copying Back",
            RadixPhase::NextDigit => "Next Digit",
            RadixPhase::Done => "Done",
        };

        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Moves: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let x = 5 + (i % 3) * 30;
            let y = stats_y + (i / 3) as u16;
            if y < height {
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
                stdout.queue(Print(stat)).unwrap();
                stdout.queue(ResetColor).unwrap();
            }
        }
    }

    /// Draws the control instructions and current status
    fn draw_controls(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let controls_y = height.saturating_sub(4);
        if controls_y >= height {
            return;
        }

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
        let controls_line_y = controls_y + 1;
        if controls_line_y < height {
            stdout.queue(MoveTo(controls_x, controls_line_y)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(controls)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    /// Draws information about the current operation
    fn draw_current_operation(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let op_y = height.saturating_sub(9);
        if op_y >= height {
            return;
        }

        if self.completed {
            let message = "✓ Array is now sorted using Radix Sort!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, op_y)).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            let operation = match self.phase {
                RadixPhase::StartingDigit => {
                    let place_name = match self.current_digit {
                        1 => "ones place",
                        2 => "tens place",
                        3 => "hundreds place",
                        4 => "thousands place",
                        _ => "digit place",
                    };
                    format!("Starting sorting pass for {} (digit position {})", place_name, self.current_digit)
                },
                RadixPhase::CountingOccurrences => {
                    if self.current_index < self.array.len() {
                        let place_name = match self.current_digit {
                            1 => "ones",
                            2 => "tens",
                            3 => "hundreds",
                            4 => "thousands",
                            _ => "current",
                        };
                        format!("Examining {} digit of {} (element {}) - found digit {}",
                                place_name, self.array[self.current_index], self.current_index, self.current_digit_value)
                    } else {
                        "Finished counting all digit occurrences".to_string()
                    }
                },
                RadixPhase::CalculatingPositions => {
                    "Converting digit counts to final positions in sorted array".to_string()
                },
                RadixPhase::PlacingElements => {
                    if self.current_index < self.array.len() {
                        let place_name = match self.current_digit {
                            1 => "ones",
                            2 => "tens",
                            3 => "hundreds",
                            4 => "thousands",
                            _ => "current",
                        };
                        format!("Placing {} (has {} digit {}) into correct sorted position",
                                self.current_element, place_name, self.current_digit_value)
                    } else {
                        "Placing all elements into their sorted positions".to_string()
                    }
                },
                RadixPhase::CopyingBack => {
                    format!("Copying sorted element {} back to main array at position {}",
                            self.temp_array.get(self.current_index).unwrap_or(&0), self.current_index)
                },
                RadixPhase::NextDigit => {
                    if self.current_digit <= self.max_digits {
                        let prev_place = match self.current_digit - 1 {
                            1 => "ones",
                            2 => "tens",
                            3 => "hundreds",
                            _ => "previous",
                        };
                        let next_place = match self.current_digit {
                            2 => "tens",
                            3 => "hundreds",
                            4 => "thousands",
                            _ => "next",
                        };
                        format!("Completed {} place sorting, moving to {} place", prev_place, next_place)
                    } else {
                        "All digit places have been processed".to_string()
                    }
                },
                RadixPhase::Done => {
                    "Radix sort completed! All digit places have been sorted.".to_string()
                },
            };

            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, op_y)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    /// Gets the digit at a specific position in a number
    fn get_digit(&self, number: u32, digit_position: u32) -> u32 {
        if digit_position == 0 {
            return 0;
        }
        (number / (self.radix.pow(digit_position - 1))) % self.radix
    }

    /// Performs a single step of the radix sort algorithm
    /// Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset states except sorted
        for state in self.states.iter_mut() {
            match *state {
                SelectionState::Sorted => {}
                _ => *state = SelectionState::Normal,
            }
        }

        match self.phase {
            RadixPhase::StartingDigit => {
                if self.current_digit <= self.max_digits {
                    // Initialize for this digit
                    self.count.fill(0);
                    self.original_count.fill(0);
                    self.buckets.iter_mut().for_each(|bucket| bucket.clear());
                    self.current_index = 0;
                    self.phase = RadixPhase::CountingOccurrences;
                    true
                } else {
                    self.phase = RadixPhase::Done;
                    false
                }
            },
            RadixPhase::CountingOccurrences => {
                if self.current_index < self.array.len() {
                    let digit = self.get_digit(self.array[self.current_index], self.current_digit);
                    self.current_digit_value = digit;
                    // Highlight current element
                    self.states[self.current_index] = SelectionState::Comparing;
                    // Count this digit
                    if (digit as usize) < self.count.len() {
                        self.count[digit as usize] += 1;
                        // Add to visual bucket
                        if (digit as usize) < self.buckets.len() {
                            self.buckets[digit as usize].push(self.array[self.current_index]);
                        }
                    }
                    self.comparisons += 1;
                    self.current_index += 1;
                    true
                } else {
                    // Store original counts for visualization
                    self.original_count = self.count.clone();
                    self.phase = RadixPhase::CalculatingPositions;
                    true
                }
            },
            RadixPhase::CalculatingPositions => {
                // Convert counts to positions (cumulative sum)
                for i in 1..self.count.len() {
                    self.count[i] += self.count[i - 1];
                }
                // Start from the end for stable sorting
                self.current_index = self.array.len();
                self.phase = RadixPhase::PlacingElements;
                true
            },
            RadixPhase::PlacingElements => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                    let element = self.array[self.current_index];
                    let digit = self.get_digit(element, self.current_digit);
                    self.current_element = element;
                    self.current_digit_value = digit;
                    // Highlight current element being placed
                    self.states[self.current_index] = SelectionState::Selected;
                    // Place element in temp array
                    if (digit as usize) < self.count.len() && self.count[digit as usize] > 0 {
                        self.count[digit as usize] -= 1;
                        let pos = self.count[digit as usize] as usize;
                        if pos < self.temp_array.len() {
                            self.temp_array[pos] = element;
                        }
                    }
                    self.swaps += 1;
                    true
                } else {
                    self.phase = RadixPhase::CopyingBack;
                    self.current_index = 0;
                    true
                }
            },
            RadixPhase::CopyingBack => {
                if self.current_index < self.array.len() {
                    // Copy back from temp array
                    self.states[self.current_index] = SelectionState::Swapping;
                    self.array[self.current_index] = self.temp_array[self.current_index];
                    self.current_index += 1;
                    true
                } else {
                    self.phase = RadixPhase::NextDigit;
                    true
                }
            },
            RadixPhase::NextDigit => {
                self.current_digit += 1;
                if self.current_digit <= self.max_digits {
                    self.phase = RadixPhase::StartingDigit;
                    true
                } else {
                    self.phase = RadixPhase::Done;
                    false
                }
            },
            RadixPhase::Done => false,
        }
    }

    /// Resets the visualization to its initial state
    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.temp_array = vec![0; len];
        self.states = vec![SelectionState::Normal; len];
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
        self.current_index = 0;
        self.current_element = 0;
        self.current_digit_value = 0;

        // Recalculate max digits
        let max_num = *self.array.iter().max().unwrap_or(&0);
        self.max_digits = if max_num == 0 { 1 } else { Self::count_digits(max_num) };
        self.count.fill(0);
        self.original_count.fill(0);
        self.buckets.iter_mut().for_each(|bucket| bucket.clear());

        if len <= 1 {
            self.completed = true;
            self.states = vec![SelectionState::Sorted; len];
            self.phase = RadixPhase::Done;
            self.current_digit = 0;
        } else {
            self.current_digit = 1;
            self.phase = RadixPhase::StartingDigit;
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
        if self.array.len() <= 1 || self.max_digits == 0 {
            100.0
        } else {
            let completed_digits = if self.current_digit > self.max_digits {
                self.max_digits as f64
            } else {
                (self.current_digit.saturating_sub(1)) as f64
            };

            let current_phase_progress = match self.phase {
                RadixPhase::StartingDigit => 0.0,
                RadixPhase::CountingOccurrences => {
                    if self.array.len() > 0 {
                        (self.current_index as f64 / self.array.len() as f64) * 0.2
                    } else {
                        0.2
                    }
                },
                RadixPhase::CalculatingPositions => 0.2,
                RadixPhase::PlacingElements => {
                    if self.array.len() > 0 {
                        0.2 + ((self.array.len() - self.current_index) as f64 / self.array.len() as f64) * 0.4
                    } else {
                        0.6
                    }
                },
                RadixPhase::CopyingBack => {
                    if self.array.len() > 0 {
                        0.6 + (self.current_index as f64 / self.array.len() as f64) * 0.2
                    } else {
                        0.8
                    }
                },
                RadixPhase::NextDigit => 1.0,
                RadixPhase::Done => 1.0,
            };

            let total_progress = if self.max_digits > 0 {
                (completed_digits + current_phase_progress) / self.max_digits as f64 * 100.0
            } else {
                100.0
            };

            total_progress.min(100.0)
        }
    }
}

/// Entry point for the radix sort visualization
pub fn radix_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = RadixSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
