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

/// Represents the different phases of the bucket sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum BucketPhase {
    Distributing,  // Distributing elements to buckets
    Sorting,       // Sorting all buckets (one step)
    Collecting,    // Collecting sorted buckets into array
    Done,          // Sorting is complete
}

/// Visualizes the bucket sort algorithm step-by-step with interactive controls
pub struct BucketSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)

    // Bucket Sort specific fields
    buckets: Vec<Vec<u32>>,    // Buckets for distribution
    num_buckets: usize,        // Number of buckets
    max_val: f64,              // Maximum value in array for bucket calculation
    current_i: usize,          // Current index for distribution
    current_pos: usize,        // Current position for collecting
    current_bucket: usize,     // Current bucket being collected
    current_in_bucket: usize,  // Current index within current bucket
    last_idx: usize,           // Last distributed index
    last_bucket: usize,        // Last bucket used
    last_placed: u32,          // Last placed value
    phase: BucketPhase,        // Current phase of the bucket sort algorithm
    state: VisualizerState,    // Common visualization state
}

impl BucketSortVisualizer {
    /// Creates a new BucketSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the key idea behind Bucket Sort?".to_string(),
                options: vec![
                    "Distributing elements into buckets and sorting each bucket individually".to_string(),
                    "Repeatedly swapping adjacent elements".to_string(),
                    "Dividing and conquering the array".to_string(),
                ],
                correct_index: 0,
                explanation: "Bucket Sort distributes elements into equally spaced buckets based on their value range, then sorts each bucket separately.".to_string(),
            },
            TeachingQuestion {
                text: "Bucket Sort works best when:".to_string(),
                options: vec![
                    "The input is uniformly distributed".to_string(),
                    "The array is already sorted".to_string(),
                    "There are few unique elements".to_string(),
                ],
                correct_index: 0,
                explanation: "Bucket Sort is efficient when elements are uniformly distributed across the value range, ensuring even bucket sizes.".to_string(),
            },
            TeachingQuestion {
                text: "The average time complexity of Bucket Sort is:".to_string(),
                options: vec![
                    "O(n + k)".to_string(),
                    "O(n log n)".to_string(),
                    "O(n^2)".to_string(),
                ],
                correct_index: 0,
                explanation: "With k buckets and uniform distribution, Bucket Sort achieves O(n + k) average time complexity.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let max_val = if let Some(&m) = array.iter().max() {
            m as f64
        } else {
            1.0
        };

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            buckets: vec![vec![]; 10],
            num_buckets: 10,
            max_val,
            current_i: 0,
            current_pos: 0,
            current_bucket: 0,
            current_in_bucket: 0,
            last_idx: 0,
            last_bucket: 0,
            last_placed: 0,
            phase: BucketPhase::Distributing,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("BucketSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
            this.mark_all_sorted();
        }

        this
    }

    /// Sorts all buckets using insertion sort and counts operations
    fn sort_all_buckets(&mut self) {
        for bucket in self.buckets.clone().iter_mut() {
            self.insertion_sort_bucket(bucket);
        }
    }

    /// Insertion sort for a single bucket, counting comparisons and swaps
    fn insertion_sort_bucket(&mut self, bucket: &mut Vec<u32>) {
        let m = bucket.len();
        for i in 1..m {
            let key = bucket[i];
            let mut j = i as isize - 1;
            while j >= 0 && bucket[j as usize] > key {
                bucket[(j + 1) as usize] = bucket[j as usize];
                self.state.swaps += 1;
                self.state.comparisons += 1;
                j -= 1;
            }
            if (j + 1) as usize != i {
                bucket[(j + 1) as usize] = key;
                self.state.swaps += 1;
            }
            self.state.comparisons += 1; // For the final comparison
        }
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
                                settings.last_visualizer = Some("BucketSort".to_string());
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

impl SortVisualizer for BucketSortVisualizer {
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
            BucketPhase::Distributing => {
                if self.current_i < n {
                    let idx = self.current_i;
                    self.states[idx] = SelectionState::Comparing;
                    let val = self.array[idx];

                    // FIX: Clamp bucket index to prevent out of bounds access
                    let bucket_idx = if self.max_val > 0.0 {
                        let idx = ((val as f64 / self.max_val) * self.num_buckets as f64).floor() as usize;
                        idx.min(self.num_buckets - 1)  // Ensure index is within valid range
                    } else {
                        0
                    };

                    self.buckets[bucket_idx].push(val);
                    self.state.comparisons += 1;
                    self.last_idx = idx;
                    self.last_bucket = bucket_idx;
                    self.current_i += 1;
                    return true;
                } else {
                    // End of distribution
                    self.sort_all_buckets();
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = 0;
                        self.state.ask_question(q_index);
                        self.phase = BucketPhase::Sorting;
                        return true;
                    } else {
                        self.phase = BucketPhase::Collecting;
                        self.current_pos = 0;
                        self.current_bucket = 0;
                        self.current_in_bucket = 0;
                        return true;
                    }
                }
            },
            BucketPhase::Sorting => {
                self.phase = BucketPhase::Collecting;
                self.current_pos = 0;
                self.current_bucket = 0;
                self.current_in_bucket = 0;
                return true;
            },
            BucketPhase::Collecting => {
                if self.current_bucket < self.num_buckets {
                    if self.current_in_bucket < self.buckets[self.current_bucket].len() {
                        let val = self.buckets[self.current_bucket][self.current_in_bucket];
                        self.array[self.current_pos] = val;
                        self.states[self.current_pos] = SelectionState::Sorted;
                        self.last_placed = val;
                        self.current_pos += 1;
                        self.current_in_bucket += 1;
                        self.state.swaps += 1;
                        return true;
                    } else {
                        self.current_in_bucket = 0;
                        self.current_bucket += 1;
                        return true;
                    }
                } else {
                    self.phase = BucketPhase::Done;
                    return false;
                }
            },
            BucketPhase::Done => return false,
        }
    }

    fn reset(&mut self) {
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; self.array.len()];
        self.buckets = vec![vec![]; self.num_buckets];
        self.current_i = 0;
        self.current_pos = 0;
        self.current_bucket = 0;
        self.current_in_bucket = 0;
        self.last_idx = 0;
        self.last_bucket = 0;
        self.last_placed = 0;
        self.phase = BucketPhase::Distributing;
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
        "TOGISOFT BUCKET SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        "What is Bucket Sort?\n\n\
         Bucket Sort is a distribution sorting algorithm that divides the input into a number of buckets, sorts each bucket individually (often using insertion sort), and then concatenates the buckets.\n\n\
         Advantages: Linear time O(n+k) for uniform distributions.\n\
         Disadvantages: Requires knowing the range of values; performance degrades with uneven distribution.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked after distribution phase.\n\n\
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
            BucketPhase::Distributing => format!("Distributing {}/{}", self.current_i, self.array.len()),
            BucketPhase::Sorting => "Sorting Buckets".to_string(),
            BucketPhase::Collecting => format!("Collecting {}/{}", self.current_pos, self.array.len()),
            BucketPhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Num Buckets: {}", self.num_buckets),
            format!("Max Value: {}", self.max_val as u32),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Swaps: {}", self.state.swaps),
            phase_str,
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Bucket Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                BucketPhase::Distributing => {
                    if self.current_i > 0 && self.last_idx < self.array.len() && self.current_i <= self.array.len() {
                        format!("Distributing array[{}] ({}) to bucket {}", self.last_idx, self.array[self.last_idx], self.last_bucket)
                    } else {
                        "Starting distribution to buckets or preparing to sort".to_string()
                    }
                },
                BucketPhase::Sorting => {
                    "Sorting each bucket using insertion sort".to_string()
                },
                BucketPhase::Collecting => {
                    if self.current_in_bucket == 0 {
                        format!("Starting collection from bucket {}", self.current_bucket)
                    } else {
                        format!("Placed {} at position {}", self.array[self.current_pos - 1], self.current_pos - 1)
                    }
                },
                BucketPhase::Done => {
                    "Bucket sort completed!".to_string()
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

/// Entry point for the bucket sort visualization
pub fn bucket_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = BucketSortVisualizer::new(array_data);
    visualizer.run_visualization();
}