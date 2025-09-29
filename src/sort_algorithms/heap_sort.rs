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

// Represents the current phase of the heap sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum HeapPhase {
    BuildingMaxHeap,    // Building the max heap from the array
    HeapifyDown,        // Heapifying down after extraction
    ExtractingMax,      // Extracting the maximum element from the heap
    SwappingRootWithLast, // Swapping the root with the last element in the heap
    Done,               // Sorting is complete
}

// Visualizes the heap sort algorithm step-by-step with interactive controls
pub struct HeapSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., comparing, swapping, sorted)
    intro_text: String,        // Dynamic intro text

    // HeapSort specific fields
    heap_size: usize,          // Current size of the heap (decreases as elements are sorted)
    current_index: usize,      // Current index being processed
    left_child: usize,         // Index of the left child of current_index
    right_child: usize,        // Index of the right child of current_index
    largest: usize,            // Index of the largest element found during heapify
    phase: HeapPhase,          // Current phase of the heap sort algorithm
    build_heap_index: i32,     // Index used during the max heap building phase (i32 to handle negative values)
    extraction_count: usize,   // Number of extractions performed (for teaching questions)
    state: VisualizerState,    // Common visualization state
}

impl HeapSortVisualizer {
    // Initializes a new HeapSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What property does a max heap satisfy?".to_string(),
                options: vec![
                    "Every parent node is greater than or equal to its children".to_string(),
                    "Every child node is greater than its parent".to_string(),
                    "Nodes are sorted in ascending order".to_string(),
                ],
                correct_index: 0,
                explanation: "In a max heap, the parent is always >= its children, ensuring the largest is at the root.".to_string(),
            },
            TeachingQuestion {
                text: "Why do we build a max heap first in Heap Sort?".to_string(),
                options: vec![
                    "To ensure the largest element is at the root for easy extraction".to_string(),
                    "To make the array unsorted".to_string(),
                    "To perform comparisons only".to_string(),
                ],
                correct_index: 0,
                explanation: "Building a max heap positions the largest element at the root, simplifying the sorting process.".to_string(),
            },
            TeachingQuestion {
                text: "What does the heapify down operation ensure?".to_string(),
                options: vec![
                    "The heap property is restored after swapping the root".to_string(),
                    "The array is fully sorted".to_string(),
                    "Elements are bubbled up".to_string(),
                ],
                correct_index: 0,
                explanation: "Heapify down maintains the max heap property by swapping the node down the tree if necessary.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of building a max heap?".to_string(),
                options: vec![
                    "O(n)".to_string(),
                    "O(n log n)".to_string(),
                    "O(log n)".to_string(),
                ],
                correct_index: 0,
                explanation: "Building a max heap from an unsorted array takes O(n) time.".to_string(),
            },
            TeachingQuestion {
                text: "What is the main advantage of Heap Sort?".to_string(),
                options: vec![
                    "It is an in-place sorting algorithm".to_string(),
                    "It is always faster than Quick Sort".to_string(),
                    "It uses extra memory for sorting".to_string(),
                ],
                correct_index: 0,
                explanation: "Heap Sort is an in-place algorithm, meaning it does not require additional memory.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Heap Sort?\n\n\
             Heap Sort utilizes a binary heap data structure. First, it builds a max heap where the largest element is at the root. Then, it repeatedly extracts the max (root), swaps it with the last unsorted element, and heapifies down to restore the heap property.\n\n\
             Advantages: O(n log n) time, in-place.\n\
             Disadvantages: Not stable.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after key phases (build complete, each extraction).\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            intro_text,
            heap_size: len,
            current_index: 0,
            left_child: 0,
            right_child: 0,
            largest: 0,
            phase: if len <= 1 { HeapPhase::Done } else { HeapPhase::BuildingMaxHeap },
            build_heap_index: if len <= 1 { -1 } else { (len / 2) as i32 - 1 },
            extraction_count: 0,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("HeapSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
            this.mark_all_sorted();
        }

        this
    }

    // Main loop: handles rendering, input, and stepping through the sort
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
                                settings.last_visualizer = Some("HeapSort".to_string());
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

