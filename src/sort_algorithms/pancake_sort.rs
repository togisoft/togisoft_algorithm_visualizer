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

/// Represents the different phases of the pancake sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum PancakePhase {
    FindingMax,    // Finding the maximum element in the unsorted portion
    FlippingToFront, // Flipping prefix to bring max to front
    FlippingToEnd,   // Flipping prefix to move max to end
    Done,          // Sorting is complete
}

/// Visualizes the pancake sort algorithm step-by-step with interactive controls
pub struct PancakeSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, flipping, sorted)

    // Pancake Sort specific fields
    unsorted_size: usize,      // Current size of unsorted portion
    max_pos: usize,            // Position of the current maximum
    flip_pos: usize,           // Position for flipping
    phase: PancakePhase,       // Current phase of the pancake sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl PancakeSortVisualizer {
    /// Creates a new PancakeSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main idea behind Pancake Sort?".to_string(),
                options: vec![
                    "Bringing the largest unsorted element to the end by flipping prefixes".to_string(),
                    "Swapping adjacent elements like bubble sort".to_string(),
                    "Dividing the array into smaller subarrays".to_string(),
                ],
                correct_index: 0,
                explanation: "Pancake Sort repeatedly finds the maximum element in the unsorted portion and flips the prefix to bring it to the front, then to the end.".to_string(),
            },
            TeachingQuestion {
                text: "Why is Pancake Sort called 'pancake' sort?".to_string(),
                options: vec![
                    "It flips portions of the array like flipping pancakes".to_string(),
                    "It sorts circular arrays".to_string(),
                    "It uses a stack-like structure".to_string(),
                ],
                correct_index: 0,
                explanation: "The flipping operation mimics flipping a stack of pancakes to sort them by size.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Pancake Sort?".to_string(),
                options: vec![
                    "O(n log n)".to_string(),
                    "O(n^2)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 1,
                explanation: "Pancake Sort has O(n^2) time complexity due to the linear scan for maximum in each iteration.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            unsorted_size: len,
            max_pos: 0,
            flip_pos: 0,
            phase: PancakePhase::FindingMax,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("PancakeSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
            this.mark_all_sorted();
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
                                settings.last_visualizer = Some("PancakeSort".to_string());
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

    /// Perform a pancake flip from 0 to flip_pos
    fn flip_prefix(&mut self, flip_pos: usize) {
        let mut temp = self.array[0..=flip_pos].to_vec();
        temp.reverse();
        self.array.splice(0..=flip_pos, temp);

        // Update states for flipped elements
        for i in 0..=flip_pos {
            self.states[i] = SelectionState::Swapping;
        }
        self.state.swaps += 1;
    }
}

impl SortVisualizer for PancakeSortVisualizer {
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
        let total = (self.array.len() * (self.array.len() - 1)) / 2;
        if total == 0 { 100.0 } else {
            (self.state.comparisons as f64 / total as f64 * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states except sorted
        for state in &mut self.states {
            if *state != SelectionState::Sorted {
                *state = SelectionState::Normal;
            }
        }

        let n = self.array.len();
        if self.unsorted_size <= 1 {
            self.phase = PancakePhase::Done;
            return false;
        }

        match self.phase {
            PancakePhase::FindingMax => {
                // Scan for max in unsorted portion
                if self.max_pos < self.unsorted_size {
                    self.states[self.max_pos] = SelectionState::Comparing;
                    if self.max_pos + 1 < self.unsorted_size {
                        self.states[self.max_pos + 1] = SelectionState::Comparing;
                        self.state.comparisons += 1;
                        if self.array[self.max_pos] < self.array[self.max_pos + 1] {
                            self.max_pos += 1;
                        }
                    } else {
                        self.max_pos += 1;
                    }
                    return true;
                } else {
                    // Max found, prepare to flip to front
                    self.max_pos -= 1; // Adjust for 0-based
                    if self.max_pos != self.unsorted_size - 1 {
                        self.phase = PancakePhase::FlippingToFront;
                        self.flip_pos = self.max_pos;
                    } else {
                        // Max already at end, reduce unsorted size
                        self.unsorted_size -= 1;
                        self.states[self.unsorted_size] = SelectionState::Sorted;
                        self.max_pos = 0;
                        self.phase = PancakePhase::FindingMax;

                        // Teaching: Ask question after placing a max
                        if self.state.teaching_mode && !self.state.questions.is_empty() {
                            let q_index = (n - self.unsorted_size) % self.state.questions.len();
                            self.state.ask_question(q_index);
                            return true;
                        }
                    }
                    return true;
                }
            },
            PancakePhase::FlippingToFront => {
                // Flip to bring max to front
                self.flip_prefix(self.flip_pos);
                self.phase = PancakePhase::FlippingToEnd;
                self.flip_pos = self.unsorted_size - 1;
                return true;
            },
            PancakePhase::FlippingToEnd => {
                // Flip to move max to end
                self.flip_prefix(self.flip_pos);
                self.unsorted_size -= 1;
                self.states[self.unsorted_size] = SelectionState::Sorted;
                self.max_pos = 0;
                self.phase = PancakePhase::FindingMax;

                // Teaching: Ask question after placing a max
                if self.state.teaching_mode && !self.state.questions.is_empty() {
                    let q_index = (n - self.unsorted_size) % self.state.questions.len();
                    self.state.ask_question(q_index);
                    return true;
                }
                return true;
            },
            PancakePhase::Done => return false,
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.unsorted_size = len;
        self.max_pos = 0;
        self.flip_pos = 0;
        self.phase = PancakePhase::FindingMax;
        self.state.reset_state();
        if len <= 1 {
            self.state.mark_completed();
            self.mark_all_sorted();
        }
    }

    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT PANCAKE SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Pancake Sort?\n\n\
         Pancake Sort is a sorting algorithm that simulates sorting a stack of pancakes by size using only flips of prefixes.\n\
         It finds the largest unsorted pancake, flips it to the top, then flips the entire unsorted stack to place it at the bottom.\n\n\
         Advantages: Fun visualization, simple concept.\n\
         Disadvantages: O(n^2) time complexity, not practical for large arrays.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after placing each maximum.\n\n\
         Press any key to continue..."
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Comparing", Color::Yellow),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            PancakePhase::FindingMax => format!("Finding max in [0..{}]", self.unsorted_size),
            PancakePhase::FlippingToFront => "Flipping max to front".to_string(),
            PancakePhase::FlippingToEnd => "Flipping max to end".to_string(),
            PancakePhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Unsorted Size: {}", self.unsorted_size),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Flips: {}", self.state.swaps),
            format!("Max Pos: {}", self.max_pos),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Pancake Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                PancakePhase::FindingMax => {
                    format!("Scanning for max in unsorted portion [0..{}]", self.unsorted_size)
                },
                PancakePhase::FlippingToFront => {
                    format!("Flipping prefix of length {} to bring max to front", self.flip_pos + 1)
                },
                PancakePhase::FlippingToEnd => {
                    format!("Flipping prefix of length {} to place max at end", self.flip_pos + 1)
                },
                PancakePhase::Done => {
                    "Pancake sort completed!".to_string()
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

/// Entry point for the pancake sort visualization
pub fn pancake_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = PancakeSortVisualizer::new(array_data);
    visualizer.run_visualization();
}