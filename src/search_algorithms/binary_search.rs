use crate::common::array_manager::ArrayData;
use crate::common::base_visualizer::{SortVisualizer, VisualizerState};
use crate::common::common_visualizer::{show_intro_screen, show_question_feedback, VisualizerDrawer};
use crate::common::enums::{SelectionState, TeachingQuestion};
use crate::common::helper::{cleanup_terminal, randomize_questions};
use crate::common::settings::Settings;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind},
    style::{Color, Print},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen},
    cursor::{MoveTo, Show, Hide},
    ExecutableCommand,
};
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

/// Represents the different phases of the binary search algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum BinarySearchPhase {
    Searching,  // Searching through the array
    Found,      // Target found
    NotFound,   // Target not found
    Done,       // Search is complete
}

/// Visualizes the binary search algorithm step-by-step with interactive controls
pub struct BinarySearchVisualizer {
    array: Vec<u32>,           // Current state of the array (sorted)
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, found)

    // Binary Search specific fields
    target: u32,               // Target value to search for
    intro_text: String,        // Intro text with target
    low: usize,                // Current low index
    high: usize,               // Current high index
    mid: usize,                // Current mid index
    found_index: Option<usize>, // Index where target was found (if any)
    phase: BinarySearchPhase,  // Current phase of the binary search algorithm
    state: VisualizerState,    // Common visualization state
}

impl BinarySearchVisualizer {
    /// Prompts the user to input the target value for the search
    fn prompt_for_target(stdout: &mut Stdout, array: &[u32]) -> u32 {
        let mut input = String::new();
        let prompt = format!(
            "Enter the target value to search for (e.g., a number in the sorted array: {}): ",
            array.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")
        );

        // Clear the screen once and hide the cursor
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(Hide).unwrap();

        // Write the initial prompt
        stdout.execute(MoveTo(0, 0)).unwrap();
        stdout.execute(Print(&prompt)).unwrap();
        stdout.flush().unwrap();

        loop {
            // Read user input
            if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = read().unwrap() {
                match code {
                    KeyCode::Char(c) if c.is_digit(10) => {
                        input.push(c);
                        // Update the display with the current input
                        stdout.execute(MoveTo(0, 1)).unwrap();
                        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                        stdout.execute(Print(&input)).unwrap();
                        stdout.flush().unwrap();
                    }
                    KeyCode::Backspace => {
                        input.pop(); // Remove the last character
                        // Update the display with the current input
                        stdout.execute(MoveTo(0, 1)).unwrap();
                        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                        stdout.execute(Print(&input)).unwrap();
                        stdout.flush().unwrap();
                    }
                    KeyCode::Enter if !input.is_empty() => {
                        if let Ok(target) = input.parse::<u32>() {
                            // Show cursor and return to normal
                            stdout.execute(Show).unwrap();
                            return target;
                        } else {
                            // Show error message on the next line
                            stdout.execute(MoveTo(0, 1)).unwrap();
                            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                            stdout.execute(Print("Invalid input. Please enter a valid number.")).unwrap();
                            stdout.flush().unwrap();
                            input.clear();
                        }
                    }
                    KeyCode::Esc => {
                        // Show cursor and default to middle element
                        stdout.execute(Show).unwrap();
                        return if !array.is_empty() { array[array.len() / 2] } else { 0 };
                    }
                    _ => {}
                }
            }
        }
    }

    /// Creates a new BinarySearchVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let mut array = array_data.data.clone();
        let len = array.len();

        // Sort the array for binary search
        array.sort_unstable();

        // Enable raw mode and prompt for target
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        let target = Self::prompt_for_target(&mut stdout, &array);

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the key requirement for Binary Search?".to_string(),
                options: vec![
                    "The array must be sorted".to_string(),
                    "The array must be unsorted".to_string(),
                    "The array size must be prime".to_string(),
                ],
                correct_index: 0,
                explanation: "Binary Search requires the array to be sorted to efficiently divide the search space in half.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Binary Search?".to_string(),
                options: vec![
                    "O(log n)".to_string(),
                    "O(n)".to_string(),
                    "O(n^2)".to_string(),
                ],
                correct_index: 0,
                explanation: "Binary Search has O(log n) time complexity because it halves the search space with each comparison.".to_string(),
            },
            TeachingQuestion {
                text: "In Binary Search, what happens when the middle element is greater than the target?".to_string(),
                options: vec![
                    "Search the left half".to_string(),
                    "Search the right half".to_string(),
                    "Start over from the beginning".to_string(),
                ],
                correct_index: 0,
                explanation: "If the middle element is greater than the target, the target must be in the left half, so high is set to mid - 1.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!("What is Binary Search?\n\n\
         Binary Search is an efficient algorithm that finds the target in a sorted array by repeatedly dividing\n\
         the search interval in half. It starts by comparing the middle element with the target.\n\n\
         Target: {}\n\n\
         Advantages: O(log n) time complexity for sorted arrays.\n\
         Disadvantages: Requires the array to be sorted first.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after each comparison.\n\n\
         Press any key to continue...", target);

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            target,
            intro_text,
            low: 0,
            high: len.saturating_sub(1),
            mid: 0,
            found_index: None,
            phase: BinarySearchPhase::Searching,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("BinarySearch".to_string());
        settings.save();

        if len == 0 {
            this.state.mark_completed();
        }

        this
    }

    /// Main loop: handles rendering, input, and stepping through the search
    pub fn run_visualization(&mut self) {
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen).unwrap();

        show_intro_screen(&self.intro_text);

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
                                    }
                                }
                            },
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                self.state.toggle_teaching_mode();
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
                                settings.last_visualizer = Some("BinarySearch".to_string());
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

