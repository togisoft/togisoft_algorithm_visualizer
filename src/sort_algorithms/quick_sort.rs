use crate::array_manager::ArrayData;
use crate::base_visualizer::{SortVisualizer, VisualizerState};
use crate::common_visualizer::{show_intro_screen, show_question_feedback, VisualizerDrawer};
use crate::enums::{SelectionState, TeachingQuestion};
use crate::helper::{cleanup_terminal, randomize_questions};
use crate::settings::Settings;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::Color,
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;

/// Represents the different phases of the quick sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum QuickPhase {
    ChoosingPivot,      // Selecting the pivot element
    PartitioningLeft,   // Moving the left pointer and comparing with pivot
    PartitioningRight,  // Moving the right pointer and comparing with pivot
    SwappingElements,   // Swapping elements at left and right pointers
    SwappingWithPivot,  // Swapping the pivot with its final position
    DonePartition,      // Partitioning is complete
}

/// Visualizes the quick sort algorithm step-by-step with interactive controls
pub struct QuickSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    intro_text: String,        // Dynamic intro text

    // QuickSort specific fields
    stack: Vec<(usize, usize)>, // Stack of (low, high) pairs to process
    low: usize,                // Lower bound of the current subarray
    high: usize,               // Upper bound of the current subarray
    pivot_index: usize,        // Index of the pivot element
    left: usize,               // Left pointer for partitioning
    right: usize,              // Right pointer for partitioning
    phase: QuickPhase,         // Current phase of the quick sort algorithm
    partition_count: usize,    // Number of partitions performed (for teaching questions)
    state: VisualizerState,    // Common visualization state
}

