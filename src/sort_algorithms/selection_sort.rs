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

/// Represents the different phases of the selection sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum SelectionPhase {
    SelectingPosition,  // Finding the position to fill
    SearchingMin,       // Searching for the minimum element
    FoundMin,           // Found the minimum, ready to swap
    Swapping,           // Performing the swap
}

/// Visualizes the selection sort algorithm step-by-step with interactive controls
pub struct SelectionSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    intro_text: String,        // Dynamic intro text
    current_i: usize,          // Current position being filled
    current_j: usize,          // Current index being examined
    min_index: usize,          // Index of the current minimum element
    phase: SelectionPhase,     // Current phase of the selection sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl SelectionSortVisualizer {
    /// Creates a new SelectionSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What does the outer loop in Selection Sort do?".to_string(),
                options: vec![
                    "It selects the position to place the next smallest element".to_string(),
                    "It compares adjacent elements".to_string(),
                    "It divides the array".to_string(),
                ],
                correct_index: 0,
                explanation: "The outer loop iterates over each position, finding the minimum in the unsorted portion.".to_string(),
            },
            TeachingQuestion {
                text: "Why search for the minimum in the unsorted part?".to_string(),
                options: vec![
                    "To place the smallest remaining element in the next sorted position".to_string(),
                    "To swap adjacent elements".to_string(),
                    "To build a heap".to_string(),
                ],
                correct_index: 0,
                explanation: "By finding and swapping the minimum to the current position, the sorted prefix grows.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Selection Sort?".to_string(),
                options: vec![
                    "O(n^2)".to_string(),
                    "O(n log n)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Selection Sort makes n-1 passes, each scanning up to n elements, resulting in O(n^2).".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Selection Sort?\n\n\
             Selection Sort divides the input list into two parts: the sorted and unsorted.\n\
             In each pass, it searches the unsorted part for the minimum element and swaps it with the first unsorted element.\n\n\
             Advantages: Simple, in-place.\n\
             Disadvantages: O(n^2) time, not stable.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each selection.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            intro_text,
            current_i: 0,
            current_j: 0,
            min_index: 0,
            phase: SelectionPhase::SelectingPosition,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("SelectionSort".to_string());
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
                                self.intro_text = format!(
                                    "What is Selection Sort?\n\n\
                                     Selection Sort divides the input list into two parts: the sorted and unsorted.\n\
                                     In each pass, it searches the unsorted part for the minimum element and swaps it with the first unsorted element.\n\n\
                                     Advantages: Simple, in-place.\n\
                                     Disadvantages: O(n^2) time, not stable.\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each selection.\n\n\
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
                                settings.last_visualizer = Some("SelectionSort".to_string());
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

impl SortVisualizer for SelectionSortVisualizer {
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
        if self.array.len() <= 1 {
            100.0
        } else {
            (self.current_i as f64 / (self.array.len() - 1) as f64 * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() || self.current_i >= self.array.len() {
            return true;
        }

        // Reset all states except sorted ones
        for (i, state) in self.states.iter_mut().enumerate() {
            if *state != SelectionState::Sorted {
                if i < self.current_i {
                    *state = SelectionState::Sorted;
                } else {
                    *state = SelectionState::Normal;
                }
            }
        }

        match self.phase {
            SelectionPhase::SelectingPosition => {
                // Mark the position we're trying to fill
                self.states[self.current_i] = SelectionState::Selected;
                self.min_index = self.current_i;
                self.current_j = self.current_i + 1; // Start from next element
                if self.current_j >= self.array.len() {
                    self.phase = SelectionPhase::FoundMin;
                } else {
                    self.phase = SelectionPhase::SearchingMin;
                }
            },
            SelectionPhase::SearchingMin => {
                if self.current_j < self.array.len() {
                    // Mark current minimum
                    self.states[self.min_index] = SelectionState::CurrentMin;
                    // Mark element being compared
                    if self.current_j != self.min_index {
                        self.states[self.current_j] = SelectionState::Comparing;
                    }
                    self.state.comparisons += 1;

                    // Check if current element is smaller than current minimum
                    if self.array[self.current_j] < self.array[self.min_index] {
                        self.min_index = self.current_j;
                    }

                    self.current_j += 1;

                    // If we've checked all elements, move to found phase
                    if self.current_j >= self.array.len() {
                        self.phase = SelectionPhase::FoundMin;
                    }
                } else {
                    self.phase = SelectionPhase::FoundMin;
                }
            },
            SelectionPhase::FoundMin => {
                // Mark both positions for swapping
                self.states[self.current_i] = SelectionState::Swapping;
                self.states[self.min_index] = SelectionState::Swapping;
                self.phase = SelectionPhase::Swapping;
            },
            SelectionPhase::Swapping => {
                // Perform the swap if needed
                if self.current_i != self.min_index {
                    self.array.swap(self.current_i, self.min_index);
                    self.state.swaps += 1;
                }

                // Mark current position as sorted
                self.states[self.current_i] = SelectionState::Sorted;

                // Move to next position
                self.current_i += 1;

                if self.current_i >= self.array.len() {
                    return false; // Sorting complete
                }

                // Teaching: Ask question after each outer loop iteration
                if self.state.teaching_mode && !self.state.questions.is_empty() {
                    let q_index = self.current_i % self.state.questions.len();
                    self.state.ask_question(q_index);
                    return true;
                }

                self.phase = SelectionPhase::SelectingPosition;
            },
        }

        true
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.current_i = 0;
        self.current_j = 0;
        self.min_index = 0;
        self.phase = SelectionPhase::SelectingPosition;
        self.state.reset_state();
        self.intro_text = format!(
            "What is Selection Sort?\n\n\
             Selection Sort divides the input list into two parts: the sorted and unsorted.\n\
             In each pass, it searches the unsorted part for the minimum element and swaps it with the first unsorted element.\n\n\
             Advantages: Simple, in-place.\n\
             Disadvantages: O(n^2) time, not stable.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each selection.\n\n\
             Press any key to continue...",
            if self.state.teaching_mode { "ON" } else { "OFF" }
        );
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
        "TOGISOFT SELECTION SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Current Min", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Selected Pos", Color::White),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            SelectionPhase::SelectingPosition => "Selecting Position",
            SelectionPhase::SearchingMin => "Searching Min",
            SelectionPhase::FoundMin => "Found Min",
            SelectionPhase::Swapping => "Swapping",
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Swaps: {}", self.state.swaps),
            format!("Current i: {}", self.current_i),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Selection Sort! Congratulations!".to_string()
        } else if self.current_i < self.array.len() {
            match self.phase {
                SelectionPhase::SelectingPosition => {
                    format!("Step {}/{}: Selecting position {} to fill",
                            self.current_i + 1, self.array.len(), self.current_i)
                },
                SelectionPhase::SearchingMin => {
                    format!("Step {}/{}: Searching for minimum in range [{}..{}] - Comparing {} with current min {} at index {}",
                            self.current_i + 1, self.array.len(),
                            self.current_i, self.array.len() - 1,
                            if self.current_j < self.array.len() { self.array[self.current_j] } else { 0 },
                            if self.min_index < self.array.len() { self.array[self.min_index] } else { 0 },
                            self.min_index)
                },
                SelectionPhase::FoundMin => {
                    format!("Step {}/{}: Found minimum {} at index {} - Ready to swap with position {}",
                            self.current_i + 1, self.array.len(),
                            if self.min_index < self.array.len() { self.array[self.min_index] } else { 0 },
                            self.min_index, self.current_i)
                },
                SelectionPhase::Swapping => {
                    format!("Step {}/{}: Swapping {} (pos {}) with {} (pos {})",
                            self.current_i + 1, self.array.len(),
                            if self.current_i < self.array.len() { self.array[self.current_i] } else { 0 },
                            self.current_i,
                            if self.min_index < self.array.len() { self.array[self.min_index] } else { 0 },
                            self.min_index)
                },
            }
        } else {
            "Sorting complete".to_string()
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

/// Entry point for the selection sort visualization
pub fn selection_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = SelectionSortVisualizer::new(array_data);
    visualizer.run_visualization();
}