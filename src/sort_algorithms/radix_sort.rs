use crate::common::array_manager::ArrayData;
use crate::common::base_visualizer::{SortVisualizer, VisualizerState};
use crate::common::common_visualizer::{show_intro_screen, show_question_feedback, VisualizerDrawer};
use crate::common::enums::{SelectionState, TeachingQuestion};
use crate::common::helper::{cleanup_terminal, randomize_questions};
use crate::common::settings::Settings;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::Color,
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen},
    ExecutableCommand,
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
    intro_text: String,        // Dynamic intro text

    // Radix Sort specific fields
    current_digit: u32,       // Current digit position being processed (1=ones, 2=tens, etc.)
    max_digits: u32,          // Maximum number of digits in any number
    radix: u32,               // Base (usually 10 for decimal numbers)
    count: Vec<u32>,          // Count array for digits 0-9
    current_index: usize,     // Current index being processed
    current_element: u32,     // Current element being processed
    current_digit_value: u32, // Current digit value being processed
    phase: RadixPhase,        // Current phase of the radix sort algorithm
    state: VisualizerState,   // Common visualization state
}

impl RadixSortVisualizer {
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

    /// Gets the digit at a specific position in a number
    fn get_digit(&self, number: u32, digit_position: u32) -> u32 {
        if digit_position == 0 {
            return 0;
        }
        (number / self.radix.pow(digit_position - 1)) % self.radix
    }

