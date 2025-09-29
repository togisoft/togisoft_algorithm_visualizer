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

/// Represents the different phases of the gnome sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum GnomePhase {
    Comparing,  // Comparing adjacent elements
    Swapping,   // Performing a swap and moving back
    Done,       // Sorting is complete
}

/// Visualizes the gnome sort algorithm step-by-step with interactive controls
pub struct GnomeSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)

    // Gnome Sort specific fields
    current_i: usize,          // Current index
    phase: GnomePhase,         // Current phase of the gnome sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl GnomeSortVisualizer {
    /// Creates a new GnomeSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is Gnome Sort also known as?".to_string(),
                options: vec![
                    "Stupid Sort".to_string(),
                    "Simple Sort".to_string(),
                    "Gnome Bubble Sort".to_string(),
                ],
                correct_index: 0,
                explanation: "Gnome Sort is also known as 'Stupid Sort' due to its simple, naive approach to sorting.".to_string(),
            },
            TeachingQuestion {
                text: "How does Gnome Sort differ from standard Insertion Sort?".to_string(),
                options: vec![
                    "It moves backward after a swap instead of shifting elements".to_string(),
                    "It uses a different comparison method".to_string(),
                    "It sorts in reverse order".to_string(),
                ],
                correct_index: 0,
                explanation: "In Gnome Sort, after swapping two out-of-order elements, the index moves backward to check the previous pair, unlike insertion sort which shifts elements forward.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Gnome Sort?".to_string(),
                options: vec![
                    "O(n^2) in worst and average cases".to_string(),
                    "O(n log n)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Gnome Sort has O(n^2) time complexity, similar to other simple sorting algorithms like Bubble Sort.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            current_i: 1,
            phase: GnomePhase::Comparing,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("GnomeSort".to_string());
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
                                settings.last_visualizer = Some("GnomeSort".to_string());
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

impl SortVisualizer for GnomeSortVisualizer {
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
        if self.current_i >= n {
            return false;
        }

        match self.phase {
            GnomePhase::Comparing => {
                if self.current_i == 0 {
                    self.current_i = 1;
                }
                if self.current_i < n {
                    self.states[self.current_i - 1] = SelectionState::Comparing;
                    self.states[self.current_i] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_i - 1] <= self.array[self.current_i] {
                        self.phase = GnomePhase::Comparing;
                        self.current_i += 1;
                    } else {
                        // Prepare for swap
                        self.phase = GnomePhase::Swapping;
                    }
                    return true;
                } else {
                    return false;
                }
            },
            GnomePhase::Swapping => {
                if self.current_i > 0 {
                    self.states[self.current_i - 1] = SelectionState::Swapping;
                    self.states[self.current_i] = SelectionState::Swapping;
                    self.array.swap(self.current_i - 1, self.current_i);
                    self.state.swaps += 1;
                    self.current_i -= 1;
                    if self.current_i == 0 {
                        self.current_i = 1;
                    }
                    self.phase = GnomePhase::Comparing;

                    // Teaching: Ask question after a swap
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.state.swaps as usize % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                }
                return true;
            },
            GnomePhase::Done => return false,
        }
    }

    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.current_i = 1;
        self.phase = GnomePhase::Comparing;
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
        "TOGISOFT GNOME SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Gnome Sort?\n\n\
         Gnome Sort, also known as Stupid Sort, is a simple sorting algorithm that compares adjacent elements.\n\
         If they are in the wrong order, it swaps them and moves backward to check the previous pair.\n\
         It continues forward when elements are in order.\n\n\
         Advantages: Very simple to implement.\n\
         Disadvantages: O(n^2) time complexity, inefficient for large arrays.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after swaps.\n\n\
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
            GnomePhase::Comparing => "Comparing Adjacent Elements".to_string(),
            GnomePhase::Swapping => "Swapping and Moving Back".to_string(),
            GnomePhase::Done => "Done".to_string(),
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
            "✓ Array is now sorted using Gnome Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                GnomePhase::Comparing => {
                    if self.current_i < self.array.len() {
                        format!("Comparing array[{}] ({}) with array[{}] ({})",
                                self.current_i - 1,
                                self.array[self.current_i - 1],
                                self.current_i,
                                self.array[self.current_i])
                    } else {
                        "Reached end of array".to_string()
                    }
                },
                GnomePhase::Swapping => {
                    format!("Swapping array[{}] and array[{}], moving back",
                            self.current_i - 1,
                            self.current_i)
                },
                GnomePhase::Done => {
                    "Gnome sort completed!".to_string()
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

/// Entry point for the gnome sort visualization
pub fn gnome_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = GnomeSortVisualizer::new(array_data);
    visualizer.run_visualization();
}