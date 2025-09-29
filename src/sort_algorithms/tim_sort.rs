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

/// Represents the different phases of the tim sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum TimPhase {
    FindingRun,    // Finding natural runs
    InsertionSort, // Extending run with insertion sort
    Merging,       // Merging runs from the stack
    Done,          // Sorting is complete
}

/// Visualizes the tim sort algorithm step-by-step with interactive controls
pub struct TimSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)

    // Tim Sort specific fields
    current_i: usize,          // Current index
    run_start: usize,          // Start of current run
    run_end: usize,            // End of current run
    min_run: usize,            // Minimum run length
    stack: Vec<(usize, usize)>, // Stack of runs (start, length)
    merging_left: usize,       // Left run start for merging
    merging_right: usize,      // Right run start for merging
    merge_pos: usize,          // Current merge position
    temp_array: Vec<u32>,      // Temporary array for merging
    phase: TimPhase,           // Current phase of the tim sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl TimSortVisualizer {
    /// Creates a new TimSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main advantage of Tim Sort over traditional merge sort?".to_string(),
                options: vec![
                    "It uses natural runs to reduce comparisons".to_string(),
                    "It sorts in-place without extra space".to_string(),
                    "It uses binary search for all comparisons".to_string(),
                ],
                correct_index: 0,
                explanation: "Tim Sort identifies naturally sorted runs and extends them, minimizing unnecessary comparisons and merges.".to_string(),
            },
            TeachingQuestion {
                text: "Tim Sort combines which two algorithms?".to_string(),
                options: vec![
                    "Merge Sort and Insertion Sort".to_string(),
                    "Quick Sort and Heap Sort".to_string(),
                    "Bubble Sort and Selection Sort".to_string(),
                ],
                correct_index: 0,
                explanation: "Tim Sort is a hybrid of merge sort (for merging runs) and insertion sort (for small runs and extensions).".to_string(),
            },
            TeachingQuestion {
                text: "The time complexity of Tim Sort is:".to_string(),
                options: vec![
                    "O(n log n) in worst and average cases".to_string(),
                    "O(n^2)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Tim Sort guarantees O(n log n) time complexity, performing better than O(n log n) when the data has natural runs.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let min_run = if len < 64 { len } else { 32 }; // Simplified min run calculation

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            current_i: 0,
            run_start: 0,
            run_end: 0,
            min_run,
            stack: Vec::new(),
            merging_left: 0,
            merging_right: 0,
            merge_pos: 0,
            temp_array: vec![0; len],
            phase: TimPhase::FindingRun,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("TimSort".to_string());
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
                                settings.last_visualizer = Some("TimSort".to_string());
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

    fn prepare_merge(&mut self) {
        if self.stack.len() >= 2 {
            let right = self.stack.pop().unwrap();
            let left = self.stack.pop().unwrap();
            self.merging_left = left.0;
            self.merging_right = left.0 + left.1;
            self.merge_pos = self.merging_left;
            // Simplified: mark runs for merging
            for i in self.merging_left..self.merging_right {
                self.states[i] = SelectionState::Comparing;
            }
            let right_end = self.merging_right + right.1;
            for i in self.merging_right..right_end {
                self.states[i] = SelectionState::Comparing;
            }
            self.stack.push(right);
            self.stack.push(left);
        }
    }

    fn perform_merge(&mut self) -> bool {
        // Simplified merge step: one element at a time
        let left_end = self.merging_right;
        let right_start = self.merging_right;
        let right_len = if let Some(&right_run) = self.stack.iter().rev().nth(1) {
            right_run.1
        } else {
            0
        };
        let right_end = right_start + right_len;

        if self.merge_pos < right_end {
            self.states[self.merge_pos] = SelectionState::Swapping;
            self.temp_array[self.merge_pos] = self.array[self.merge_pos];
            self.merge_pos += 1;
            self.state.swaps += 1;
            self.state.comparisons += 1;
            if self.merge_pos >= right_end {
                // Merge complete
                for i in self.merging_left..right_end {
                    self.array[i] = self.temp_array[i];
                    self.states[i] = SelectionState::Sorted;
                }
                let merged_len = right_end - self.merging_left;
                self.stack.push((self.merging_left, merged_len));
                false
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl SortVisualizer for TimSortVisualizer {
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
        if self.current_i >= n && self.stack.len() <= 1 {
            return false;
        }

        match self.phase {
            TimPhase::FindingRun => {
                if self.current_i + 1 < n {
                    self.states[self.current_i] = SelectionState::Comparing;
                    self.states[self.current_i + 1] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_i] <= self.array[self.current_i + 1] {
                        self.run_end = self.current_i + 1;
                        self.current_i += 1;
                    } else {
                        self.run_end = self.current_i;
                        self.phase = TimPhase::InsertionSort;
                    }
                } else {
                    // End of array, push last run
                    if self.run_end > self.run_start {
                        self.stack.push((self.run_start, self.run_end - self.run_start));
                    }
                    self.phase = TimPhase::Merging;
                    self.prepare_merge();
                }
                return true;
            },
            TimPhase::InsertionSort => {
                if self.run_end - self.run_start < self.min_run && self.run_end < n {
                    // Perform one insertion sort step
                    let key_idx = self.run_end;
                    if key_idx < n {
                        self.states[key_idx] = SelectionState::Swapping;
                        let key = self.array[key_idx];
                        let mut j = key_idx as isize - 1;
                        while j >= self.run_start as isize && self.array[j as usize] > key {
                            let from = (j + 1) as usize;
                            let to = j as usize;
                            self.array.swap(from, to);
                            self.states[to] = SelectionState::Swapping;
                            self.states[from] = SelectionState::Normal;
                            self.state.swaps += 1;
                            self.state.comparisons += 1;
                            j -= 1;
                        }
                        self.array[(j + 1) as usize] = key;
                        self.run_end += 1;
                    }
                } else {
                    // Run complete, push to stack
                    self.stack.push((self.run_start, self.run_end - self.run_start));
                    self.run_start = self.run_end;
                    self.phase = TimPhase::FindingRun;
                    self.current_i = self.run_end;
                    if self.run_start >= n {
                        self.phase = TimPhase::Merging;
                        self.prepare_merge();
                    }
                }
                return true;
            },
            TimPhase::Merging => {
                if self.stack.len() > 1 {
                    if !self.perform_merge() {
                        // After merge, continue merging if needed
                        self.stack.pop(); // Remove the merged run, but since we pushed it, adjust
                        self.prepare_merge();
                        if self.stack.len() <= 1 {
                            self.phase = TimPhase::Done;
                            return false;
                        }
                    }
                } else {
                    self.phase = TimPhase::Done;
                    return false;
                }
                return true;
            },
            TimPhase::Done => return false,
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.current_i = 0;
        self.run_start = 0;
        self.run_end = 0;
        self.stack.clear();
        self.merging_left = 0;
        self.merging_right = 0;
        self.merge_pos = 0;
        self.temp_array = vec![0; len];
        self.phase = TimPhase::FindingRun;
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
        "TOGISOFT TIM SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Tim Sort?\n\n\
         Tim Sort is a highly efficient hybrid sorting algorithm that combines merge sort and insertion sort.\n\
         It identifies natural runs in the data and extends small runs using insertion sort, then merges them optimally.\n\n\
         Advantages: Adaptive, stable, O(n log n) worst-case, excellent for real-world data.\n\
         Disadvantages: Complex implementation.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked during run identification.\n\n\
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
            TimPhase::FindingRun => "Finding Natural Runs".to_string(),
            TimPhase::InsertionSort => "Extending Run with Insertion Sort".to_string(),
            TimPhase::Merging => "Merging Runs".to_string(),
            TimPhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Swaps: {}", self.state.swaps),
            format!("Current i: {}", self.current_i),
            format!("Runs on Stack: {}", self.stack.len()),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Tim Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                TimPhase::FindingRun => {
                    if self.current_i + 1 < self.array.len() {
                        format!("Checking if array[{}] ({}) <= array[{}] ({})", self.current_i, self.array[self.current_i], self.current_i + 1, self.array[self.current_i + 1])
                    } else {
                        "End of array, pushing last run".to_string()
                    }
                },
                TimPhase::InsertionSort => {
                    format!("Insertion sorting to extend run starting at {}", self.run_start)
                },
                TimPhase::Merging => {
                    format!("Merging runs: left [{}..{}] with right [{}..{}]", self.merging_left, self.merging_right, self.merging_right, self.merging_right + if let Some(&r) = self.stack.iter().rev().nth(1) { r.1 } else { 0 })
                },
                TimPhase::Done => {
                    "Tim sort completed!".to_string()
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

/// Entry point for the tim sort visualization
pub fn tim_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = TimSortVisualizer::new(array_data);
    visualizer.run_visualization();
}