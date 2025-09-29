mod sort_algorithms;
mod common;
mod search_algorithms;

// Import specific functions from modules
use crate::welcome_banner::print_welcome_banner;
use std::error::Error;
use crate::common::*;
use crate::search_algorithms::{binary_search_visualization, linear_search_visualization};
use crate::sort_algorithms::*;
use crate::sort_algorithms::counting_sort::counting_sort_visualization;

/// Main entry point for the algorithm visualizer application
///
/// This function:
/// 1. Displays a welcome banner
/// 2. Creates an array manager to track arrays
/// 3. Enters a main loop that displays a menu and processes user selections
/// 4. Exits when the user selects the exit option
fn main() -> Result<(), Box<dyn Error>> {

    // Load settings
    let mut settings = Settings::load();

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
                run_sort(&mut array_manager, |array| linear_search_visualization(array));
            },
            3 => {
                // Selection Sort: Visualize the selection sort algorithm
                run_sort(&mut array_manager, |array| binary_search_visualization(array));
            },
            4 => {
                run_sort(&mut array_manager, |array| bubble_sort_visualization(array));
            },
            5 => {
                run_sort(&mut array_manager, |array| bucket_sort_visualization(array));
            },
            6 => {
                run_sort(&mut array_manager, |array| cocktail_sort_visualization(array));
            },
            7 => {
                run_sort(&mut array_manager, |array| comb_sort_visualization(array));
            },
            8 => {
                run_sort(&mut array_manager, |array| counting_sort_visualization(array));
            },
            9 => {
                run_sort(&mut array_manager, |array| gnome_sort_visualization(array));
            },
            10 => {
                run_sort(&mut array_manager, |array| heap_sort_visualization(array));
            },
            11 => {
               run_sort(&mut array_manager, |array| insertion_sort_visualization(array));
            },
            12 => {
                run_sort(&mut array_manager, |array| merge_sort_visualization(array));
            },
            13 => {
                run_sort(&mut array_manager, |array| pancake_sort_visualization(array));
            },
            14 => {
                run_sort(&mut array_manager, |array| quick_sort_visualization(array));
            },
            15 => {
                run_sort(&mut array_manager, |array| radix_sort_visualization(array));
            },
            16 => {
                run_sort(&mut array_manager, |array| selection_sort_visualization(array));
            },
            17 => {
                run_sort(&mut array_manager, |array| shell_sort_visualization(array));
            },
            18 => {
                run_sort(&mut array_manager, |array| tim_sort_visualization(array));
            },
            31 => {
                // Settings: Show and modify settings
                let updated_settings = Settings::show_settings_menu(settings.clone());
                settings = updated_settings;
                settings.save(); // Save immediately after changes
            },
            99 => {
                // Exit the application
                settings.save(); // Save settings on exit
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
