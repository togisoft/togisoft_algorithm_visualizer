use crossterm::style::ResetColor;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io::{stdout, Write};
use rand::prelude::SliceRandom;
use crate::common::array_manager::{ArrayData, ArrayManager};
use crate::common::dialog::show_no_array_selected;
use crate::common::enums::TeachingQuestion;

/// Executes a sorting function on the currently selected array in the manager.
///
/// # Arguments
/// * `array_manager` - A mutable reference to the array manager containing the arrays.
/// * `sort_fn` - A closure or function that performs the sorting on the selected array.
///
/// # Behavior
/// - If an array is selected, applies the sorting function to it.
/// - If no array is selected, shows a dialog informing the user to select an array first.
pub fn run_sort<F>(array_manager: &mut ArrayManager, mut sort_fn: F)
where
    F: FnMut(&mut ArrayData),
{
    // Check if an array is selected
    if let Some(array) = array_manager.get_selected_array_mut() {
        // Apply the sorting function to the selected array
        sort_fn(array);
    } else {
        // Show a dialog if no array is selected
        show_no_array_selected();
    }
}

/// Restores the terminal to its original state after a visualization or interactive session.
///
/// # Effects
/// - Resets all terminal colors and styles.
/// - Exits the alternate screen (if one was entered).
/// - Disables raw mode (restores normal terminal input handling).
/// - Flushes the output to ensure all changes are applied.
pub fn cleanup_terminal() {
    let mut stdout = stdout();

    // Reset all terminal colors and styles
    stdout.execute(ResetColor).unwrap();

    // Exit the alternate screen (if one was entered)
    stdout.execute(LeaveAlternateScreen).unwrap();

    // Disable raw mode (restore normal terminal input handling)
    disable_raw_mode().unwrap();

    // Flush the output to ensure all changes are applied
    stdout.flush().unwrap();
}


// Function to randomize the position of the correct answer for each question
pub fn randomize_questions(mut questions: Vec<TeachingQuestion>) -> Vec<TeachingQuestion> {
    let mut rng = rand::rng();

    for question in &mut questions {
        // Assume the correct answer is originally at index 0 (you can adjust if different)
        let correct_text = question.options[0].clone();

        // Shuffle the options
        question.options.shuffle(&mut rng);

        // Find the new index of the correct answer after shuffling
        if let Some(new_index) = question.options.iter().position(|opt| opt == &correct_text) {
            question.correct_index = new_index;
        } else {
            // Fallback if not found (shouldn't happen)
            question.correct_index = 0;
        }
    }

    // Shuffle the order of questions
    questions.shuffle(&mut rng);

    // Print the questions and their details
    for (i, question) in questions.iter().enumerate() {
        println!("Question {}: {}", i + 1, question.text);
        for (j, option) in question.options.iter().enumerate() {
            println!("  {}. {}", j + 1, option);
        }
        println!("  Correct answer: {}", question.correct_index + 1);
        println!("  Explanation: {}\n", question.explanation);
    }

    questions
}