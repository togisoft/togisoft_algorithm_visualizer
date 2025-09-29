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

/// Represents the different phases of the insertion sort algorithm
#[derive(Clone, Copy, PartialEq)]
enum InsertionPhase {
    SelectingElement,    // Selecting the next element to insert
    SearchingPosition,   // Comparing and shifting elements to find the correct position
    InsertingElement,    // Inserting the element at its correct position
    MoveToNext,          // Moving to the next element
}

/// Visualizes the insertion sort algorithm step-by-step with interactive controls
pub struct InsertionSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, shifting, sorted)
    intro_text: String,        // Dynamic intro text
    current_i: usize,          // Current outer loop index (element to insert)
    current_j: usize,          // Current inner loop index (position being compared)
    key: u32,                  // Current key element being inserted
    phase: InsertionPhase,     // Current phase of the insertion sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl InsertionSortVisualizer {
    /// Creates a new InsertionSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();
        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main idea of Insertion Sort?".to_string(),
                options: vec![
                    "It builds the sorted array one item at a time".to_string(),
                    "It divides the array into halves".to_string(),
                    "It uses a pivot to partition".to_string(),
                ],
                correct_index: 0,
                explanation: "Insertion Sort constructs a sorted subarray incrementally by inserting each unsorted element into its correct position.".to_string(),
            },
            TeachingQuestion {
                text: "In Insertion Sort, what happens in the inner loop?".to_string(),
                options: vec![
                    "Compares and shifts elements larger than the key".to_string(),
                    "Swaps adjacent elements if out of order".to_string(),
                    "Finds the minimum element".to_string(),
                ],
                correct_index: 0,
                explanation: "The inner loop shifts elements that are greater than the key one position to the right to make space for insertion.".to_string(),
            },
            TeachingQuestion {
                text: "Why is the first element considered sorted?".to_string(),
                options: vec![
                    "A single element is always sorted".to_string(),
                    "It is the largest".to_string(),
                    "It needs to be compared".to_string(),
                ],
                correct_index: 0,
                explanation: "By definition, a single element array is sorted, so the loop starts from the second element.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Insertion Sort?\n\n\
             Insertion Sort builds the final sorted array one item at a time. It is much like sorting a hand of playing cards: assume the cards are to the left of your hand are in sorted order. For each new card, you slide it into the correct position among the cards to its left.\n\n\
             Advantages: Simple, efficient for small or nearly sorted data.\n\
             Disadvantages: O(n^2) worst case.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each insertion.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            intro_text,
            current_i: if len <= 1 { len } else { 1 },
            current_j: 0,
            key: 0,
            phase: if len <= 1 { InsertionPhase::MoveToNext } else { InsertionPhase::SelectingElement },
            state,
        };

        if len > 0 {
            this.states[0] = SelectionState::Sorted; // First element is always sorted
        }

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("InsertionSort".to_string());
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
                                    "What is Insertion Sort?\n\n\
                                     Insertion Sort builds the final sorted array one item at a time. It is much like sorting a hand of playing cards: assume the cards are to the left of your hand are in sorted order. For each new card, you slide it into the correct position among the cards to its left.\n\n\
                                     Advantages: Simple, efficient for small or nearly sorted data.\n\
                                     Disadvantages: O(n^2) worst case.\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each insertion.\n\n\
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
                                settings.last_visualizer = Some("InsertionSort".to_string());
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

impl SortVisualizer for InsertionSortVisualizer {
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
            ((self.current_i as f64) / (self.array.len() as f64) * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
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

        let result = match self.phase {
            InsertionPhase::SelectingElement => {
                if self.current_i >= self.array.len() {
                    false // Done sorting
                } else {
                    // Select the key element
                    self.key = self.array[self.current_i];
                    self.states[self.current_i] = SelectionState::CurrentMin;
                    self.current_j = if self.current_i > 0 { self.current_i - 1 } else { 0 };

                    if self.current_i == 0 {
                        // First element is already sorted
                        self.states[0] = SelectionState::Sorted;
                        self.current_i += 1;
                        self.current_i < self.array.len()
                    } else {
                        self.phase = InsertionPhase::SearchingPosition;
                        true
                    }
                }
            },
            InsertionPhase::SearchingPosition => {
                // Compare key with current element
                if self.current_j < self.array.len() {
                    self.states[self.current_j] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_j] > self.key {
                        // Need to shift this element right
                        self.states[self.current_j] = SelectionState::Swapping;
                        if self.current_j + 1 < self.array.len() {
                            self.array[self.current_j + 1] = self.array[self.current_j];
                            self.state.swaps += 1;
                        }

                        if self.current_j > 0 {
                            self.current_j -= 1;
                            true
                        } else {
                            // Reached the beginning, insert here
                            self.phase = InsertionPhase::InsertingElement;
                            true
                        }
                    } else {
                        // Found correct position (after current element)
                        self.current_j += 1;
                        self.phase = InsertionPhase::InsertingElement;
                        true
                    }
                } else {
                    self.phase = InsertionPhase::InsertingElement;
                    true
                }
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
                    false // Done sorting
                } else {
                    // Teaching: Ask question after each insertion
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = (self.current_i - 1) % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                    self.phase = InsertionPhase::SelectingElement;
                    true
                }
            },
        };

        result
    }

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
        self.phase = if len <= 1 { InsertionPhase::MoveToNext } else { InsertionPhase::SelectingElement };
        self.state.reset_state();
        self.intro_text = format!(
            "What is Insertion Sort?\n\n\
             Insertion Sort builds the final sorted array one item at a time. It is much like sorting a hand of playing cards: assume the cards are to the left of your hand are in sorted order. For each new card, you slide it into the correct position among the cards to its left.\n\n\
             Advantages: Simple, efficient for small or nearly sorted data.\n\
             Disadvantages: O(n^2) worst case.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each insertion.\n\n\
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
        "TOGISOFT INSERTION SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Key Element", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Position", Color::White),
            ("Shifting", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Shifts: {}", self.state.swaps),
            format!("Current Index: {}", if self.current_i < self.array.len() { self.current_i.to_string() } else { "Done".to_string() }),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Insertion Sort! Congratulations!".to_string()
        } else {
            match self.phase {
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

/// Entry point for the insertion sort visualization
pub fn insertion_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = InsertionSortVisualizer::new(array_data);
    visualizer.run_visualization();
}