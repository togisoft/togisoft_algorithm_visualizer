use crate::enums::{SelectionState, TeachingQuestion};
use std::time::Duration;

// Base trait that all visualizers must implement
pub trait SortVisualizer {
    // Returns the current state of the array
    fn get_array(&self) -> &[u32];

    // Returns the original array
    fn get_original_array(&self) -> &[u32];

    // Returns the selection states
    fn get_states(&self) -> &[SelectionState];

    // Returns statistics
    fn get_comparisons(&self) -> u32;
    fn get_swaps(&self) -> u32;
    fn get_speed(&self) -> Duration;

    // Returns status information
    fn is_running(&self) -> bool;
    fn is_paused(&self) -> bool;
    fn is_completed(&self) -> bool;
    fn is_teaching_mode(&self) -> bool;

    // Returns question information
    fn get_awaiting_question(&self) -> Option<usize>;
    fn get_questions(&self) -> &[TeachingQuestion];

    // Returns the progress percentage
    fn get_progress(&self) -> f64;

    // Advances one step
    fn step(&mut self) -> bool;

    // Resets the visualizer
    fn reset(&mut self);

    // Marks all elements as sorted
    fn mark_all_sorted(&mut self);

    // Returns a visualizer-specific title
    fn get_title(&self) -> &str;

    // Returns a visualizer-specific intro text
    fn get_intro_text(&self) -> &str;

    // Returns visualizer-specific legend items
    fn get_legend_items(&self) -> Vec<(&str, crossterm::style::Color)>;

    // Returns the current operation description
    fn get_current_operation(&self) -> String;

    // Returns the status message
    fn get_status(&self) -> &str {
        if self.is_completed() {
            "COMPLETED!"
        } else if self.is_running() && !self.is_paused() {
            "RUNNING..."
        } else if self.is_paused() {
            "PAUSED"
        } else if self.get_awaiting_question().is_some() {
            "WAITING FOR QUESTION"
        } else {
            "READY"
        }
    }

    // Returns the controls text
    fn get_controls_text(&self) -> &str {
        if self.get_awaiting_question().is_some() {
            "1,2,3: Answer | ESC: Exit"
        } else if self.is_completed() {
            "SPACE: Restart | R: Reset | T: Teaching Toggle | ESC: Exit"
        } else {
            "SPACE: Start/Pause | S: Step | R: Reset | T: Teaching | +/-: Speed | ESC: Exit"
        }
    }

    // Returns statistics as strings
    fn get_statistics_strings(&self) -> Vec<String> {
        vec![
            format!("Array Size: {}", self.get_array().len()),
            format!("Comparisons: {}", self.get_comparisons()),
            format!("Swaps: {}", self.get_swaps()),
            format!("Speed: {}ms", self.get_speed().as_millis()),
            format!("Progress: {:.1}%", self.get_progress()),
            if self.is_teaching_mode() {
                "Teaching: ON"
            } else {
                "Teaching: OFF"
            }
                .to_string(),
        ]
    }
}

// Common visualizer behaviors
pub struct VisualizerState {
    pub is_running: bool,
    pub is_paused: bool,
    pub completed: bool,
    pub teaching_mode: bool,
    pub speed: Duration,
    pub comparisons: u32,
    pub swaps: u32,
    pub awaiting_question: Option<usize>,
    pub questions: Vec<TeachingQuestion>,
}

impl VisualizerState {
    // Creates a new VisualizerState
    pub fn new(questions: Vec<TeachingQuestion>, default_speed: Duration) -> Self {
        Self {
            is_running: false,
            is_paused: false,
            completed: false,
            teaching_mode: true,
            speed: default_speed,
            comparisons: 0,
            swaps: 0,
            awaiting_question: None,
            questions,
        }
    }

    // Increases the speed
    pub fn increase_speed(&mut self, min_speed: u64) {
        self.speed = Duration::from_millis(
            (self.speed.as_millis() as u64)
                .saturating_sub(50)
                .max(min_speed),
        );
    }

    // Decreases the speed
    pub fn decrease_speed(&mut self, max_speed: u64) {
        self.speed = Duration::from_millis(
            (self.speed.as_millis() as u64 + 50).min(max_speed),
        );
    }

    // Toggles teaching mode
    pub fn toggle_teaching_mode(&mut self) {
        self.teaching_mode = !self.teaching_mode;
    }

    // Toggles play/pause
    pub fn toggle_play_pause(&mut self) {
        if self.is_running {
            self.is_paused = !self.is_paused;
        } else {
            self.is_running = true;
            self.is_paused = false;
        }
    }

    // Resets the state
    pub fn reset_state(&mut self) {
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
        self.comparisons = 0;
        self.swaps = 0;
        self.awaiting_question = None;
    }

    // Marks the process as completed
    pub fn mark_completed(&mut self) {
        self.is_running = false;
        self.completed = true;
    }

    // Asks a question
    pub fn ask_question(&mut self, current_step: usize) {
        if self.teaching_mode && !self.questions.is_empty() {
            let q_index = current_step % self.questions.len();
            self.awaiting_question = Some(q_index);
        }
    }

    // Clears the current question
    pub fn clear_question(&mut self) {
        self.awaiting_question = None;
    }
}
