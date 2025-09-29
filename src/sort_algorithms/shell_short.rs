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

/// Represents the different phases of the shell sort algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum ShellPhase {
    StartingGap,        // Starting a new gap size
    InsertionSorting,   // Performing insertion sort with current gap
    ComparingElements,  // Comparing elements during insertion sort
    ShiftingElement,    // Shifting an element to make space
    InsertingElement,   // Inserting an element in its correct position
    GapComplete,        // Completed sorting with current gap
    Done,               // Sorting is complete
}

/// Visualizes the shell sort algorithm step-by-step with interactive controls
pub struct ShellSortVisualizer {
    array: Vec<u32>,           // Current state of the array being sorted
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element
    intro_text: String,        // Dynamic intro text

    // Shell Sort specific fields
    gap: usize,                // Current gap size
    current_group: usize,      // Current group being processed
    current_index: usize,      // Current index being processed
    insertion_index: usize,    // Index where element will be inserted
    comparing_index: usize,    // Index of element being compared
    key: u32,                  // Current element being inserted
    phase: ShellPhase,         // Current phase of the shell sort algorithm
    gap_sequence: Vec<usize>,  // Sequence of gap sizes (Knuth sequence)
    gap_sequence_index: usize, // Index of current gap in the sequence
    state: VisualizerState,    // Common visualization state
}

