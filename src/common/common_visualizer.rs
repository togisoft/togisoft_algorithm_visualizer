use crossterm::{cursor::MoveTo, style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor}, terminal::{size, Clear, ClearType}, ExecutableCommand, QueueableCommand};
use std::io::{stdout, Write};
use crate::enums::{SelectionState, TeachingQuestion};
use crossterm::event::{poll, read};
use std::time::Duration;

// Common drawing functions
pub struct VisualizerDrawer;

impl VisualizerDrawer {
    // Draws the title
    pub fn draw_title(stdout: &mut std::io::Stdout, title: &str) {
        let (width, _) = size().unwrap();
        let title_x = (width.saturating_sub(title.len() as u16)) / 2;
        stdout.queue(MoveTo(title_x, 1)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(SetBackgroundColor(Color::DarkBlue)).unwrap();
        stdout.queue(Print(title)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws the array as a bar graph
    pub fn draw_array_bars(
        stdout: &mut std::io::Stdout,
        array: &[u32],
        states: &[SelectionState],
        width: u16,
        height: u16,
        array_start_y: usize,
    ) {
        let max_value = *array.iter().max().unwrap_or(&1) as f64;
        let array_len = array.len();
        if array_len == 0 {
            return;
        }
        // Calculate bar sizes
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
        let max_bar_height = (height as usize).saturating_sub(20).min(20);

        for (i, &value) in array.iter().enumerate() {
            let bar_height = ((value as f64 / max_value) * max_bar_height as f64) as usize + 1;
            let x = start_x + i * (bar_width + spacing);
            let (fg_color, bg_color) = Self::get_state_colors(states[i]);
            // Draw the bar from bottom to top
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
            // Draw the value
            let value_str = value.to_string();
            let value_x = x + (bar_width.saturating_sub(value_str.len())) / 2;
            stdout.queue(MoveTo(value_x as u16, (array_start_y + max_bar_height + 1) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(value_str)).unwrap();
            stdout.queue(ResetColor).unwrap();
            // Draw the index
            let index_str = i.to_string();
            let index_x = x + (bar_width.saturating_sub(index_str.len())) / 2;
            stdout.queue(MoveTo(index_x as u16, (array_start_y + max_bar_height + 2) as u16)).unwrap();
            stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
            stdout.queue(Print(index_str)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Returns colors based on state
    pub fn get_state_colors(state: SelectionState) -> (Color, Color) {
        match state {
            SelectionState::Normal => (Color::Cyan, Color::Reset),
            SelectionState::Sorted => (Color::Green, Color::DarkGreen),
            SelectionState::CurrentMin => (Color::Yellow, Color::DarkYellow),
            SelectionState::Comparing => (Color::Magenta, Color::DarkMagenta),
            SelectionState::Selected => (Color::White, Color::DarkBlue),
            SelectionState::Swapping => (Color::Red, Color::DarkRed),
            SelectionState::PartitionLeft | SelectionState::PartitionRight => (Color::Blue, Color::DarkBlue),
        }
    }

    // Draws the legend
    pub fn draw_legend(
        stdout: &mut std::io::Stdout,
        items: &[(&str, Color)],
        width: u16,
        height: u16,
        array_start_y: usize,
    ) {
        let max_bar_height = (height as usize).saturating_sub(20).min(20);
        let legend_y = array_start_y + max_bar_height + 4;
        let legend_width = items.len() * 15;
        let legend_start_x = (width as usize - legend_width) / 2;
        for (i, (label, color)) in items.iter().enumerate() {
            let x = legend_start_x + i * 15;
            stdout.queue(MoveTo(x as u16, legend_y as u16)).unwrap();
            stdout.queue(SetForegroundColor(*color)).unwrap();
            stdout.queue(Print("██")).unwrap();
            stdout.queue(ResetColor).unwrap();
            stdout.queue(Print(format!(" {}", label))).unwrap();
        }
    }

    // Draws the statistics
    pub fn draw_statistics(
        stdout: &mut std::io::Stdout,
        stats: &[String],
        width: u16,
        height: u16,
    ) {
        let stats_y = height.saturating_sub(12);
        for (i, stat) in stats.iter().enumerate() {
            let x = 5 + (i % 3) * 25;
            let y = stats_y + (i / 3) as u16;
            stdout.queue(MoveTo(x as u16, y)).unwrap();
            stdout.queue(SetForegroundColor(Color::Cyan)).unwrap();
            stdout.queue(Print(stat)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
    }

    // Draws the controls
    pub fn draw_controls(
        stdout: &mut std::io::Stdout,
        status: &str,
        controls: &str,
        width: u16,
        height: u16,
    ) {
        let controls_y = height.saturating_sub(4);
        // Status
        stdout.queue(MoveTo(5, controls_y)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        let status_color = match status {
            "COMPLETED!" => Color::Green,
            "RUNNING..." => Color::Yellow,
            "PAUSED" => Color::Red,
            "AWAITING SWAP CONFIRMATION" => Color::Magenta,
            "WAITING FOR QUESTION" => Color::Blue,
            _ => Color::White,
        };
        stdout.queue(SetForegroundColor(status_color)).unwrap();
        stdout.queue(Print(format!("Status: {}", status))).unwrap();
        stdout.queue(ResetColor).unwrap();
        // Controls
        let controls_x = (width.saturating_sub(controls.len() as u16)) / 2;
        stdout.queue(MoveTo(controls_x, controls_y + 1)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print(controls)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws the current operation info
    pub fn draw_operation_info(
        stdout: &mut std::io::Stdout,
        message: &str,
        width: u16,
        height: u16,
        color: Color,
    ) {
        let op_x = (width.saturating_sub(message.len() as u16)) / 2;
        stdout.queue(MoveTo(op_x, height.saturating_sub(6))).unwrap();
        stdout.queue(SetForegroundColor(color)).unwrap();
        if color == Color::Green {
            stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        }
        stdout.queue(Print(message)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }

    // Draws a question
    pub fn draw_question(
        stdout: &mut std::io::Stdout,
        question: &TeachingQuestion,
        width: u16,
        height: u16,
    ) {
        let q_text = format!("QUESTION: {}", question.text);
        let q_x = (width.saturating_sub(q_text.len() as u16)) / 2;
        let q_y = height.saturating_sub(10);
        stdout.queue(MoveTo(q_x, q_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::Magenta)).unwrap();
        stdout.queue(SetAttribute(Attribute::Bold)).unwrap();
        stdout.queue(Print(q_text)).unwrap();
        stdout.queue(ResetColor).unwrap();
        for (i, option) in question.options.iter().enumerate() {
            let opt_text = format!("{}: {}", i + 1, option);
            let opt_x = (width.saturating_sub(opt_text.len() as u16)) / 2;
            let opt_y = q_y + (i as u16 + 1);
            stdout.queue(MoveTo(opt_x, opt_y)).unwrap();
            stdout.queue(SetForegroundColor(Color::White)).unwrap();
            stdout.queue(Print(opt_text)).unwrap();
            stdout.queue(ResetColor).unwrap();
        }
        let inst_y = q_y + (question.options.len() as u16 + 2);
        let inst_x = (width.saturating_sub("Press 1,2, or 3.".len() as u16)) / 2;
        stdout.queue(MoveTo(inst_x, inst_y)).unwrap();
        stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
        stdout.queue(Print("Press 1,2, or 3.")).unwrap();
        stdout.queue(ResetColor).unwrap();
    }
}

// Common function to show the intro screen
pub fn show_intro_screen(intro_text: &str) {
    let mut stdout = stdout();
    let (width, height) = size().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    let lines: Vec<&str> = intro_text.lines().collect();
    let start_y = (height as usize / 2).saturating_sub(lines.len() / 2);
    for (i, line) in lines.iter().enumerate() {
        let x = (width.saturating_sub(line.len() as u16)) / 2;
        stdout.queue(MoveTo(x, (start_y + i) as u16)).unwrap();
        stdout.queue(SetForegroundColor(Color::Yellow)).unwrap();
        stdout.queue(Print(*line)).unwrap();
        stdout.queue(ResetColor).unwrap();
    }
    stdout.flush().unwrap();
    // Wait for any key press
    loop {
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            let _ = read();
            break;
        }
    }
}

// Shows feedback for question answers
pub fn show_question_feedback(
    correct: bool,
    question: &TeachingQuestion,
    answer: usize,
) {
    let mut stdout = stdout();
    let (width, height) = size().unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    let feedback = if correct {
        "Correct! 👍".to_string()
    } else {
        format!(
            "Wrong. Correct answer: {}. Explanation: {}",
            question.options[question.correct_index], question.explanation
        )
    };
    let fb_y = (height / 2) as usize;
    let fb_x = (width.saturating_sub(feedback.len() as u16)) / 2;
    stdout.queue(MoveTo(fb_x, fb_y as u16)).unwrap();
    stdout.queue(SetForegroundColor(if correct { Color::Green } else { Color::Red })).unwrap();
    stdout.queue(Print(feedback)).unwrap();
    stdout.queue(ResetColor).unwrap();
    stdout.queue(MoveTo(5, (fb_y + 2) as u16)).unwrap();
    stdout.queue(SetForegroundColor(Color::DarkGrey)).unwrap();
    stdout.queue(Print("Press any key to continue...")).unwrap();
    stdout.queue(ResetColor).unwrap();
    stdout.flush().unwrap();
    // Wait for any key press
    loop {
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            let _ = read();
            break;
        }
    }
}