impl SortVisualizer for HeapSortVisualizer {
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
            let total_elements = self.array.len() as f64;
            let unsorted_elements = self.heap_size as f64;
            let sorted_elements = total_elements - unsorted_elements;
            (sorted_elements / total_elements * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states except sorted
        for (i, state) in self.states.iter_mut().enumerate() {
            match *state {
                SelectionState::Sorted => {}
                _ => {
                    if i >= self.heap_size {
                        *state = SelectionState::Sorted;
                    } else {
                        *state = SelectionState::Normal;
                    }
                }
            }
        }

        let was_building = self.phase == HeapPhase::BuildingMaxHeap;
        let result = match self.phase {
            HeapPhase::BuildingMaxHeap => {
                if self.build_heap_index >= 0 {
                    self.current_index = self.build_heap_index as usize;
                    // Perform one step of heapify down
                    if !self.heapify_down_step() {
                        // This subtree is done, move to next
                        self.build_heap_index -= 1;
                    }
                    true
                } else {
                    // Max heap built, start extraction phase
                    self.phase = HeapPhase::ExtractingMax;
                    // Teaching: Ask question after build
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.extraction_count % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                    true
                }
            },
            HeapPhase::ExtractingMax => {
                if self.heap_size > 1 {
                    self.phase = HeapPhase::SwappingRootWithLast;
                    true
                } else if self.heap_size == 1 {
                    // Last element
                    if self.array.len() > 0 {
                        self.states[0] = SelectionState::Sorted;
                    }
                    self.heap_size = 0;
                    self.phase = HeapPhase::Done;
                    false
                } else {
                    self.phase = HeapPhase::Done;
                    false
                }
            },
            HeapPhase::SwappingRootWithLast => {
                if self.heap_size > 1 && self.heap_size <= self.array.len() {
                    // Swap root with last element in heap
                    self.states[0] = SelectionState::Swapping;
                    self.states[self.heap_size - 1] = SelectionState::Swapping;
                    self.array.swap(0, self.heap_size - 1);
                    self.state.swaps += 1;
                    // Mark the last element as sorted
                    self.states[self.heap_size - 1] = SelectionState::Sorted;
                    self.heap_size -= 1;
                    // Now heapify down from root
                    self.current_index = 0;
                    self.phase = HeapPhase::HeapifyDown;
                    true
                } else {
                    self.phase = HeapPhase::Done;
                    false
                }
            },
            HeapPhase::HeapifyDown => {
                if self.heapify_down_step() {
                    true // Continue heapifying
                } else {
                    // Heapify complete, extract next max
                    self.phase = HeapPhase::ExtractingMax;
                    self.extraction_count += 1;
                    // Teaching: Ask question after each extraction
                    if self.state.teaching_mode && !self.state.questions.is_empty() {
                        let q_index = self.extraction_count % self.state.questions.len();
                        self.state.ask_question(q_index);
                        return true;
                    }
                    true
                }
            },
            HeapPhase::Done => false,
        };

        result
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.heap_size = len;
        self.current_index = 0;
        self.left_child = 0;
        self.right_child = 0;
        self.largest = 0;
        self.extraction_count = 0;
        self.phase = if len <= 1 { HeapPhase::Done } else { HeapPhase::BuildingMaxHeap };
        self.build_heap_index = if len <= 1 { -1 } else { (len / 2) as i32 - 1 };
        self.state.reset_state();
        self.intro_text = format!(
            "What is Heap Sort?\n\n\
             Heap Sort utilizes a binary heap data structure. First, it builds a max heap where the largest element is at the root. Then, it repeatedly extracts the max (root), swaps it with the last unsorted element, and heapifies down to restore the heap property.\n\n\
             Advantages: O(n log n) time, in-place.\n\
             Disadvantages: Not stable.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after key phases (build complete, each extraction).\n\n\
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
        "TOGISOFT HEAP SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Heap Elements", Color::Cyan),
            ("Parent", Color::Yellow),
            ("Left Child", Color::Blue),
            ("Right Child", Color::AnsiValue(208)),
            ("Largest", Color::White),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Heap Size: {}", self.heap_size),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Swaps: {}", self.state.swaps),
            format!("Phase: {}", match self.phase {
                HeapPhase::BuildingMaxHeap => "Building Max Heap",
                HeapPhase::HeapifyDown => "Heapifying Down",
                HeapPhase::ExtractingMax => "Extracting Maximum",
                HeapPhase::SwappingRootWithLast => "Swapping Root",
                HeapPhase::Done => "Done",
            }),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Heap Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                HeapPhase::BuildingMaxHeap => {
                    if self.build_heap_index >= 0 {
                        format!("Building max heap: heapifying subtree rooted at index {}", self.build_heap_index)
                    } else {
                        "Building max heap completed".to_string()
                    }
                },
                HeapPhase::HeapifyDown => {
                    if self.current_index < self.array.len() && self.largest < self.array.len() {
                        format!("Heapify down from index {} (value: {}), largest so far: {} (value: {})",
                                self.current_index, self.array[self.current_index],
                                self.largest, self.array[self.largest])
                    } else {
                        "Heapifying down...".to_string()
                    }
                },
                HeapPhase::ExtractingMax => {
                    format!("Extracting maximum element from heap (size: {})", self.heap_size)
                },
                HeapPhase::SwappingRootWithLast => {
                    if self.heap_size > 0 && self.heap_size <= self.array.len() {
                        format!("Swapping root (max) with last heap element at index {}", self.heap_size - 1)
                    } else {
                        "Swapping root with last element".to_string()
                    }
                },
                HeapPhase::Done => {
                    "Heap sort completed!".to_string()
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

// Performs a single step of the heapify down operation
// Returns `true` if heapifying should continue, `false` if complete
impl HeapSortVisualizer {
    fn heapify_down_step(&mut self) -> bool {
        let left = 2 * self.current_index + 1;
        let right = 2 * self.current_index + 2;

        // Highlight current node
        if self.current_index < self.array.len() {
            self.states[self.current_index] = SelectionState::CurrentMin;
        }

        self.largest = self.current_index;
        self.left_child = left;
        self.right_child = right;

        // Compare with left child
        if left < self.heap_size && left < self.array.len() {
            self.states[left] = SelectionState::PartitionLeft;
            self.state.comparisons += 1;
            if self.array[left] > self.array[self.largest] {
                self.largest = left;
            }
        }

        // Compare with right child
        if right < self.heap_size && right < self.array.len() {
            self.states[right] = SelectionState::PartitionRight;
            self.state.comparisons += 1;
            if self.array[right] > self.array[self.largest] {
                self.largest = right;
            }
        }

        // Highlight the largest
        if self.largest < self.array.len() {
            self.states[self.largest] = SelectionState::Selected;
        }

        // If largest is not current, swap and continue
        if self.largest != self.current_index {
            self.states[self.current_index] = SelectionState::Swapping;
            self.states[self.largest] = SelectionState::Swapping;
            self.array.swap(self.current_index, self.largest);
            self.state.swaps += 1;
            self.current_index = self.largest;

            // Continue heapifying if we haven't reached a leaf
            let next_left = 2 * self.current_index + 1;
            if next_left < self.heap_size {
                return true; // Continue heapifying
            }
        }

        false // Heapify complete
    }
}

// Entry point for the heap sort visualization
pub fn heap_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = HeapSortVisualizer::new(array_data);
    visualizer.run_visualization();
}