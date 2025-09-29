use crate::common::array_manager::ArrayData;
use crate::common::base_visualizer::{SortVisualizer, VisualizerState};
use crate::common::common_visualizer::{show_intro_screen, show_question_feedback, VisualizerDrawer};
use crate::common::enums::{SelectionState, TeachingQuestion};
use crate::common::helper::{cleanup_terminal, randomize_questions};
use crate::common::settings::Settings;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind},
    style::{Color, Print},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen},
    cursor::{MoveTo, Show, Hide},
    ExecutableCommand,
};
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

/// Represents the different phases of the linear search algorithm
#[derive(Clone, Copy, PartialEq)]
pub enum LinearSearchPhase {
    Searching,  // Searching through the array
    Found,      // Target found
    NotFound,   // Target not found
    Done,       // Search is complete
}

/// Visualizes the linear search algorithm step-by-step with interactive controls
pub struct LinearSearchVisualizer {
    array: Vec<u32>,           // Current state of the array
    original_array: Vec<u32>,  // Original array, used for resetting
    states: Vec<SelectionState>, // Visual state of each element (e.g., searching, found)

    // Linear Search specific fields
    target: u32,               // Target value to search for
    intro_text: String,        // Intro text with target
    current_i: usize,          // Current search index
    found_index: Option<usize>, // Index where target was found (if any)
    phase: LinearSearchPhase,  // Current phase of the linear search algorithm
    state: VisualizerState,    // Common visualization state
}

impl LinearSearchVisualizer {
    /// Prompts the user to input the target value for the search
    fn prompt_for_target(stdout: &mut Stdout, array: &[u32]) -> u32 {
        let mut input = String::new();
        let prompt = format!(
            "Enter the target value to search for (e.g., a number in the array: {}): ",
            array.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")
        );

        // Clear the screen once and hide the cursor
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(Hide).unwrap();

        // Write the initial prompt
        stdout.execute(MoveTo(0, 0)).unwrap();
        stdout.execute(Print(&prompt)).unwrap();
        stdout.flush().unwrap();

        loop {
            // Read user input
            if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = read().unwrap() {
                match code {
                    KeyCode::Char(c) if c.is_digit(10) => {
                        input.push(c);
                        // Update the display with the current input
                        stdout.execute(MoveTo(0, 1)).unwrap();
                        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                        stdout.execute(Print(&input)).unwrap();
                        stdout.flush().unwrap();
                    }
                    KeyCode::Backspace => {
                        input.pop(); // Remove the last character
                        // Update the display with the current input
                        stdout.execute(MoveTo(0, 1)).unwrap();
                        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                        stdout.execute(Print(&input)).unwrap();
                        stdout.flush().unwrap();
                    }
                    KeyCode::Enter if !input.is_empty() => {
                        if let Ok(target) = input.parse::<u32>() {
                            // Show cursor and return to normal
                            stdout.execute(Show).unwrap();
                            return target;
                        } else {
                            // Show error message on the next line
                            stdout.execute(MoveTo(0, 1)).unwrap();
                            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                            stdout.execute(Print("Invalid input. Please enter a valid number.")).unwrap();
                            stdout.flush().unwrap();
                            input.clear();
                        }
                    }
                    KeyCode::Esc => {
                        // Show cursor and default to middle element
                        stdout.execute(Show).unwrap();
                        return if !array.is_empty() { array[array.len() / 2] } else { 0 };
                    }
                    _ => {}
                }
            }
        }
    }

    /// Creates a new LinearSearchVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let settings = Settings::load();
        let array = array_data.data.clone();
        let len = array.len();

        // Enable raw mode and prompt for target
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        let target = Self::prompt_for_target(&mut stdout, &array);