impl ShellSortVisualizer {
    /// Creates a new ShellSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        let questions = vec![
            TeachingQuestion {
                text: "What is the purpose of gaps in Shell Sort?".to_string(),
                options: vec![
                    "To allow swapping of elements that are far apart, improving on insertion sort".to_string(),
                    "To divide the array into equal halves".to_string(),
                    "To find the minimum element in a subarray".to_string(),
                ],
                correct_index: 0,
                explanation: "Gaps enable comparing and swapping non-adjacent elements, reducing inversions faster than adjacent swaps.".to_string(),
            },
            TeachingQuestion {
                text: "How does Shell Sort relate to Insertion Sort?".to_string(),
                options: vec![
                    "It is insertion sort performed on multiple gap-spaced subarrays".to_string(),
                    "It is a variant of bubble sort".to_string(),
                    "It uses selection sort for gaps".to_string(),
                ],
                correct_index: 0,
                explanation: "Shell Sort applies insertion sort to subarrays separated by gaps, gradually reducing the gap to 1.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Shell Sort?".to_string(),
                options: vec![
                    "O(n log n) in practice, but worst case O(n^2)".to_string(),
                    "Always O(n log n)".to_string(),
                    "O(n^2) always".to_string(),
                ],
                correct_index: 0,
                explanation: "The exact complexity depends on the gap sequence, but good sequences achieve O(n log n) or better.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        // Generate Knuth gap sequence: h = 3h + 1
        let mut gap_sequence = Vec::new();
        let mut gap = 1;
        while gap < len {
            gap_sequence.push(gap);
            gap = gap * 3 + 1;
        }
        gap_sequence.reverse(); // Start with largest gap

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!(
            "What is Shell Sort?\n\n\
             Shell Sort is an optimization of insertion sort that allows the exchange of elements that are far apart.\n\
             It starts with large gaps and reduces them, performing insertion sort on gap-spaced subarrays.\n\n\
             Advantages: Better than O(n^2) in practice, in-place.\n\
             Disadvantages: Not stable, complexity depends on gap sequence.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each gap.\n\n\
             Press any key to continue...",
            if state.teaching_mode { "ON" } else { "OFF" }
        );

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            intro_text,
            gap: if gap_sequence.is_empty() { 1 } else { gap_sequence[0] },
            current_group: 0,
            current_index: 0,
            insertion_index: 0,
            comparing_index: 0,
            key: 0,
            phase: ShellPhase::StartingGap,
            gap_sequence,
            gap_sequence_index: 0,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("ShellSort".to_string());
        settings.save();

        if len <= 1 {
            this.state.mark_completed();
            this.mark_all_sorted();
            this.phase = ShellPhase::Done;
        } else {
            this.current_index = this.gap;
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
                                    "What is Shell Sort?\n\n\
                                     Shell Sort is an optimization of insertion sort that allows the exchange of elements that are far apart.\n\
                                     It starts with large gaps and reduces them, performing insertion sort on gap-spaced subarrays.\n\n\
                                     Advantages: Better than O(n^2) in practice, in-place.\n\
                                     Disadvantages: Not stable, complexity depends on gap sequence.\n\n\
                                     Teaching Mode: {} (Toggle with T). Questions will be asked after each gap.\n\n\
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
                                settings.last_visualizer = Some("ShellSort".to_string());
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

impl SortVisualizer for ShellSortVisualizer {
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
        if self.array.len() <= 1 || self.gap_sequence.is_empty() {
            100.0
        } else {
            let total_gaps = self.gap_sequence.len() as f64;
            let completed_gaps = self.gap_sequence_index as f64;
            let current_gap_progress = if self.gap > 0 {
                (self.current_group as f64) / (self.gap as f64)
            } else {
                0.0
            };

            ((completed_gaps + current_gap_progress) / total_gaps * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states to normal except sorted
        for state in self.states.iter_mut() {
            match *state {
                SelectionState::Sorted => {}
                _ => *state = SelectionState::Normal,
            }
        }

        match self.phase {
            ShellPhase::StartingGap => {
                if self.gap_sequence_index < self.gap_sequence.len() {
                    // Set the current gap
                    self.gap = self.gap_sequence[self.gap_sequence_index];
                    self.current_group = 0;
                    self.current_index = self.gap;
                    self.phase = ShellPhase::InsertionSorting;
                    true
                } else {
                    // All gaps processed, sorting complete
                    self.phase = ShellPhase::Done;
                    false
                }
            },
            ShellPhase::InsertionSorting => {
                if self.current_index < self.array.len() {
                    // Start insertion sort for current element
                    self.key = self.array[self.current_index];
                    self.insertion_index = self.current_index;
                    self.comparing_index = if self.current_index >= self.gap {
                        self.current_index - self.gap
                    } else {
                        0
                    };

                    // Highlight current element
                    self.states[self.current_index] = SelectionState::CurrentMin;

                    if self.insertion_index >= self.gap {
                        self.phase = ShellPhase::ComparingElements;
                    } else {
                        // No need to sort, move to next element
                        self.current_index += 1;

                        // Skip to next element in the same group
                        while self.current_index < self.array.len() &&
                            self.current_index % self.gap != self.current_group {
                            self.current_index += 1;
                        }

                        if self.current_index >= self.array.len() {
                            self.phase = ShellPhase::GapComplete;
                        }
                    }
                    true
                } else {
                    // All elements in this gap processed
                    self.phase = ShellPhase::GapComplete;
                    true
                }
            },
            ShellPhase::ComparingElements => {
                if self.insertion_index >= self.gap && self.comparing_index < self.array.len() {
                    // Highlight elements being compared
                    self.states[self.comparing_index] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.comparing_index] > self.key {
                        // Need to shift this element
                        self.phase = ShellPhase::ShiftingElement;
                    } else {
                        // Found correct position
                        self.phase = ShellPhase::InsertingElement;
                    }
                } else {
                    // Found correct position
                    self.phase = ShellPhase::InsertingElement;
                }
                true
            },
            ShellPhase::ShiftingElement => {
                if self.comparing_index < self.array.len() && self.insertion_index < self.array.len() {
                    // Highlight elements being shifted
                    self.states[self.comparing_index] = SelectionState::Swapping;
                    self.states[self.insertion_index] = SelectionState::Swapping;

                    // Shift element to the right
                    self.array[self.insertion_index] = self.array[self.comparing_index];
                    self.state.swaps += 1;

                    self.insertion_index = self.comparing_index;

                    if self.comparing_index >= self.gap {
                        self.comparing_index -= self.gap;
                        self.phase = ShellPhase::ComparingElements;
                    } else {
                        // Reached the beginning of the group
                        self.phase = ShellPhase::InsertingElement;
                    }
                } else {
                    // Reached the beginning of the group
                    self.phase = ShellPhase::InsertingElement;
                }
                true
            },
            ShellPhase::InsertingElement => {
                if self.insertion_index < self.array.len() {
                    // Highlight position where element will be inserted
                    self.states[self.insertion_index] = SelectionState::Selected;

                    // Insert the key at its correct position
                    self.array[self.insertion_index] = self.key;

                    // Move to next element in the same gap group
                    self.current_index += 1;

                    // Skip to next element in the same group
                    while self.current_index < self.array.len() &&
                        self.current_index % self.gap != self.current_group {
                        self.current_index += 1;
                    }

                    if self.current_index >= self.array.len() {
                        // All elements in this group processed
                        self.phase = ShellPhase::GapComplete;
                    } else {
                        // Process next element
                        self.phase = ShellPhase::InsertionSorting;
                    }
                } else {
                    // All elements in this group processed
                    self.phase = ShellPhase::GapComplete;
                }
                true
            },
            ShellPhase::GapComplete => {
                // Move to next group
                self.current_group += 1;

                if self.current_group < self.gap {
                    // Start next group in the same gap
                    self.current_index = self.current_group + self.gap;
                    self.phase = ShellPhase::InsertionSorting;
                } else {
                    // Move to next gap
                    self.gap_sequence_index += 1;

                    if self.gap_sequence_index < self.gap_sequence.len() {
                        // Start with next gap
                        self.phase = ShellPhase::StartingGap;
                    } else {
                        // All gaps processed, sorting complete
                        self.phase = ShellPhase::Done;
                        return false;
                    }
                }
                true
            },
            ShellPhase::Done => false,
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.current_group = 0;
        self.current_index = 0;
        self.insertion_index = 0;
        self.comparing_index = 0;
        self.key = 0;
        self.gap_sequence_index = 0;

        // Regenerate gap sequence
        let mut gap_sequence = Vec::new();
        let mut gap = 1;
        while gap < len {
            gap_sequence.push(gap);
            gap = gap * 3 + 1;
        }
        gap_sequence.reverse();
        self.gap_sequence = gap_sequence;

        self.gap = if self.gap_sequence.is_empty() { 1 } else { self.gap_sequence[0] };
        self.current_index = self.gap;
        self.phase = ShellPhase::StartingGap;
        self.state.reset_state();
        self.intro_text = format!(
            "What is Shell Sort?\n\n\
             Shell Sort is an optimization of insertion sort that allows the exchange of elements that are far apart.\n\
             It starts with large gaps and reduces them, performing insertion sort on gap-spaced subarrays.\n\n\
             Advantages: Better than O(n^2) in practice, in-place.\n\
             Disadvantages: Not stable, complexity depends on gap sequence.\n\n\
             Teaching Mode: {} (Toggle with T). Questions will be asked after each gap.\n\n\
             Press any key to continue...",
            if self.state.teaching_mode { "ON" } else { "OFF" }
        );
        if len <= 1 {
            self.state.mark_completed();
            self.mark_all_sorted();
            self.phase = ShellPhase::Done;
        }
    }

    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT SHELL SORT VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Gap Group", Color::DarkBlue),
            ("Key", Color::Yellow),
            ("Comparing", Color::Magenta),
            ("Position", Color::White),
            ("Shifting", Color::Red),
            ("Sorted", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            ShellPhase::StartingGap => "Starting Gap",
            ShellPhase::InsertionSorting => "Insertion Sort",
            ShellPhase::ComparingElements => "Comparing",
            ShellPhase::ShiftingElement => "Shifting",
            ShellPhase::InsertingElement => "Inserting",
            ShellPhase::GapComplete => "Gap Complete",
            ShellPhase::Done => "Done",
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Shifts: {}", self.state.swaps),
            format!("Gap: {}", self.gap),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            "✓ Array is now sorted using Shell Sort! Congratulations!".to_string()
        } else {
            match self.phase {
                ShellPhase::StartingGap => {
                    format!("Starting gap-{} insertion sort for group {}", self.gap, self.current_group)
                },
                ShellPhase::InsertionSorting => {
                    if self.current_index < self.array.len() {
                        format!("Gap-{} sort: processing element {} (value: {})",
                                self.gap, self.current_index, self.array[self.current_index])
                    } else {
                        format!("Gap-{} insertion sorting", self.gap)
                    }
                },
                ShellPhase::ComparingElements => {
                    if self.insertion_index < self.array.len() && self.comparing_index < self.array.len() {
                        format!("Comparing key {} with element at {} (value: {})",
                                self.key, self.comparing_index, self.array[self.comparing_index])
                    } else {
                        "Comparing elements...".to_string()
                    }
                },
                ShellPhase::ShiftingElement => {
                    if self.comparing_index < self.array.len() {
                        format!("Shifting element {} (value: {}) {} positions right",
                                self.comparing_index, self.array[self.comparing_index], self.gap)
                    } else {
                        "Shifting element...".to_string()
                    }
                },
                ShellPhase::InsertingElement => {
                    format!("Inserting key {} at position {}", self.key, self.insertion_index)
                },
                ShellPhase::GapComplete => {
                    format!("Gap-{} sorting completed, moving to next gap", self.gap)
                },
                ShellPhase::Done => {
                    "Shell sort completed!".to_string()
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

/// Entry point for the shell sort visualization
pub fn shell_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = ShellSortVisualizer::new(array_data);
    visualizer.run_visualization();
}