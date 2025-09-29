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

/// Represents the different phases of the cocktail sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum CocktailPhase {
    ForwardPass,    // Forward bubble pass (left to right)
    BackwardPass,   // Backward bubble pass (right to left)
    Swapping,       // Performing a swap during a pass
    Done,           // Sorting is complete
}

/// Visualizes the cocktail sort algorithm step-by-step with interactive controls
pub struct CocktailSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)

    // Cocktail Sort specific fields
    current_i: usize,          // Current outer loop index
    current_j: usize,          // Current inner loop index
    direction: bool,           // true for forward, false for backward
    swapped: bool,             // Whether a swap occurred in the current pass
    phase: CocktailPhase,      // Current phase of the cocktail sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl CocktailSortVisualizer {
    /// Creates a new CocktailSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main difference between Cocktail Sort and Bubble Sort?".to_string(),
                options: vec![
                    "It sorts in both directions (forward and backward passes)".to_string(),
                    "It uses a different comparison operator".to_string(),
                    "It skips some elements".to_string(),
                ],
                correct_index: 0,
                explanation: "Cocktail Sort alternates between forward and backward passes, allowing elements to 'bubble' in both directions, which can reduce the number of passes needed.".to_string(),
            },
            TeachingQuestion {
                text: "Why does Cocktail Sort perform backward passes?".to_string(),
                options: vec![
                    "To move small elements faster to the beginning of the array".to_string(),
                    "To compare only even indices".to_string(),
                    "To optimize for large arrays".to_string(),
                ],
                correct_index: 0,
                explanation: "Backward passes help small elements move leftward quickly, complementing the forward passes that move large elements rightward.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Cocktail Sort?".to_string(),
                options: vec![
                    "O(n^2) in worst and average cases".to_string(),
                    "O(n log n)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Like Bubble Sort, Cocktail Sort has O(n^2) time complexity, though it performs slightly better in practice due to bidirectional passes.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            current_i: 0,
            current_j: 0,
            direction: true, // Start with forward pass
            swapped: false,
            phase: CocktailPhase::ForwardPass,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("CocktailSort".to_string());
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
                                settings.last_visualizer = Some("CocktailSort".to_string());
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

impl SortVisualizer for CocktailSortVisualizer {
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
        if self.current_i >= n - 1 {
            return false;
        }

        match self.phase {
            CocktailPhase::ForwardPass => {
                if self.current_j < n - 1 - self.current_i {
                    self.states[self.current_j] = SelectionState::Comparing;
                    self.states[self.current_j + 1] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_j] > self.array[self.current_j + 1] {
                        self.states[self.current_j] = SelectionState::Swapping;
                        self.states[self.current_j + 1] = SelectionState::Swapping;
                        self.array.swap(self.current_j, self.current_j + 1);
                        self.state.swaps += 1;
                        self.swapped = true;
                        self.phase = CocktailPhase::Swapping;
                        return true;
                    } else {
                        self.current_j += 1;
                    }
                } else {
                    // End of forward pass
                    self.current_j = n - 2 - self.current_i;
                    self.phase = CocktailPhase::BackwardPass;
                    self.direction = false;
                }
            },
            CocktailPhase::BackwardPass => {
                if self.current_j > self.current_i {
                    self.states[self.current_j] = SelectionState::Comparing;
                    self.states[self.current_j - 1] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_j - 1] > self.array[self.current_j] {
                        self.states[self.current_j - 1] = SelectionState::Swapping;
                        self.states[self.current_j] = SelectionState::Swapping;
                        self.array.swap(self.current_j - 1, self.current_j);
                        self.state.swaps += 1;
                        self.swapped = true;
                        self.phase = CocktailPhase::Swapping;
                        return true;
                    } else {
                        self.current_j -= 1;
                    }
                } else {
                    // End of backward pass
                    if self.swapped {
                        self.swapped = false;
                        self.current_i += 1;
                        self.current_j = self.current_i;
                        self.phase = CocktailPhase::ForwardPass;
                        self.direction = true;

                        // Teaching: Ask question after each full pass (forward + backward)
                        if self.state.teaching_mode && !self.state.questions.is_empty() {
                            let q_index = self.current_i % self.state.questions.len();
                            self.state.ask_question(q_index);
                            return true;
                        }
                    } else {
                        // No swaps in the last full pass, sorting is complete
                        return false;
                    }
                }
            },
            CocktailPhase::Swapping => {
                // Continue the pass after swap
                self.phase = if self.direction { CocktailPhase::ForwardPass } else { CocktailPhase::BackwardPass };
            },
            CocktailPhase::Done => return false,
        }

        true
    }

    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.current_i = 0;
        self.current_j = 0;
        self.direction = true;
        self.swapped = false;
        self.phase = CocktailPhase::ForwardPass;
        self.state.reset_state();
        if self.array.len() <= 1 {
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
        "TOGISOFT COCKTAIL SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Cocktail Sort?\n\n\
         Cocktail Sort, also known as Cocktail Shaker Sort, is a variation of Bubble Sort that alternates between forward and backward passes.\n\
         In the forward pass, larger elements bubble to the end; in the backward pass, smaller elements bubble to the beginning.\n\n\
         Advantages: Slightly faster than Bubble Sort in practice.\n\
         Disadvantages: Still O(n^2) time complexity.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after each full pass.\n\n\
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
        let direction_str = if self.direction { "Forward" } else { "Backward" };
        let phase_str = match self.phase {
            CocktailPhase::ForwardPass => format!("{} Pass", direction_str),
            CocktailPhase::BackwardPass => format!("{} Pass", direction_str),
            CocktailPhase::Swapping => "Swapping".to_string(),
            CocktailPhase::Done => "Done".to_string(),
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
            "✓ Array is now sorted using Cocktail Sort! Congratulations!".to_string()
        } else {
            let direction_str = if self.direction { "forward" } else { "backward" };
            let n = self.array.len();

            match self.phase {
                CocktailPhase::ForwardPass | CocktailPhase::BackwardPass => {
                    // FIX: Bounds checking before accessing array
                    if self.direction {
                        if self.current_j < n && self.current_j + 1 < n {
                            format!("Pass {}: comparing array[{}] ({}) with array[{}] ({}) in {} direction",
                                    self.current_i + 1,
                                    self.current_j,
                                    self.array[self.current_j],
                                    self.current_j + 1,
                                    self.array[self.current_j + 1],
                                    direction_str,
                            )
                        } else {
                            format!("Pass {}: {} pass in progress", self.current_i + 1, direction_str)
                        }
                    } else {
                        if self.current_j > 0 && self.current_j < n {
                            format!("Pass {}: comparing array[{}] ({}) with array[{}] ({}) in {} direction",
                                    self.current_i + 1,
                                    self.current_j,
                                    self.array[self.current_j],
                                    self.current_j - 1,
                                    self.array[self.current_j - 1],
                                    direction_str,
                            )
                        } else {
                            format!("Pass {}: {} pass in progress", self.current_i + 1, direction_str)
                        }
                    }
                },
                CocktailPhase::Swapping => {
                    let other_idx = if self.direction { self.current_j + 1 } else { self.current_j.saturating_sub(1) };
                    format!("Swapping elements at indices {} and {}", self.current_j, other_idx)
                },
                CocktailPhase::Done => {
                    "Cocktail sort completed!".to_string()
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

/// Entry point for the cocktail sort visualization
pub fn cocktail_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = CocktailSortVisualizer::new(array_data);
    visualizer.run_visualization();
}