impl QuickSortVisualizer {
    /// Creates a new QuickSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What is the role of the pivot in Quick Sort?".to_string(),
                options: vec![
                    "It separates elements smaller and larger than itself".to_string(),
                    "It is the smallest element".to_string(),
                    "It is always the first element".to_string(),
                ],
                correct_index: 0,
                explanation: "The pivot partitions the array into subarrays of elements less than and greater than the pivot.".to_string(),
            },
            TeachingQuestion {
                text: "Why is Quick Sort often faster than other O(n log n) sorts?".to_string(),
                options: vec![
                    "Good average-case performance due to random partitioning".to_string(),
                    "It uses less memory".to_string(),
                    "It is stable".to_string(),
                ],
                correct_index: 0,
                explanation: "Quick Sort has O(n log n) average time but O(n^2) worst case; choosing good pivots makes it fast in practice.".to_string(),
            },
            TeachingQuestion {
                text: "What happens after partitioning?".to_string(),
                options: vec![
                    "Recurse on subarrays excluding the pivot".to_string(),
                    "The entire array is sorted".to_string(),
                    "Swap all elements".to_string(),
                ],
                correct_index: 0,
                explanation: "After partitioning, the pivot is in its final position, and recursion sorts the left and right subarrays.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Quick Sort?\n\n\
             Quick Sort is a divide-and-conquer algorithm that selects a 'pivot' element and partitions the array around it.\n\
             Elements smaller than the pivot go to the left, larger to the right, then recurse on subarrays.\n\n\
             Advantages: Fast average O(n log n), in-place.\n\
             Disadvantages: Worst case O(n^2) if poor pivots.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each partition.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            intro_text,
            stack: Vec::new(),
            low: 0,
            high: 0,
            pivot_index: 0,
            left: 0,
            right: 0,
            phase: QuickPhase::DonePartition,
            partition_count: 0,
            state,
        };

        if len > 1 {
            this.stack.push((0, len - 1));
            this.low = 0;
            this.high = len - 1;
            this.phase = QuickPhase::ChoosingPivot;
        } else if len == 1 {
            this.states[0] = SelectionState::Sorted;
        }

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("QuickSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
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
                                    "What is Quick Sort?\n\n\
                                     Quick Sort is a divide-and-conquer algorithm that selects a 'pivot' element and partitions the array around it.\n\
                                     Elements smaller than the pivot go to the left, larger to the right, then recurse on subarrays.\n\n\
                                     Advantages: Fast average O(n log n), in-place.\n\
                                     Disadvantages: Worst case O(n^2) if poor pivots.\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each partition.\n\n\
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
                                settings.last_visualizer = Some("QuickSort".to_string());
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

impl SortVisualizer for QuickSortVisualizer {
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

        // Reset states to normal except sorted
        for (i, state) in self.states.iter_mut().enumerate() {
            match *state {
                SelectionState::Sorted => {}
                _ => *state = SelectionState::Normal,
            }
        }

        match self.phase {
            QuickPhase::ChoosingPivot => {
                // Get a new range from the stack
                if let Some((l, h)) = self.stack.pop() {
                    self.low = l;
                    self.high = h;

                    // Single element range, mark as sorted
                    if self.low >= self.high {
                        if self.low == self.high && self.low < self.array.len() {
                            self.states[self.low] = SelectionState::Sorted;
                        }
                        return self.step(); // Move to next range
                    }

                    // Choose pivot (last element)
                    self.pivot_index = self.high;
                    self.states[self.pivot_index] = SelectionState::CurrentMin;

                    // Initialize pointers
                    self.left = self.low;
                    self.right = if self.high > 0 { self.high - 1 } else { 0 };

                    self.phase = QuickPhase::PartitioningLeft;
                } else {
                    // Stack is empty, algorithm is complete
                    return false;
                }
                true
            },
            QuickPhase::PartitioningLeft => {
                if self.left <= self.right {
                    self.states[self.left] = SelectionState::PartitionLeft;
                    self.state.comparisons += 1;

                    // Move left pointer if element is less than or equal to pivot
                    if self.array[self.left] <= self.array[self.pivot_index] {
                        self.left += 1;
                    } else {
                        // Element is greater than pivot, move to right pointer
                        self.phase = QuickPhase::PartitioningRight;
                    }
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::PartitioningRight => {
                if self.left <= self.right {
                    self.states[self.right] = SelectionState::PartitionRight;
                    self.state.comparisons += 1;

                    // Move right pointer if element is greater than pivot
                    if self.array[self.right] > self.array[self.pivot_index] {
                        self.right = if self.right > 0 { self.right - 1 } else { 0 };
                    } else {
                        // Element is less than or equal to pivot, swap with left
                        self.phase = QuickPhase::SwappingElements;
                    }
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::SwappingElements => {
                if self.left <= self.right {
                    self.states[self.left] = SelectionState::Swapping;
                    self.states[self.right] = SelectionState::Swapping;

                    // Swap elements at left and right pointers
                    self.array.swap(self.left, self.right);
                    self.state.swaps += 1;

                    // Move pointers
                    self.left += 1;
                    self.right = if self.right > 0 { self.right - 1 } else { 0 };

                    self.phase = QuickPhase::PartitioningLeft;
                } else {
                    // Pointers crossed, swap pivot with left
                    self.phase = QuickPhase::SwappingWithPivot;
                }
                true
            },
            QuickPhase::SwappingWithPivot => {
                self.states[self.pivot_index] = SelectionState::Swapping;
                self.states[self.left] = SelectionState::Swapping;

                // Swap pivot with left pointer (final position)
                if self.left != self.pivot_index {
                    self.array.swap(self.pivot_index, self.left);
                    self.state.swaps += 1;
                }

                let pivot_final_pos = self.left;
                self.states[pivot_final_pos] = SelectionState::Sorted;

                // Push new subarrays to stack (larger subarray first)
                if pivot_final_pos + 1 <= self.high {
                    self.stack.push((pivot_final_pos + 1, self.high));
                }
                if self.low < pivot_final_pos {
                    self.stack.push((self.low, pivot_final_pos - 1));
                }

                self.partition_count += 1;
                // Teaching: Ask question after each partition
                if self.state.teaching_mode && !self.state.questions.is_empty() {
                    let q_index = self.partition_count % self.state.questions.len();
                    self.state.ask_question(q_index);
                    return true;
                }

                self.phase = QuickPhase::ChoosingPivot;
                true
            },
            QuickPhase::DonePartition => {
                self.phase = QuickPhase::ChoosingPivot;
                true
            },
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.stack = Vec::new();
        self.low = 0;
        self.high = 0;
        self.pivot_index = 0;
        self.left = 0;
        self.right = 0;
        self.partition_count = 0;
        self.phase = QuickPhase::DonePartition;
        self.state.reset_state();
        self.intro_text = format!(
            "What is Quick Sort?\n\n\
             Quick Sort is a divide-and-conquer algorithm that selects a 'pivot' element and partitions the array around it.\n\
             Elements smaller than the pivot go to the left, larger to the right, then recurse on subarrays.\n\n\
             Advantages: Fast average O(n log n), in-place.\n\
             Disadvantages: Worst case O(n^2) if poor pivots.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each partition.\n\n\
             Press any key to continue...",
            if self.state.teaching_mode { "ON" } else { "OFF" }
        );

        if len > 1 {
            self.stack.push((0, len - 1));
            self.low = 0;
            self.high = len - 1;
            self.phase = QuickPhase::ChoosingPivot;
        } else if len == 1 {
            self.states[0] = SelectionState::Sorted;
        }

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
        "TOGISOFT QUICK SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Pivot", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Left Ptr", Color::Blue),
            ("Right Ptr", Color::AnsiValue(208)),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Swaps: {}", self.state.swaps),
            format!("Stack Size: {}", self.stack.len()),
            format!("Partitions: {}", self.partition_count),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Quick Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                QuickPhase::ChoosingPivot => {
                    format!("Partition [{}..{}]: Choosing pivot at index {}",
                            self.low, self.high, self.pivot_index)
                },
                QuickPhase::PartitioningLeft => {
                    if self.left < self.array.len() && self.pivot_index < self.array.len() {
                        format!("Partition [{}..{}]: left={} ({}) <= pivot {}?",
                                self.low, self.high, self.left, self.array[self.left], self.array[self.pivot_index])
                    } else {
                        format!("Partition [{}..{}]: Moving left pointer", self.low, self.high)
                    }
                },
                QuickPhase::PartitioningRight => {
                    if self.right < self.array.len() && self.pivot_index < self.array.len() {
                        format!("Partition [{}..{}]: right={} ({}) > pivot {}?",
                                self.low, self.high, self.right, self.array[self.right], self.array[self.pivot_index])
                    } else {
                        format!("Partition [{}..{}]: Moving right pointer", self.low, self.high)
                    }
                },
                QuickPhase::SwappingElements => {
                    if self.left < self.array.len() && self.right < self.array.len() {
                        format!("Swapping left={} ({}) with right={} ({})",
                                self.left, self.array[self.left], self.right, self.array[self.right])
                    } else {
                        "Swapping elements".to_string()
                    }
                },
                QuickPhase::SwappingWithPivot => {
                    format!("Final swap: pivot at {} with left={}", self.pivot_index, self.left)
                },
                QuickPhase::DonePartition => {
                    format!("Moving to next subarray")
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

/// Entry point for the quick sort visualization
pub fn quick_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = QuickSortVisualizer::new(array_data);
    visualizer.run_visualization();
}