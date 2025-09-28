// Import all the modules that make up the application
mod welcome_banner;
mod menu;
mod array_manager;
mod bubble_sort;
mod dialog;
mod selection_sort;
mod helper;
mod insertion_sort;
mod enums;
mod merge_sort;
mod quick_sort;
mod heap_sort;
mod shell_short;  // Note: This should probably be 'shell_sort' for consistency
mod radix_sort;

// Import specific functions from modules
use crate::welcome_banner::print_welcome_banner;
use std::error::Error;
use crate::array_manager::{array_management_screen, ArrayManager};
use crate::bubble_sort::bubble_sort_visualization;
use crate::heap_sort::heap_sort_visualization;
use crate::helper::run_sort;
use crate::insertion_sort::insertion_sort_visualization;
use crate::menu::print_menu_banner;
use crate::merge_sort::merge_sort_visualization;
use crate::quick_sort::quick_sort_visualization;
use crate::radix_sort::radix_sort_visualization;
use crate::selection_sort::selection_sort_visualization;
use crate::shell_short::shell_sort_visualization;  // Note: This should probably be 'shell_sort'

/// Main entry point for the algorithm visualizer application
///
/// This function:
/// 1. Displays a welcome banner
/// 2. Creates an array manager to track arrays
/// 3. Enters a main loop that displays a menu and processes user selections
/// 4. Exits when the user selects the exit option
fn main() -> Result<(), Box<dyn Error>> {
    // Display the welcome banner
    print_welcome_banner();

    // Create an array manager to track and manage arrays
    let mut array_manager = ArrayManager::new();

    // Main application loop
    loop {
        // Display the menu and get user selection
        let selection = print_menu_banner();

        // Process the user's selection
        match selection {
            1 => {
                // Array Management: Create, select, view, or delete arrays
                array_management_screen(&mut array_manager);
            },
            2 => {
                // Bubble Sort: Visualize the bubble sort algorithm
                run_sort(&mut array_manager, |array| bubble_sort_visualization(array));
            },
            3 => {
                // Selection Sort: Visualize the selection sort algorithm
                run_sort(&mut array_manager, |array| selection_sort_visualization(array));
            },
            4 => {
                // Insertion Sort: Visualize the insertion sort algorithm
                run_sort(&mut array_manager, |array| insertion_sort_visualization(array));
            },
            5 => {
                // Quick Sort: Visualize the quick sort algorithm
                run_sort(&mut array_manager, |array| quick_sort_visualization(array));
            },
            6 => {
                // Merge Sort: Visualize the merge sort algorithm
                run_sort(&mut array_manager, |array| merge_sort_visualization(array));
            },
            7 => {
                // Heap Sort: Visualize the heap sort algorithm
                run_sort(&mut array_manager, |array| heap_sort_visualization(array));
            },
            8 => {
                // Shell Sort: Visualize the shell sort algorithm
                run_sort(&mut array_manager, |array| shell_sort_visualization(array));
            },
            9 => {
                // Radix Sort: Visualize the radix sort algorithm
                run_sort(&mut array_manager, |array| radix_sort_visualization(array));
            },
            10 => {
                // Exit the application
                break;
            }
            _ => {
                // Ignore invalid selections
            }
        }
    }

    // Return success
    Ok(())
}
