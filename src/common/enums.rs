/// Represents the visual state of an element in a sorting visualization.
/// Each state can be used to apply different colors or styles to elements
/// during the sorting process, making it easier to track the algorithm's progress.
#[derive(Clone, Copy, PartialEq)]
pub enum SelectionState {
    /// Default state for elements that are not currently involved in any operation.
    Normal,

    /// State for elements that have been sorted and are in their final position.
    Sorted,

    /// State for the current minimum element (e.g., in selection sort).
    CurrentMin,

    /// State for elements that are currently being compared.
    Comparing,

    /// State for the element that is currently selected or highlighted.
    Selected,

    /// State for elements that are being swapped.
    Swapping,

    /// State for elements on the left side of a partition (e.g., in quicksort).
    PartitionLeft,

    /// State for elements on the right side of a partition (e.g., in quicksort).
    PartitionRight,
}


// Simple question structure for teaching
#[derive(Clone)]
pub struct TeachingQuestion {
    pub text: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}