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

/// Represents the different phases of the comb sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum CombPhase {
    ShrinkingGap, // Comparing and swapping elements with current gap
    Swapping,     // Performing a swap
    Done,         // Sorting is complete
}

/// Visualizes the comb sort algorithm step-by-step with interactive controls
pub struct CombSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    gap: usize,               // Current gap between compared elements
    current_i: usize,         // Current index for comparison
    swapped: bool,            // Whether a swap occurred in the current pass
    phase: CombPhase,         // Current phase of the comb sort algorithm
    state: VisualizerState,   // Common visualization state
}

impl CombSortVisualizer {
    /// Creates a new CombSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main difference between Comb Sort and Bubble Sort?".to_string(),
                options: vec![
                    "Comb Sort uses a shrinking gap for comparisons".to_string(),
                    "Comb Sort sorts in both directions".to_string(),
                    "Comb Sort skips every other element".to_string(),
                ],
                correct_index: 0,
                explanation: "Comb Sort improves Bubble Sort by using a gap that shrinks over time, allowing for faster movement of elements.".to_string(),
            },
            TeachingQuestion {
                text: "What is the typical shrink factor used in Comb Sort?".to_string(),
                options: vec![
                    "1.3".to_string(),
                    "2.0".to_string(),
                    "1.5".to_string(),
                ],
                correct_index: 0,
                explanation: "Comb Sort typically uses a shrink factor of approximately 1.3, which has been found to be effective in practice.".to_string(),
            },
            TeachingQuestion {
                text: "What is the average time complexity of Comb Sort?".to_string(),
                options: vec![
                    "O(n^2)".to_string(),
                    "O(n log n)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 1,
                explanation: "Comb Sort has an average time complexity of O(n log n), making it more efficient than Bubble Sort's O(n^2).".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            gap: len,
            current_i: 0,
            swapped: false,
            phase: CombPhase::ShrinkingGap,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("CombSort".to_string());
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
                                settings.last_visualizer = Some("CombSort".to_string());
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

impl SortVisualizer for CombSortVisualizer {
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
        if self.gap == 1 && !self.swapped {
            self.phase = CombPhase::Done;
            return false;
        }

        match self.phase {
            CombPhase::ShrinkingGap => {
                if self.current_i + self.gap < n {
                    self.states[self.current_i] = SelectionState::Comparing;
                    self.states[self.current_i + self.gap] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_i] > self.array[self.current_i + self.gap] {
                        self.states[self.current_i] = SelectionState::Swapping;
                        self.states[self.current_i + self.gap] = SelectionState::Swapping;
                        self.array.swap(self.current_i, self.current_i + self.gap);
                        self.state.swaps += 1;
                        self.swapped = true;
                        self.phase = CombPhase::Swapping;
                        return true;
                    } else {
                        self.current_i += 1;
                    }
                } else {
                    // End of pass with current gap
                    self.current_i = 0;
                    self.gap = (self.gap as f64 / 1.3).floor() as usize;
                    if self.gap < 1 {
                        self.gap = 1;
                    }
                    self.swapped = false;

                    // Teaching: Ask question after each pass
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = (n - self.gap) % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                }
            },
            CombPhase::Swapping => {
                // Continue the pass after swap
                self.current_i += 1;
                self.phase = CombPhase::ShrinkingGap;
            },
            CombPhase::Done => return false,
        }

        true
    }

    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.gap = self.array.len();
        self.current_i = 0;
        self.swapped = false;
        self.phase = CombPhase::ShrinkingGap;
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
        "TOGISOFT COMB SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Comb Sort?\n\n\
         Comb Sort is an improved version of Bubble Sort that eliminates small elements at the start of the array faster.\n\
         It uses a gap that shrinks by a factor (typically 1.3) each pass, comparing and swapping elements separated by the gap.\n\n\
         Advantages: More efficient than Bubble Sort with average O(n log n) complexity.\n\
         Disadvantages: Still not as efficient as advanced sorting algorithms like Quick Sort.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after each pass.\n\n\
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
            CombPhase::ShrinkingGap => format!("Gap: {}", self.gap),
            CombPhase::Swapping => "Swapping".to_string(),
            CombPhase::Done => "Done".to_string(),
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
            "✓ Array is now sorted using Comb Sort! Congratulations!".to_string()
        } else {
            let n = self.array.len();
            match self.phase {
                CombPhase::ShrinkingGap => {
                    // FIX: Bounds checking before accessing array
                    if self.current_i < n && self.current_i + self.gap < n {
                        format!("Comparing array[{}] ({}) with array[{}] ({}) with gap {}",
                                self.current_i,
                                self.array[self.current_i],
                                self.current_i + self.gap,
                                self.array[self.current_i + self.gap],
                                self.gap)
                    } else {
                        format!("Processing with gap {}", self.gap)
                    }
                },
                CombPhase::Swapping => {
                    format!("Swapping elements at indices {} and {}", self.current_i, self.current_i + self.gap)
                },
                CombPhase::Done => {
                    "Comb sort completed!".to_string()
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

/// Entry point for the comb sort visualization
pub fn comb_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = CombSortVisualizer::new(array_data);
    visualizer.run_visualization();
}