        let mut questions = vec![
            TeachingQuestion {
                text: "What is the main characteristic of Linear Search?".to_string(),
                options: vec![
                    "It checks each element sequentially from beginning to end".to_string(),
                    "It divides the search space in half each time".to_string(),
                    "It uses a hash table for lookups".to_string(),
                ],
                correct_index: 0,
                explanation: "Linear Search examines each element in the array one by one until the target is found or the end is reached.".to_string(),
            },
            TeachingQuestion {
                text: "What is the time complexity of Linear Search in the worst case?".to_string(),
                options: vec![
                    "O(n)".to_string(),
                    "O(log n)".to_string(),
                    "O(1)".to_string(),
                ],
                correct_index: 0,
                explanation: "In the worst case, Linear Search must examine all n elements, resulting in O(n) time complexity.".to_string(),
            },
            TeachingQuestion {
                text: "When is Linear Search most efficient?".to_string(),
                options: vec![
                    "For small, unsorted arrays".to_string(),
                    "For large, sorted arrays".to_string(),
                    "When using parallel processing".to_string(),
                ],
                correct_index: 0,
                explanation: "Linear Search is simple and efficient for small datasets or when the array is unsorted and random access is costly.".to_string(),
            },
        ];

        randomize_questions(questions.clone());

        let mut state = VisualizerState::new(questions, Duration::from_millis(settings.speed));
        state.teaching_mode = settings.teaching_mode;

        let intro_text = format!("What is Linear Search?\n\n\
         Linear Search is a simple algorithm that sequentially checks each element in an array\n\
         until it finds the target value or reaches the end of the array.\n\n\
         Target: {}\n\n\
         Advantages: Easy to implement, works on unsorted data.\n\
         Disadvantages: O(n) time complexity in worst case.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked periodically.\n\n\
         Press any key to continue...", target);

        let mut this = Self {
            original_array: array.clone(),
            array,
            states: vec![SelectionState::Normal; len],
            target,
            intro_text,
            current_i: 0,
            found_index: None,
            phase: LinearSearchPhase::Searching,
            state,
        };

        // Set last visualizer
        let mut settings = Settings::load();
        settings.last_visualizer = Some("LinearSearch".to_string());
        settings.save();

        if len == 0 {
            this.state.mark_completed();
        }

        this
    }

    /// Main loop: handles rendering, input, and stepping through the search
    pub fn run_visualization(&mut self) {
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen).unwrap();

        show_intro_screen(&self.intro_text);

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
                                settings.last_visualizer = Some("LinearSearch".to_string());
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

impl SortVisualizer for LinearSearchVisualizer {
    fn get_array(&self) -> &[u32] { &self.array }
    fn get_original_array(&self) -> &[u32] { &self.original_array }
    fn get_states(&self) -> &[SelectionState] { &self.states }
    fn get_comparisons(&self) -> u32 { self.state.comparisons }
    fn get_swaps(&self) -> u32 { self.state.swaps } // Not used for search, but kept for trait
    fn get_speed(&self) -> Duration { self.state.speed }
    fn is_running(&self) -> bool { self.state.is_running }
    fn is_paused(&self) -> bool { self.state.is_paused }
    fn is_completed(&self) -> bool { self.state.completed }
    fn is_teaching_mode(&self) -> bool { self.state.teaching_mode }
    fn get_awaiting_question(&self) -> Option<usize> { self.state.awaiting_question }
    fn get_questions(&self) -> &[TeachingQuestion] { &self.state.questions }

    fn get_progress(&self) -> f64 {
        let total = self.array.len() as f64;
        if total == 0.0 { 100.0 } else {
            ((self.current_i as f64 / total) * 100.0).min(100.0)
        }
    }