impl SortVisualizer for BinarySearchVisualizer {
    fn get_array(&self) -> &[u32] { &self.array }
    fn get_original_array(&self) -> &[u32] { &self.original_array }
    fn get_states(&self) -> &[SelectionState] { &self.states }
    fn get_comparisons(&self) -> u32 { self.state.comparisons }
    fn get_swaps(&self) -> u32 { self.state.swaps } // Not used for search
    fn get_speed(&self) -> Duration { self.state.speed }
    fn is_running(&self) -> bool { self.state.is_running }
    fn is_paused(&self) -> bool { self.state.is_paused }
    fn is_completed(&self) -> bool { self.state.completed }
    fn is_teaching_mode(&self) -> bool { self.state.teaching_mode }
    fn get_awaiting_question(&self) -> Option<usize> { self.state.awaiting_question }
    fn get_questions(&self) -> &[TeachingQuestion] { &self.state.questions }

    fn get_progress(&self) -> f64 {
        let n = self.array.len() as f64;
        if n == 0.0 { 100.0 } else {
            let max_comparisons = (n.log2() + 1.0).ceil();
            (self.state.comparisons as f64 / max_comparisons * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states except found
        for (i, state) in self.states.iter_mut().enumerate() {
            if self.found_index.is_some_and(|found| i == found) {
                *state = SelectionState::Sorted; // Reuse Sorted for Found
            } else if *state != SelectionState::Sorted {
                *state = SelectionState::Normal;
            }
        }

        let n = self.array.len();
        match self.phase {
            BinarySearchPhase::Searching => {
                if self.low > self.high {
                    self.phase = BinarySearchPhase::NotFound;
                    return false;
                }

                self.mid = self.low + (self.high - self.low) / 2;
                if self.mid >= n {
                    self.phase = BinarySearchPhase::NotFound;
                    return false;
                }

                // Highlight low, high, mid
                if self.low < n {
                    self.states[self.low] = SelectionState::Comparing;
                }
                if self.high < n {
                    self.states[self.high] = SelectionState::Comparing;
                }
                self.states[self.mid] = SelectionState::Comparing;
                self.state.comparisons += 1;

                if self.array[self.mid] == self.target {
                    self.found_index = Some(self.mid);
                    self.phase = BinarySearchPhase::Found;
                    // Teaching after found
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.state.comparisons as usize % self.state.questions.len();
                        self.state.ask_question(q_index);
                    }
                    true
                } else if self.array[self.mid] < self.target {
                    self.low = self.mid + 1;
                    // Teaching after each comparison
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.state.comparisons as usize % self.state.questions.len();
                        self.state.ask_question(q_index);
                    }
                    true
                } else {
                    self.high = self.mid.saturating_sub(1);
                    // Teaching after each comparison
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.state.comparisons as usize % self.state.questions.len();
                        self.state.ask_question(q_index);
                    }
                    true
                }
            },
            BinarySearchPhase::Found | BinarySearchPhase::NotFound => {
                false
            },
            BinarySearchPhase::Done => {
                false
            },
        }
    }

    fn reset(&mut self) {
        let mut array = self.original_array.clone();
        array.sort_unstable();
        let len = array.len();
        self.array = array;
        self.states = vec![SelectionState::Normal; len];
        self.low = 0;
        self.high = len.saturating_sub(1);
        self.mid = 0;
        self.found_index = None;
        self.phase = BinarySearchPhase::Searching;

        // Prompt for new target
        let mut stdout = stdout();
        let target = Self::prompt_for_target(&mut stdout, &self.array);
        self.target = target;
        self.intro_text = format!("What is Binary Search?\n\n\
         Binary Search is an efficient algorithm that finds the target in a sorted array by repeatedly dividing\n\
         the search interval in half. It starts by comparing the middle element with the target.\n\n\
         Target: {}\n\n\
         Advantages: O(log n) time complexity for sorted arrays.\n\
         Disadvantages: Requires the array to be sorted first.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after each comparison.\n\n\
         Press any key to continue...", target);
        self.state.reset_state();
        if len == 0 {
            self.state.mark_completed();
        }
    }

    fn mark_all_sorted(&mut self) {
        // For search, mark found as sorted, others normal
        if let Some(found) = self.found_index {
            self.states[found] = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT BINARY SEARCH VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Comparing", Color::Yellow),
            ("Found", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            BinarySearchPhase::Searching => format!("Low: {} High: {} Mid: {}", self.low, self.high, self.mid),
            BinarySearchPhase::Found => format!("Found at index {}", self.found_index.unwrap()),
            BinarySearchPhase::NotFound => "Not Found".to_string(),
            BinarySearchPhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Target: {}", self.target),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Search Range: [{}..{}]", self.low, self.high),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            if self.found_index.is_some() {
                format!("✓ Target {} found at index {}!", self.target, self.found_index.unwrap())
            } else {
                format!("✗ Target {} not found in the array.", self.target)
            }
        } else {
            match self.phase {
                BinarySearchPhase::Searching => {
                    if self.low <= self.high && self.mid < self.array.len() {
                        format!("Binary search: low={} mid={}({}) high={}, target={}", self.low, self.mid, self.array[self.mid], self.high, self.target)
                    } else {
                        "Search space exhausted".to_string()
                    }
                },
                BinarySearchPhase::Found => {
                    format!("Target {} found at index {}!", self.target, self.found_index.unwrap())
                },
                BinarySearchPhase::NotFound => {
                    format!("Target {} not found after {} comparisons.", self.target, self.state.comparisons)
                },
                BinarySearchPhase::Done => {
                    "Binary search completed!".to_string()
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
            "SEARCHING..."
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

/// Entry point for the binary search visualization
pub fn binary_search_visualization(array_data: &ArrayData) {
    let mut visualizer = BinarySearchVisualizer::new(array_data);
    visualizer.run_visualization();
}