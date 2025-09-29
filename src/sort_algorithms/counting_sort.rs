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

/// Represents the different phases of the counting sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum CountingPhase {
    Counting,      // Counting occurrences of each value
    PrefixSum,     // Building cumulative counts
    Placing,       // Placing elements in sorted positions
    Done,          // Sorting is complete
}

/// Visualizes the counting sort algorithm step-by-step with interactive controls
pub struct CountingSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, sorted)

    // Counting Sort specific fields
    count: Vec<usize>,         // Count array
    min_val: u32,              // Minimum value in array
    max_val: u32,              // Maximum value in array
    range: usize,              // Range of values (max - min + 1)
    current_i: usize,          // Current index
    last_val: u32,             // Last processed value
    last_pos: usize,           // Last placement position
    last_count_idx: usize,     // Last count index used
    phase: CountingPhase,      // Current phase of the counting sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl CountingSortVisualizer {
    /// Creates a new CountingSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();
        let (min_val, max_val) = if len == 0 {
            (0u32, 0u32)
        } else {
            (array.iter().min().unwrap().clone(), array.iter().max().unwrap().clone())
        };
        let range = (max_val.saturating_sub(min_val) + 1) as usize;

        let mut questions = vec![
            TeachingQuestion {
                text: "What is Counting Sort best suited for?".to_string(),
                options: vec![
                    "Arrays with small integer range".to_string(),
                    "Random floating point numbers".to_string(),
                    "Very large unsorted arrays".to_string(),
                ],
                correct_index: 0,
                explanation: "Counting Sort excels when the range of input values is small compared to the number of elements.".to_string(),
            },
            TeachingQuestion {
                text: "The time complexity of Counting Sort is:".to_string(),
                options: vec![
                    "O(n + k) where k is the range".to_string(),
                    "O(n log n)".to_string(),
                    "O(n^2)".to_string(),
                ],
                correct_index: 0,
                explanation: "Counting Sort has linear time complexity O(n + k), where n is the number of elements and k is the range of values.".to_string(),
            },
            TeachingQuestion {
                text: "Counting Sort is:".to_string(),
                options: vec![
                    "Stable".to_string(),
                    "In-place".to_string(),
                    "Comparison-based".to_string(),
                ],
                correct_index: 0,
                explanation: "Counting Sort is stable, meaning equal elements retain their relative order, but it requires additional space.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            count: vec![0; range],
            min_val,
            max_val,
            range,
            current_i: 0,
            last_val: 0,
            last_pos: 0,
            last_count_idx: 0,
            phase: CountingPhase::Counting,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("CountingSort".to_string());
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
                                settings.last_visualizer = Some("CountingSort".to_string());
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

impl SortVisualizer for CountingSortVisualizer {
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
        match self.phase {
            CountingPhase::Counting => {
                if self.current_i < n {
                    let idx = self.current_i;
                    self.states[idx] = SelectionState::Comparing;
                    let val = self.array[idx];
                    let c_idx = (val - self.min_val) as usize;
                    self.count[c_idx] += 1;
                    self.last_val = val;
                    self.last_count_idx = c_idx;
                    self.state.comparisons += 1;
                    self.current_i += 1;
                    return true;
                } else {
                    // End of counting
                    self.phase = CountingPhase::PrefixSum;
                    self.current_i = 0;
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = 0 % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                    return true;
                }
            },
            CountingPhase::PrefixSum => {
                if self.current_i < self.range.saturating_sub(1) {
                    let prev = self.current_i;
                    self.count[self.current_i + 1] += self.count[prev];
                    self.last_count_idx = prev;
                    self.state.comparisons += 1;
                    self.current_i += 1;
                    return true;
                } else {
                    // End of prefix sum
                    self.phase = CountingPhase::Placing;
                    self.current_i = n;
                    return true;
                }
            },
            CountingPhase::Placing => {
                if self.current_i > 0 {
                    self.current_i -= 1;
                    let val = self.original_array[self.current_i];
                    let idx = (val - self.min_val) as usize;
                    let pos = self.count[idx].saturating_sub(1);
                    self.array[pos] = val;
                    self.states[pos] = SelectionState::Sorted;
                    self.count[idx] -= 1;
                    self.last_val = val;
                    self.last_pos = pos;
                    self.last_count_idx = idx;
                    self.state.swaps += 1;
                    return true;
                } else {
                    self.phase = CountingPhase::Done;
                    return false;
                }
            },
            CountingPhase::Done => return false,
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.count = vec![0; self.range];
        self.current_i = 0;
        self.last_val = 0;
        self.last_pos = 0;
        self.last_count_idx = 0;
        self.phase = CountingPhase::Counting;
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
        "TOGISOFT COUNTING SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Counting Sort?\n\n\
         Counting Sort is a non-comparison sorting algorithm that counts the occurrences of each value and uses arithmetic to determine positions.\n\
         It works in three phases: counting occurrences, building cumulative counts, and placing elements in sorted order.\n\n\
         Advantages: Linear time O(n + k) for integer ranges, stable.\n\
         Disadvantages: Requires knowing the value range, uses O(k) extra space.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after counting phase.\n\n\
         Press any key to continue..."
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Comparing", Color::Yellow),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            CountingPhase::Counting => "Counting Occurrences".to_string(),
            CountingPhase::PrefixSum => "Building Cumulative Counts".to_string(),
            CountingPhase::Placing => "Placing Elements".to_string(),
            CountingPhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Min Value: {}", self.min_val),
            format!("Max Value: {}", self.max_val),
            format!("Range: {}", self.range),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Placements: {}", self.state.swaps),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Counting Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                CountingPhase::Counting => {
                    format!("Counting: value {} -> count[{}]", self.last_val, self.last_count_idx)
                },
                CountingPhase::PrefixSum => {
                    format!("Cumulative: count[{}] += count[{}]", self.last_count_idx + 1, self.last_count_idx)
                },
                CountingPhase::Placing => {
                    format!("Placing value {} at position {}", self.last_val, self.last_pos)
                },
                CountingPhase::Done => {
                    "Counting sort completed!".to_string()
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

/// Entry point for the counting sort visualization
pub fn counting_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = CountingSortVisualizer::new(array_data);
    visualizer.run_visualization();
}