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

/// Represents the different phases of the merge sort algorithm
#[derive(Clone, Copy, PartialEq)]
enum MergePhase {
    MergePairs,    // Merging pairs of subarrays
    MergingInit,   // Initializing a merge operation
    MergingStep,   // Performing a single merge step
    DoneMerge,     // Merge operation completed
}

/// Visualizes the merge sort algorithm step-by-step with interactive controls
pub struct MergeSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, merging, sorted)
    intro_text: String,        // Dynamic intro text
    temp: Vec<u32>,            // Temporary array used during merging

    // Bottom-up merge sort fields
    current_size: usize,       // Current size of subarrays being merged
    current_pair_start: usize, // Starting index of the current pair of subarrays
    low: usize,                // Lower bound of the current subarray
    high: usize,               // Upper bound of the current subarray
    mid: usize,                // Middle index between two subarrays
    i: usize,                  // Index for the left subarray
    j: usize,                  // Index for the right subarray
    k: usize,                  // Index for the merged array
    phase: MergePhase,         // Current phase of the merge sort algorithm
    merge_count: usize,        // Number of merges performed (for teaching questions)
    state: VisualizerState,    // Common visualization state
}

impl MergeSortVisualizer {
    /// Creates a new MergeSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What is the time complexity of Merge Sort?".to_string(),
                options: vec![
                    "O(n log n)".to_string(),
                    "O(n^2)".to_string(),
                    "O(n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Merge Sort divides the array in half repeatedly and merges, leading to O(n log n) time.".to_string(),
            },
            TeachingQuestion {
                text: "Why is a temporary array used in Merge Sort?".to_string(),
                options: vec![
                    "To merge two sorted subarrays without overwriting".to_string(),
                    "To store the original array".to_string(),
                    "To perform comparisons".to_string(),
                ],
                correct_index: 0,
                explanation: "The temp array holds the merged result temporarily to avoid data loss during merging.".to_string(),
            },
            TeachingQuestion {
                text: "What does the merge step do?".to_string(),
                options: vec![
                    "Combines two sorted subarrays into one sorted subarray".to_string(),
                    "Divides the array".to_string(),
                    "Finds the minimum".to_string(),
                ],
                correct_index: 0,
                explanation: "The merge step takes two sorted halves and produces a single sorted array by comparing elements.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Merge Sort?\n\n\
             Merge Sort is a divide-and-conquer algorithm that recursively divides the array into halves, sorts them, and then merges the sorted halves back together.\n\n\
             Advantages: Stable, O(n log n) time.\n\
             Disadvantages: Requires extra space O(n).\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each merge.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            temp: vec![0; len],
            states: vec![SelectionState::Normal; len],
            intro_text,
            current_size: 1,
            current_pair_start: 0,
            low: 0,
            high: 0,
            mid: 0,
            i: 0,
            j: 0,
            k: 0,
            phase: MergePhase::MergePairs,
            merge_count: 0,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("MergeSort".to_string());
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
                                    "What is Merge Sort?\n\n\
                                     Merge Sort is a divide-and-conquer algorithm that recursively divides the array into halves, sorts them, and then merges the sorted halves back together.\n\n\
                                     Advantages: Stable, O(n log n) time.\n\
                                     Disadvantages: Requires extra space O(n).\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each merge.\n\n\
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
                                settings.last_visualizer = Some("MergeSort".to_string());
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

impl SortVisualizer for MergeSortVisualizer {
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
            let sorted_count = self.states.iter().filter(|&&s| s == SelectionState::Sorted).count() as f64;
            (sorted_count / self.array.len() as f64 * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states except sorted
        for (i, state) in self.states.iter_mut().enumerate() {
            if *state != SelectionState::Sorted {
                *state = SelectionState::Normal;
            }
        }

        match self.phase {
            MergePhase::MergePairs => {
                // Check if we've processed all pairs at current size
                if self.current_pair_start + 2 * self.current_size > self.array.len() {
                    // Double the size for next pass
                    self.current_size *= 2;
                    self.current_pair_start = 0;

                    // Check if we're done
                    if self.current_size > self.array.len() {
                        return false;
                    }
                }

                // Set up indices for the next pair to merge
                let left = self.current_pair_start;
                self.low = left;
                self.mid = (left + self.current_size).saturating_sub(1);
                let right_start = left + self.current_size;
                self.high = ((left + 2 * self.current_size).min(self.array.len())).saturating_sub(1);
                self.i = self.low;
                self.j = right_start;
                self.k = self.low;

                // Copy subarrays to temp for merging
                if self.low <= self.mid {
                    self.temp[self.low..=self.mid].copy_from_slice(&self.array[self.low..=self.mid]);
                }
                if right_start <= self.high {
                    self.temp[right_start..=self.high].copy_from_slice(&self.array[right_start..=self.high]);
                }

                self.phase = MergePhase::MergingInit;
                true
            },
            MergePhase::MergingInit => {
                self.phase = MergePhase::MergingStep;
                true
            },
            MergePhase::MergingStep => {
                // Mark pointers
                if self.i <= self.mid {
                    self.states[self.i] = SelectionState::PartitionLeft;
                }
                if self.j <= self.high {
                    self.states[self.j] = SelectionState::PartitionRight;
                }

                // Check if we've exhausted either subarray
                if self.i > self.mid && self.j > self.high {
                    self.phase = MergePhase::DoneMerge;
                    true
                } else if self.i > self.mid {
                    // Take from right subarray
                    self.array[self.k] = self.temp[self.j];
                    self.state.swaps += 1;
                    self.k += 1;
                    self.j += 1;
                    true
                } else if self.j > self.high {
                    // Take from left subarray
                    self.array[self.k] = self.temp[self.i];
                    self.state.swaps += 1;
                    self.k += 1;
                    self.i += 1;
                    true
                } else {
                    // Compare elements from both subarrays
                    self.state.comparisons += 1;
                    if self.temp[self.i] <= self.temp[self.j] {
                        self.array[self.k] = self.temp[self.i];
                        self.k += 1;
                        self.i += 1;
                    } else {
                        self.array[self.k] = self.temp[self.j];
                        self.k += 1;
                        self.j += 1;
                    }
                    self.state.swaps += 1;
                    true
                }
            },
            MergePhase::DoneMerge => {
                // Mark merged range as sorted
                for idx in self.low..=self.high {
                    self.states[idx] = SelectionState::Sorted;
                }

                self.merge_count += 1;
                // Teaching: Ask question after each merge
                if self.state.teaching_mode && !self.state.questions.is_empty() {
                    let q_index = self.merge_count % self.state.questions.len();
                    self.state.ask_question(q_index);
                    return true;
                }

                // Move to next pair
                self.current_pair_start += 2 * self.current_size;
                self.phase = MergePhase::MergePairs;
                true
            },
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.temp = vec![0; len];
        self.states = vec![SelectionState::Normal; len];
        self.current_size = 1;
        self.current_pair_start = 0;
        self.low = 0;
        self.high = 0;
        self.mid = 0;
        self.i = 0;
        self.j = 0;
        self.k = 0;
        self.merge_count = 0;
        self.phase = MergePhase::MergePairs;
        self.state.reset_state();
        self.intro_text = format!(
            "What is Merge Sort?\n\n\
             Merge Sort is a divide-and-conquer algorithm that recursively divides the array into halves, sorts them, and then merges the sorted halves back together.\n\n\
             Advantages: Stable, O(n log n) time.\n\
             Disadvantages: Requires extra space O(n).\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each merge.\n\n\
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
        "TOGISOFT MERGE SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Merging L", Color::Blue),
            ("Merging R", Color::AnsiValue(208)),
            ("Comparing", Color::Magenta),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Moves: {}", self.state.swaps),
            format!("Subarray Size: {}", self.current_size),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Merge Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                MergePhase::MergePairs => {
                    format!("Starting pass: merging subarrays of size {}", self.current_size)
                },
                MergePhase::MergingInit => {
                    format!("Initializing merge [{}..{}] + [{}..{}]",
                            self.low, self.mid, self.mid + 1, self.high)
                },
                MergePhase::MergingStep => {
                    let left_val = if self.i <= self.mid { self.temp[self.i] } else { 0 };
                    let right_val = if self.j <= self.high { self.temp[self.j] } else { 0 };
                    format!("Merging: left[{}]={:?} vs right[{}]={:?} -> pos {}",
                            self.i.saturating_sub(self.low), left_val, self.j.saturating_sub(self.mid + 1), right_val, self.k)
                },
                MergePhase::DoneMerge => {
                    format!("Merge complete for [{}..{}]", self.low, self.high)
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

/// Entry point for the merge sort visualization
pub fn merge_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = MergeSortVisualizer::new(array_data);
    visualizer.run_visualization();
}