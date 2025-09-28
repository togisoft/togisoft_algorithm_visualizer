use crossterm::style::ResetColor;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io::{stdout, Write};
use crate::array_manager::{ArrayData, ArrayManager};
use crate::dialog::show_no_array_selected;

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
