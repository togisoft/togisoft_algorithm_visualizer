use crate::common::base_visualizer::{SortVisualizer, VisualizerState};
use crate::common::common_visualizer::{show_intro_screen, show_question_feedback, VisualizerDrawer};
use crate::common::helper::cleanup_terminal;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;

// General visualizer runner
// Common input handling and render loop for each sorting algorithm
pub fn run_visualizer<V: SortVisualizer>(visualizer: &mut V, state: &mut VisualizerState) {
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();

    // Show intro screen
    show_intro_screen(visualizer.get_intro_text());

    loop {
        // Draw the screen
        draw_screen(&mut stdout, visualizer, state);

        // Process input
        if poll(Duration::from_millis(50)).unwrap_or(false) {
            match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    // Handle question answer if a question is pending
                    if let Some(q_index) = state.awaiting_question {
                        match key_event.code {
                            KeyCode::Char('1') => handle_question_answer(visualizer, state, q_index, 0, &mut stdout),
                            KeyCode::Char('2') => handle_question_answer(visualizer, state, q_index, 1, &mut stdout),
                            KeyCode::Char('3') => handle_question_answer(visualizer, state, q_index, 2, &mut stdout),
                            _ => continue,
                        }
                        continue;
                    }

                    // Handle normal controls
                    match key_event.code {
                        KeyCode::Char(' ') => {
                            if state.completed {
                                visualizer.reset();
                                state.reset_state();
                            } else {
                                state.toggle_play_pause();
                            }
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            visualizer.reset();
                            state.reset_state();
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            if !state.completed && !state.is_running {
                                if !visualizer.step() {
                                    state.mark_completed();
                                    visualizer.mark_all_sorted();
                                }
                            }
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            state.toggle_teaching_mode();
                        }
                        KeyCode::Char('+') => {
                            state.increase_speed(50);
                        }
                        KeyCode::Char('-') => {
                            state.decrease_speed(2000);
                        }
                        KeyCode::Esc => {
                            cleanup_terminal();
                            return;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Auto-step if running and not paused
        if state.is_running && !state.is_paused && !state.completed && state.awaiting_question.is_none() {
            std::thread::sleep(state.speed);
            if !visualizer.step() {
                state.mark_completed();
                visualizer.mark_all_sorted();
            }
        }
    }
}

// Draws the screen
fn draw_screen<V: SortVisualizer>(
    stdout: &mut std::io::Stdout,
    visualizer: &V,
    state: &VisualizerState,
) {
    let (width, height) = size().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();

    // Title
    VisualizerDrawer::draw_title(stdout, visualizer.get_title());

    // Array bars
    VisualizerDrawer::draw_array_bars(
        stdout,
        visualizer.get_array(),
        visualizer.get_states(),
        width,
        height,
        5,
    );

    // Legend
    VisualizerDrawer::draw_legend(
        stdout,
        &visualizer.get_legend_items(),
        width,
        height,
        5,
    );

    // Statistics
    let stats = visualizer.get_statistics_strings();
    VisualizerDrawer::draw_statistics(stdout, &stats, width, height);

    // Controls
    VisualizerDrawer::draw_controls(
        stdout,
        visualizer.get_status(),
        visualizer.get_controls_text(),
        width,
        height,
    );

    // Current operation
    if state.awaiting_question.is_none() {
        let operation = visualizer.get_current_operation();
        let color = if state.completed {
            crossterm::style::Color::Green
        } else {
            crossterm::style::Color::White
        };
        VisualizerDrawer::draw_operation_info(stdout, &operation, width, height, color);
    }

    // Question
    if let Some(q_index) = state.awaiting_question {
        if let Some(question) = state.questions.get(q_index) {
            VisualizerDrawer::draw_question(stdout, question, width, height);
        }
    }

    stdout.flush().unwrap();
}

// Handles question answers
fn handle_question_answer<V: SortVisualizer>(
    visualizer: &V,
    state: &mut VisualizerState,
    q_index: usize,
    answer: usize,
    _stdout: &mut std::io::Stdout,
) {
    if let Some(question) = state.questions.get(q_index) {
        let correct = answer == question.correct_index;
        show_question_feedback(correct, question, answer);
        state.clear_question();
    }
}

// Extendable visualizer runner for special cases
// For example, bubble sort's swap confirmation
pub trait ExtendedVisualizerBehavior {
    // Extra input handling
    fn handle_extra_input(&mut self, key_code: KeyCode) -> bool;

    // Check for extra state
    fn has_extra_state(&self) -> bool;

    // Get extra state message
    fn get_extra_state_message(&self) -> Option<String>;
}
