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

pub struct BubbleSortVisualizer {
    array: Vec<u32>,
    original_array: Vec<u32>,
    states: Vec<SelectionState>,
    current_i: usize,
    current_j: usize,
    sorted_count: usize,
    state: VisualizerState,
    awaiting_swap_confirmation: bool,
}

impl BubbleSortVisualizer {
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What is the purpose of the outer loop in Bubble Sort?".to_string(),
                options: vec![
                    "It bubbles each element to the end like a bubble".to_string(),
                    "It only performs comparisons".to_string(),
                    "It shuffles the array".to_string()
                ],
                correct_index: 0,
                explanation: "The outer loop places the largest element at the end in each pass, thus growing the sorted part.".to_string(),
            },
            TeachingQuestion {
                text: "Why does the inner loop run up to n-i-1?".to_string(),
                options: vec![
                    "To skip the already sorted part".to_string(),
                    "To scan the entire array".to_string(),
                    "For randomness".to_string()
                ],
                correct_index: 0,
                explanation: "In each pass, the end has sorted elements, so the inner loop skips that part.".to_string(),
            },
            TeachingQuestion {
                text: "Why is a swap performed after a comparison?".to_string(),
                options: vec![
                    "If the left element is larger, it swaps to bubble the smaller one right".to_string(),
                    "It is always performed".to_string(),
                    "If it is smaller".to_string()
                ],
                correct_index: 0,
                explanation: "Bubble sort bubbles larger elements to the end like bubbles.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut visualizer = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            current_i: 0,
            current_j: 0,
            sorted_count: 0,
            state,
            awaiting_swap_confirmation: false,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("BubbleSort".to_string());
        settings.save();

        visualizer
    }

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

                        // Handle swap confirmation
                        if self.awaiting_swap_confirmation {
                            match key_event.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    self.states[self.current_j] = SelectionState::Swapping;
                                    self.states[self.current_j + 1] = SelectionState::Swapping;
                                    self.array.swap(self.current_j, self.current_j + 1);
                                    self.state.swaps += 1;
                                    self.awaiting_swap_confirmation = false;
                                    self.current_j += 1;
                                    continue;
                                },
                                KeyCode::Char('n') | KeyCode::Char('N') => {
                                    self.awaiting_swap_confirmation = false;
                                    self.current_j += 1;
                                    continue;
                                },
                                _ => {}
                            }
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
                                settings.last_visualizer = Some("BubbleSort".to_string());
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
                && !self.awaiting_swap_confirmation && self.state.awaiting_question.is_none() {
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
        if self.state.awaiting_question.is_none() && !self.awaiting_swap_confirmation {
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

impl SortVisualizer for BubbleSortVisualizer {
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
        if self.state.completed || self.awaiting_swap_confirmation || self.state.awaiting_question.is_some() {
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

        if self.current_j < n - 1 - self.current_i {
            self.states[self.current_j] = SelectionState::Comparing;
            self.states[self.current_j + 1] = SelectionState::Comparing;
            self.state.comparisons += 1;

            if self.array[self.current_j] > self.array[self.current_j + 1] {
                if self.state.is_running {
                    self.states[self.current_j] = SelectionState::Swapping;
                    self.states[self.current_j + 1] = SelectionState::Swapping;
                    self.array.swap(self.current_j, self.current_j + 1);
                    self.state.swaps += 1;
                    self.current_j += 1;
                } else {
                    self.awaiting_swap_confirmation = true;
                    return true;
                }
            } else {
                self.current_j += 1;
            }
        } else {
            self.states[n - 1 - self.current_i] = SelectionState::Sorted;
            self.sorted_count += 1;
            self.current_i += 1;
            self.current_j = 0;

            self.state.ask_question(self.current_i - 1);
        }
        true
    }

    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.current_i = 0;
        self.current_j = 0;
        self.sorted_count = 0;
        self.awaiting_swap_confirmation = false;
        self.state.reset_state();
    }

    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT BUBBLE SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Bubble Sort?\n\n\
         Bubble Sort compares elements in the array and swaps them if they are in the wrong order.\n\
         In each pass, the largest element 'bubbles' to the end.\n\n\
         Advantages: Simple.\n\
         Disadvantages: Slow (O(n^2)).\n\n\
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

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted! Congratulations!".to_string()
        } else if self.current_i < self.array.len() {
            if self.current_j < self.array.len() - 1 - self.current_i {
                format!(
                    "Pass {}: comparing array[{}] ({}) with array[{}] ({})",
                    self.current_i + 1,
                    self.current_j,
                    self.array[self.current_j],
                    self.current_j + 1,
                    self.array[self.current_j + 1]
                )
            } else {
                format!("Pass {} completed. Largest element bubbled to the end.", self.current_i + 1)
            }
        } else {
            "Initializing...".to_string()
        }
    }

    fn get_status(&self) -> &str {
        if self.awaiting_swap_confirmation {
            "AWAITING SWAP CONFIRMATION"
        } else if self.state.completed {
            "COMPLETED!"
        } else if self.state.is_running && !self.state.is_paused {
            "RUNNING..."
        } else if self.state.is_paused {
            "PAUSED"
        } else if self.state.awaiting_question.is_some() {
            "WAITING FOR QUESTION"
        } else {
            "READY"
        }
    }

    fn get_controls_text(&self) -> &str {
        if self.state.awaiting_question.is_some() {
            "1,2,3: Answer | ESC: Exit"
        } else if self.awaiting_swap_confirmation {
            "y: Yes Swap | n: No | R: Reset | ESC: Exit"
        } else if self.state.completed {
            "SPACE: Restart | R: Reset | T: Teaching Toggle | ESC: Exit"
        } else {
            "SPACE: Start/Pause | S: Step | R: Reset | T: Teaching | +/-: Speed | ESC: Exit"
        }
    }
}

pub fn bubble_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = BubbleSortVisualizer::new(array_data);
    visualizer.run_visualization();
}