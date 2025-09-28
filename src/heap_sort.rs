use crate::array_manager::ArrayData;
use crate::enums::SelectionState;
use crate::helper::cleanup_terminal;
use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen}, ExecutableCommand,
    QueueableCommand,
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
    comparisons: u32,          // Total number of comparisons made
    swaps: u32,                // Total number of swaps made
    is_running: bool,          // Whether the visualization is running automatically
    is_paused: bool,           // Whether the visualization is paused
    speed: Duration,           // Delay between steps in milliseconds
    completed: bool,           // Whether the sorting is complete

    // HeapSort specific fields
    heap_size: usize,          // Current size of the heap (decreases as elements are sorted)
    current_index: usize,      // Current index being processed
    left_child: usize,         // Index of the left child of current_index
    right_child: usize,        // Index of the right child of current_index
    largest: usize,            // Index of the largest element found during heapify
    phase: HeapPhase,          // Current phase of the heap sort algorithm
    build_heap_index: i32,     // Index used during the max heap building phase (i32 to handle negative values)
}

impl HeapSortVisualizer {
    // Initializes a new HeapSortVisualizer with the given array
    pub fn new(array_data: &ArrayData) -> Self {
        let array = array_data.data.clone();
        let len = array.len();

        // If the array has 0 or 1 elements, it's already sorted
        if len <= 1 {
            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Sorted; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: true,
                heap_size: len,
                current_index: 0,
                left_child: 0,
                right_child: 0,
                largest: 0,
                phase: HeapPhase::Done,
                build_heap_index: -1,
            }
        } else {
            // Start with the building max heap phase
            Self {
                original_array: array.clone(),
                array,
                states: vec![SelectionState::Normal; len],
                comparisons: 0,
                swaps: 0,
                is_running: false,
                is_paused: false,
                speed: Duration::from_millis(800),
                completed: false,
                heap_size: len,
                current_index: 0,
                left_child: 0,
                right_child: 0,
                largest: 0,
                phase: HeapPhase::BuildingMaxHeap,
                build_heap_index: (len / 2) as i32 - 1, // Start from the last non-leaf node
            }
        }
    }

    // Main loop: handles rendering, input, and stepping through the sort
    pub fn run_visualization(&mut self) {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(EnterAlternateScreen).unwrap();

        loop {
            self.draw(&mut stdout);

            // --- Handle Input ---
            if poll(Duration::from_millis(50)).unwrap_or(false) {
                match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Char(' ') => {
                                // Space: Toggle play/pause or restart if completed
                                if self.completed {
                                    self.reset();
                                } else if self.is_running {
                                    self.is_paused = !self.is_paused;
                                } else {
                                    self.is_running = true;
                                    self.is_paused = false;
                                }
                            },
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                // R: Reset the visualization
                                self.reset();
                            },
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // S: Step through the sort manually
                                if !self.completed && !self.is_running {
                                    self.step();
                                }
                            },
                            KeyCode::Char('+') => {
                                // +: Increase speed (decrease delay)
                                self.speed = Duration::from_millis(
                                    (self.speed.as_millis() as u64).saturating_sub(50).max(100)
                                );
                            },
                            KeyCode::Char('-') => {
                                // -: Decrease speed (increase delay)
                                self.speed = Duration::from_millis(
                                    (self.speed.as_millis() as u64 + 100).min(3000)
                                );
                            },
                            KeyCode::Esc => {
                                // ESC: Exit the visualization
                                cleanup_terminal();
                                return;
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // --- Auto Step ---
            // Automatically step if running and not paused
            if self.is_running && !self.is_paused && !self.completed {
                std::thread::sleep(self.speed);
                if !self.step() {
                    self.is_running = false;
                    self.completed = true;
                    self.mark_all_sorted();
                }
            }
        }
    }

    // Renders the current state of the visualization to the terminal
    fn draw(&mut self, stdout: &mut std::io::Stdout) {
        let (width, height) = size().unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();

        // --- Title ---
        let title = "TOGISOFT HEAP SORT VISUALIZER";
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, 1)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();

        // --- Array Visualization ---
        self.draw_array(stdout, width, height);

        // --- Statistics ---
        self.draw_statistics(stdout, width, height);

        // --- Controls ---
        self.draw_controls(stdout, width, height);

        // --- Current Operation Info ---
        self.draw_current_operation(stdout, width, height);

        // --- Heap Structure Visualization ---
        self.draw_heap_structure(stdout, width, height);

        stdout.flush().unwrap();
    }

    // Draws the array as a series of bars, with colors indicating their state
    fn draw_array(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let array_start_y = 5;
        let max_value = *self.array.iter().max().unwrap_or(&1) as f64;
        let array_len = self.array.len();
        if array_len == 0 {
            return;
        }

        // Calculate bar dimensions and spacing
        let available_width = (width as usize).saturating_sub(4);
        let bar_width = if available_width / array_len >= 3 {
            3
        } else if available_width / array_len >= 2 {
            2
        } else {
            1
        };
        let spacing = if bar_width >= 2 { 1 } else { 0 };
        let total_width_needed = array_len * bar_width + (array_len - 1) * spacing;
        let start_x = (width as usize - total_width_needed) / 2;

        // Draw bars
        let max_bar_height = (height as usize).saturating_sub(20).min(15);
        for (i, &value) in self.array.iter().enumerate() {
            let bar_height = ((value as f64 / max_value) * max_bar_height as f64) as usize + 1;
            let x = start_x + i * (bar_width + spacing);

            // Choose color based on element state
            let (fg_color, bg_color) = match self.states[i] {
                SelectionState::Normal => {
                    if i >= self.heap_size {
                        (Color::DarkGrey, Color::Reset) // Out of heap (sorted)
                    } else {
                        (Color::Cyan, Color::Reset) // In heap
                    }
                },
                SelectionState::Sorted => (Color::Green, Color::DarkGreen),
                SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow), // Root/Parent
                SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta), // Children being compared
                SelectionState::Selected => (Color::White, Color::DarkBlue), // Largest found
                SelectionState::Swapping => (Color::Red, Color::DarkRed),
                SelectionState::PartitionLeft => (Color::Blue, Color::DarkBlue), // Left child
                SelectionState::PartitionRight => (Color::AnsiValue(208), Color::DarkBlue), // Right child
            };

            // Draw the bar from bottom up
            for h in 0..bar_height {
                let y = array_start_y + max_bar_height - h;
                stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
                stdout.queue(SetForegroundColor(fg_color)).unwrap();
                stdout.queue(SetBackgroundColor(bg_color)).unwrap();
                if bar_width == 1 {
                    stdout.queue(Print("█")).unwrap();
                } else {
                    stdout.queue(Print("█".repeat(bar_width))).unwrap();
                }
                stdout.queue(ResetColor).unwrap();
            }

            // Draw value below bar
            let value_str = value.to_string();
            let value_x = x + (bar_width.saturating_sub(value_str.len())) / 2;
            stdout.queue(MoveTo(value_x as u16, (array_start_y + max_bar_height + 1) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(value_str)).unwrap();
            stdout.queue(ResetColor).unwrap();

            // Draw index below value
            let index_str = i.to_string();
            let index_x = x + (bar_width.saturating_sub(index_str.len())) / 2;
            stdout.queue(MoveTo(index_x as u16, (array_start_y + max_bar_height + 2) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(index_str)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }

        // --- Legend ---
        let legend_y = array_start_y + max_bar_height + 4;
        let legend_items = [
            ("Heap", Color::Cyan),
            ("Parent", Color::Yellow),
            ("Left Child", Color::Blue),
            ("Right Child", Color::AnsiValue(208)),
            ("Largest", Color::White),
            ("Swapping", Color::Red),
            ("Sorted", Color::Green),
        ];
        let legend_start_x = (width as usize - 100) / 2;
        for (i, (label, color)) in legend_items.iter().enumerate() {
            let x = legend_start_x + (i % 4) * 22;
            let y = legend_y + (i / 4);
            stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
            stdout.queue(SetForegroundColor(*color)).unwrap();
            stdout.queue(Print("██")).unwrap();
            stdout.queue(ResetColor).unwrap();
            stdout.queue(Print(format!(" {}", label))).unwrap();
        }
    }

    // Draws information about the current heap structure
    fn draw_heap_structure(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        if self.heap_size == 0 {
            return;
        }

        let tree_y = height.saturating_sub(12);
        let tree_info = format!("Heap Size: {} | Current: {} | Left Child: {} | Right Child: {} | Largest: {}",
                                self.heap_size,
                                if self.current_index < self.array.len() { self.current_index.to_string() } else { "N/A".to_string() },
                                if self.left_child < self.heap_size { self.left_child.to_string() } else { "N/A".to_string() },
                                if self.right_child < self.heap_size { self.right_child.to_string() } else { "N/A".to_string() },
                                if self.largest < self.array.len() { self.largest.to_string() } else { "N/A".to_string() });

        let info_x = (width.saturating_sub(tree_info.len() as u16)) / 2;
        stdout.queue(MoveTo(info_x, tree_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(tree_info)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws statistics such as comparisons, swaps, and progress
    fn draw_statistics(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let stats_y = height.saturating_sub(10);
        let phase_str = match self.phase {
            HeapPhase::BuildingMaxHeap => "Building Max Heap",
            HeapPhase::HeapifyDown => "Heapifying Down",
            HeapPhase::ExtractingMax => "Extracting Maximum",
            HeapPhase::SwappingRootWithLast => "Swapping Root",
            HeapPhase::Done => "Done",
        };

        let stats = [
            format!("Array Size: {}", self.array.len()),
            format!("Comparisons: {}", self.comparisons),
            format!("Swaps: {}", self.swaps),
            format!("Speed: {}ms", self.speed.as_millis()),
            format!("Phase: {}", phase_str),
            format!("Progress: {:.1}%", self.get_progress()),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let x = 5 + (i % 3) * 30;
            let y = stats_y + (i / 3) as u16;
            stdout.queue(MoveTo(x as u16, y as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
            stdout.queue(Print(stat)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Draws the control instructions and current status
    fn draw_controls(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        let controls_y = height.saturating_sub(4);
        let status = if self.completed {
            "COMPLETED!"
        } else if self.is_running && !self.is_paused {
            "RUNNING..."
        } else if self.is_paused {
            "PAUSED"
        } else {
            "READY"
        };

        // Status
        stdout.queue(MoveTo(5, controls_y)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        let status_color = match status {
            "COMPLETED!" => Color::Green,
            "RUNNING..." => Color::Yellow,
            "PAUSED" => Color::Red,
            _ => Color::White,
        };
        stdout.queue(SetForegroundColor(status_color)).unwrap();
        stdout.queue(Print(format!("Status: {}", status))).unwrap();
        stdout.queue(ResetColor).unwrap();

        // Controls
        let controls = if self.completed {
            "SPACE: Restart | R: Reset | ESC: Exit"
        } else {
            "SPACE: Start/Pause | S: Step | R: Reset | +/-: Speed | ESC: Exit"
        };
        let controls_x = (width.saturating_sub(controls.len() as u16)) / 2;
        stdout.queue(MoveTo(controls_x, controls_y + 1)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(controls)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws information about the current operation
    fn draw_current_operation(&self, stdout: &mut std::io::Stdout, width: u16, height: u16) {
        if self.completed {
            let message = "✓ Array is now sorted using Heap Sort!";
            let message_x = (width.saturating_sub(message.len() as u16)) / 2;
            stdout.queue(MoveTo(message_x, height.saturating_sub(8))).unwrap();
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
            stdout.queue(SetForegroundColor(Color::Green)).unwrap();
            stdout.queue(Print(message)).unwrap();
            stdout.queue(ResetColor).unwrap();
        } else {
            let operation = match self.phase {
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
            };

            let op_x = (width.saturating_sub(operation.len() as u16)) / 2;
            stdout.queue(MoveTo(op_x, height.saturating_sub(8))).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(operation)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Performs a single step of the heap sort algorithm
    // Returns `false` if sorting is complete, `true` otherwise
    fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        // Reset states to normal except sorted
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

        match self.phase {
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
                    self.swaps += 1;
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
                    true
                }
            },
            HeapPhase::Done => false,
        }
    }

    // Performs a single step of the heapify down operation
    // Returns `true` if heapifying should continue, `false` if complete
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
            self.comparisons += 1;
            if self.array[left] > self.array[self.largest] {
                self.largest = left;
            }
        }

        // Compare with right child
        if right < self.heap_size && right < self.array.len() {
            self.states[right] = SelectionState::PartitionRight;
            self.comparisons += 1;
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
            self.swaps += 1;
            self.current_index = self.largest;

            // Continue heapifying if we haven't reached a leaf
            let next_left = 2 * self.current_index + 1;
            if next_left < self.heap_size {
                return true; // Continue heapifying
            }
        }

        false // Heapify complete
    }

    // Resets the visualization to its initial state
    fn reset(&mut self) {
        let len = self.original_array.len();
        self.array = self.original_array.clone();
        self.states = vec![SelectionState::Normal; len];
        self.comparisons = 0;
        self.swaps = 0;
        self.is_running = false;
        self.is_paused = false;
        self.completed = false;
        self.heap_size = len;
        self.current_index = 0;
        self.left_child = 0;
        self.right_child = 0;
        self.largest = 0;

        if len <= 1 {
            self.completed = true;
            self.states = vec![SelectionState::Sorted; len];
            self.phase = HeapPhase::Done;
            self.build_heap_index = -1;
        } else {
            self.phase = HeapPhase::BuildingMaxHeap;
            self.build_heap_index = (len / 2) as i32 - 1;
        }
    }

    // Marks all elements as sorted (used when sorting is complete)
    fn mark_all_sorted(&mut self) {
        for state in &mut self.states {
            *state = SelectionState::Sorted;
        }
    }

    // Calculates the progress of the sorting as a percentage
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
}

// Entry point for the heap sort visualization
pub fn heap_sort_visualization(array_data: &ArrayData) {
    let mut visualizer = HeapSortVisualizer::new(array_data);
    visualizer.run_visualization();
}