    /// Creates a new RadixSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What makes Radix Sort efficient for integers?".to_string(),
                options: vec![
                    "It sorts by individual digits, avoiding comparisons".to_string(),
                    "It uses divide and conquer".to_string(),
                    "It selects a pivot".to_string(),
                ],
                correct_index: 0,
                explanation: "Radix Sort processes numbers digit by digit, making it non-comparative and efficient for fixed-length keys.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Radix Sort?".to_string(),
                options: vec![
                    "O(d(n + k)) where d is digits, k is radix".to_string(),
                    "O(n log n)".to_string(),
                    "O(n^2)".to_string(),
                ],
                correct_index: 0,
                explanation: "Time complexity is O(d(n + k)), linear in the number of digits and array size.".to_string(),
            },
            TeachingQuestion {
                text: "Why is Radix Sort stable?".to_string(),
                options: vec![
                    "It processes digits from least to most significant".to_string(),
                    "It uses a temporary array".to_string(),
                    "It compares elements".to_string(),
                ],
                correct_index: 0,
                explanation: "Processing from LSD to MSD ensures stability, preserving relative order of equal keys.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let max_num = *array.iter().max().unwrap_or(&0);
        let max_digits = if max_num == 0 { 1 } else { Self::count_digits(max_num) };

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Radix Sort?\n\n\
             Radix Sort is a non-comparative integer sorting algorithm that sorts data by grouping keys by individual digits.\n\
             It processes digits from least to most significant, using stable counting sort for each digit.\n\n\
             Advantages: Linear time O(d(n+k)) for integers.\n\
             Disadvantages: Only for integers or fixed-length keys.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each digit pass.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            temp_array: vec![0; len],
            states: vec![SelectionState::Normal; len],
            intro_text,
            current_digit: 1,
            max_digits,
            radix: 10,
            count: vec![0; 10],
            current_index: 0,
            current_element: 0,
            current_digit_value: 0,
            phase: RadixPhase::StartingDigit,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("RadixSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
            this.mark_all_sorted();
            this.phase = RadixPhase::Done;
            this.current_digit = 0;
        }

        this
    }

    /// Main loop: handles rendering, input, and stepping through the sort
    pub fn run_visualization(&mut self) {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(EnterAlternateScreen).unwrap();

        show_intro_screen(self.get_intro_text());

        loop {
            self.draw(&mut stdout);

            if poll(Duration::from_millis(50)).unwrap_or(false) {
                match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        // Handle question
                        if let Some(q_index) = self.state.awaiting_question {
                            match key_event.code {
                                KeyCode::Char('1') => self.handle_question_answer(q_index, 0),
                                KeyCode::Char('2') => self.handle_question_answer(q_index, 1),
                                KeyCode::Char('3') => self.handle_question_answer(q_index, 2),
                                _ => continue,
                            }
                            continue;
                        }

                        match key_event.code {
                            KeyCode::Char(' ') => {
                                if self.state.completed {
                                    self.reset();
                                } else {
                                    self.state.toggle_play_pause();
                                }
                            },
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                self.reset();
                            },
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                if !self.state.completed && !self.state.is_running {
                                    if !self.step() {
                                        self.state.mark_completed();
                                        self.mark_all_sorted();
                                    }
                                }
                            },
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                self.state.toggle_teaching_mode();
                                self.intro_text = format!(
                                    "What is Radix Sort?\n\n\
                                     Radix Sort is a non-comparative integer sorting algorithm that sorts data by grouping keys by individual digits.\n\
                                     It processes digits from least to most significant, using stable counting sort for each digit.\n\n\
                                     Advantages: Linear time O(d(n+k)) for integers.\n\
                                     Disadvantages: Only for integers or fixed-length keys.\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each digit pass.\n\n\
                                     Press any key to continue...",
                                    if self.state.teaching_mode { "ON" } else { "OFF" }
                                );
                                let mut settings = Settings::load();
                                settings.teaching_mode = self.state.teaching_mode;
                                settings.save();
                            },
                            KeyCode::Char('+') => {
                                self.state.increase_speed(50);
                                let mut settings = Settings::load();
                                settings.speed = self.state.speed.as_millis() as u64;
                                settings.save();
                            },
                            KeyCode::Char('-') => {
                                self.state.decrease_speed(2000);
                                let mut settings = Settings::load();
                                settings.speed = self.state.speed.as_millis() as u64;
                                settings.save();
                            },
                            KeyCode::Esc => {
                                let mut settings = Settings::load();
                                settings.last_visualizer = Some("RadixSort".to_string());
                                settings.save();
                                cleanup_terminal();
                                return;
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Auto step
            if self.state.is_running && !self.state.is_paused && !self.state.completed
                && self.state.awaiting_question.is_none() {
                std::thread::sleep(self.state.speed);
                if !self.step() {
                    self.state.mark_completed();
                    self.mark_all_sorted();
                }
            }
        }
    }

    fn handle_question_answer(&mut self, q_index: usize, answer: usize) {
        if let Some(question) = self.state.questions.get(q_index) {
            let correct = answer == question.correct_index;
            show_question_feedback(correct, question, answer);
            self.state.clear_question();
        }
    }

    fn draw(&mut self, stdout: &mut std::io::Stdout) {
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // Title
        VisualizerDrawer::draw_title(stdout, self.get_title());

        // Array
        VisualizerDrawer::draw_array_bars(stdout, &self.array, &self.states, width, height, 5);

        // Legend
        VisualizerDrawer::draw_legend(stdout, &self.get_legend_items(), width, height, 5);

        // Statistics
        let stats = self.get_statistics_strings();
        VisualizerDrawer::draw_statistics(stdout, &stats, width, height);

        // Controls
        VisualizerDrawer::draw_controls(stdout, self.get_status(), self.get_controls_text(), width, height);

        // Current operation
        if self.state.awaiting_question.is_none() {
            let operation = self.get_current_operation();
            let color = if self.state.completed { Color::Green } else { Color::White };
            VisualizerDrawer::draw_operation_info(stdout, &operation, width, height, color);
        }

        // Question
        if let Some(q_index) = self.state.awaiting_question {
            if let Some(question) = self.state.questions.get(q_index) {
                VisualizerDrawer::draw_question(stdout, question, width, height);
            }
        }

        stdout.flush().unwrap();
    }
}