    fn step(&mut self) -> bool {
        if self.state.completed || self.state.awaiting_question.is_some() {
            return true;
        }

        // Reset states except found
        for (i, state) in self.states.iter_mut().enumerate() {
            if self.found_index.is_some_and(|found| i == found) {
                *state = SelectionState::Sorted; // Reuse Sorted for Found
            } else if *state != SelectionState::Sorted {
                *state = SelectionState::Normal;
            }
        }

        let n = self.array.len();

        match self.phase {
            LinearSearchPhase::Searching => {
                if self.current_i < n {
                    self.states[self.current_i] = SelectionState::Comparing;
                    self.state.comparisons += 1;

                    if self.array[self.current_i] == self.target {
                        self.found_index = Some(self.current_i);
                        self.phase = LinearSearchPhase::Found;
                        if self.state.teaching_mode && !self.state.questions.is_empty() {
                            let q_index = self.current_i % self.state.questions.len();
                            self.state.ask_question(q_index);
                            return true;
                        }
                        true
                    } else {
                        self.current_i += 1;
                        if self.state.teaching_mode && !self.state.questions.is_empty() && self.current_i % 3 == 0 {
                            let q_index = self.current_i % self.state.questions.len();
                            self.state.ask_question(q_index);
                            return true;
                        }
                        true
                    }
                } else {
                    self.phase = LinearSearchPhase::NotFound;
                    false
                }
            },
            LinearSearchPhase::Found | LinearSearchPhase::NotFound | LinearSearchPhase::Done => false,
        }
    }

    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.current_i = 0;
        self.found_index = None;
        self.phase = LinearSearchPhase::Searching;

        // Prompt for new target
        let mut stdout = stdout();
        let target = Self::prompt_for_target(&mut stdout, &self.array);
        self.target = target;
        self.intro_text = format!("What is Linear Search?\n\n\
         Linear Search is a simple algorithm that sequentially checks each element in an array\n\
         until it finds the target value or reaches the end of the array.\n\n\
         Target: {}\n\n\
         Advantages: Easy to implement, works on unsorted data.\n\
         Disadvantages: O(n) time complexity in worst case.\n\n\
         Teaching Mode: ON (Toggle with T). Questions will be asked periodically.\n\n\
         Press any key to continue...", target);
        self.state.reset_state();
        if len == 0 {
            self.state.mark_completed();
        }
    }

    fn mark_all_sorted(&mut self) {
        // For search, mark found as sorted, others normal
        if let Some(found) = self.found_index {
            self.states[found] = SelectionState::Sorted;
        }
    }

    fn get_title(&self) -> &str {
        "TOGISOFT LINEAR SEARCH VISUALIZER"
    }

    fn get_intro_text(&self) -> &str {
        &self.intro_text
    }

    fn get_legend_items(&self) -> Vec<(&str, Color)> {
        vec![
            ("Normal", Color::Cyan),
            ("Comparing", Color::Yellow),
            ("Found", Color::Green),
        ]
    }

    fn get_statistics_strings(&self) -> Vec<String> {
        let phase_str = match self.phase {
            LinearSearchPhase::Searching => "Searching".to_string(),
            LinearSearchPhase::Found => format!("Found at index {}", self.found_index.unwrap()),
            LinearSearchPhase::NotFound => "Not Found".to_string(),
            LinearSearchPhase::Done => "Done".to_string(),
        };

        vec![
            format!("Array Size: {}", self.array.len()),
            format!("Target: {}", self.target),
            format!("Comparisons: {}", self.state.comparisons),
            format!("Current Index: {}", self.current_i),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.state.teaching_mode { "Teaching: ON".to_string() } else { "Teaching: OFF".to_string() },
        ]
    }

    fn get_current_operation(&self) -> String {
        if self.state.completed {
            if self.found_index.is_some() {
                "✓ Target found at index!".to_string()
            } else {
                "✗ Target not found in the array.".to_string()
            }
        } else {
            match self.phase {
                LinearSearchPhase::Searching => {
                    if self.current_i < self.array.len() {
                        "Comparing with target".to_string()
                    } else {
                        "Reached end of array".to_string()
                    }
                },
                LinearSearchPhase::Found => {
                    "Target found!".to_string()
                },
                LinearSearchPhase::NotFound => {
                    "Target not found after checking all elements.".to_string()
                },
                LinearSearchPhase::Done => {
                    "Linear search completed!".to_string()
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
            "SEARCHING..."
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

/// Entry point for the linear search visualization
pub fn linear_search_visualization(array_data: &ArrayData) {
    let mut visualizer = LinearSearchVisualizer::new(array_data);
    visualizer.run_visualization();
}