impl SortVisualizer for RadixSortVisualizer {
    fn get_array(&self) -> &[u32] { &self.array }
    fn get_original_array(&self) -> &[u32] { &self.original_array }
    fn get_states(&self) -> &[SelectionState] { &self.states }
    fn get_comparisons(&self) -> u32 { self.state.comparisons }
    fn get_swaps(&self) -> u32 { self.state.swaps }
    fn get_speed(&self) -> Duration { self.state.speed }
    fn is_running(&self) -> bool { self.state.is_running }
    fn is_paused(&self) -> bool { self.state.is_paused }
    fn is_completed(&self) -> bool { self.state.completed }
    fn is_teaching_mode(&self) -> bool { self.state.teaching_mode }
    fn get_awaiting_question(&self) -> Option<usize> { self.state.awaiting_question }
    fn get_questions(&self) -> &[TeachingQuestion] { &self.state.questions }

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

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
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
                    }
                    self.state.comparisons += 1;
                    self.current_index += 1;
                    true
                } else {
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
                    self.state.swaps += 1;
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
                    self.state.swaps += 1;
                    true
                } else {
                    // Teaching: Ask question after each pass
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = (self.current_digit as usize - 1) % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
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

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.temp_array = vec![0; len];
        self.states = vec![SelectionState::Normal; len];
        self.current_index = 0;
        self.current_element = 0;
        self.current_digit_value = 0;

        // Recalculate max digits
        let max_num = *self.array.iter().max().unwrap_or(&0);
        self.max_digits = if max_num == 0 { 1 } else { Self::count_digits(max_num) };
        self.count.fill(0);

        self.current_digit = 1;
        self.phase = RadixPhase::StartingDigit;
        self.state.reset_state();
        self.intro_text = format!(
            "What is Radix Sort?\n\n\
             Radix Sort is a non-comparative integer sorting algorithm that sorts data by grouping keys by individual digits.\n\
             It processes digits from least to most significant, using stable counting sort for each digit.\n\n\
             Advantages: Linear time O(d(n+k)) for integers.\n\
             Disadvantages: Only for integers or fixed-length keys.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each digit pass.\n\n\
             Press any key to continue...",
            if self.state.teaching_mode { "ON" } else { "OFF" }
        );
        if len <= 1 {
            self.state.mark_completed();
            self.mark_all_sorted();
            self.phase = RadixPhase::Done;
            self.current_digit = 0;
        }
    }

    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT RADIX SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Being Counted", Color::Magenta),
            ("Being Placed", Color::White),
            ("Being Moved", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            RadixPhase::StartingDigit => "Starting Digit",
            RadixPhase::CountingOccurrences => "Counting",
            RadixPhase::CalculatingPositions => "Calculating Positions",
            RadixPhase::PlacingElements => "Placing Elements",
            RadixPhase::CopyingBack => "Copying Back",
            RadixPhase::NextDigit => "Next Digit",
            RadixPhase::Done => "Done",
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Moves: {}", self.state.swaps),
            format!("Current Digit: {}", self.current_digit),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Radix Sort! Congratulations!".to_string()
        } else {
            match self.phase {
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
            }
        }
    }

    fn get_status(&self) -> &str {
        if self.state.awaiting_question.is_some() {
            "WAITING FOR QUESTION"
        } else if self.state.completed {
            "COMPLETED!"
        } else if self.state.is_running && !self.state.is_paused {
            "RUNNING..."
        } else if self.state.is_paused {
            "PAUSED"
        } else {
            "READY"
        }
    }

    fn get_controls_text(&self) -> &str {
        if self.state.awaiting_question.is_some() {
            "1,2,3: Answer | ESC: Exit"
        } else if self.state.completed {
            "SPACE: Restart | R: Reset | T: Teaching Toggle | ESC: Exit"
        } else {
            "SPACE: Start/Pause | S: Step | R: Reset | T: Teaching | +/-: Speed | ESC: Exit"
        }
    }
}

/// Entry point for the radix sort visualization
pub fn radix_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = RadixSortVisualizer::new(array_data);
    visualizer.run_visualization();
}