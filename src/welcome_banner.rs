use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    ExecutableCommand, cursor::MoveTo,
};
use std::io::{stdout, Write};
use std::time::Duration;

fn print_centered_block(block: &str, color: Color, y_offset: u16) {
    let mut stdout = stdout();
    let (width, _) = crossterm::terminal::size().unwrap_or((80, 24));

    for (i, line) in block.lines().enumerate() {
        if !line.trim().is_empty() {
            let line_width = line.chars().count() as u16;
            let x = if width > line_width { (width - line_width) / 2 } else { 0 };

            stdout.execute(MoveTo(x, y_offset + i as u16)).unwrap();
            stdout.execute(SetAttribute(Attribute::Bold)).unwrap();
            stdout.execute(SetForegroundColor(color)).unwrap();
            stdout.execute(Print(line)).unwrap();
            stdout.execute(ResetColor).unwrap();
        }
    }
    stdout.flush().unwrap();
}

const BLOCK1: &str = r#"
████████╗ ██████╗  ██████╗ ██╗███████╗ ██████╗ ███████╗████████╗
╚══██╔══╝██╔═══██╗██╔════╝ ██║██╔════╝██╔═══██╗██╔════╝╚══██╔══╝
   ██║   ██║   ██║██║  ███╗██║███████╗██║   ██║█████╗     ██║
   ██║   ██║   ██║██║   ██║██║╚════██║██║   ██║██╔══╝     ██║
   ██║   ╚██████╔╝╚██████╔╝██║███████║╚██████╔╝██║        ██║
   ╚═╝    ╚═════╝  ╚═════╝ ╚═╝╚══════╝ ╚═════╝ ╚═╝        ╚═╝"#;

const BLOCK2: &str = r#"
█████╗ ██╗      ██████╗  ██████╗ ██████╗ ██╗████████╗██╗  ██╗███╗   ███╗
██╔══██╗██║     ██╔════╝ ██╔═══██╗██╔══██╗██║╚══██╔══╝██║  ██║████╗ ████║
███████║██║     ██║  ███╗██║   ██║██████╔╝██║   ██║   ███████║██╔████╔██║
██╔══██║██║     ██║   ██║██║   ██║██╔══██╗██║   ██║   ██╔══██║██║╚██╔╝██║
██║  ██║███████╗╚██████╔╝╚██████╔╝██║  ██║██║   ██║   ██║  ██║██║ ╚═╝ ██║
╚═╝  ╚═╝╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝     ╚═╝"#;

const BLOCK3: &str = r#"
██╗   ██╗██╗███████╗██╗   ██╗ █████╗ ██╗     ██╗███████╗███████╗██████╗
██║   ██║██║██╔════╝██║   ██║██╔══██╗██║     ██║╚══███╔╝██╔════╝██╔══██╗
██║   ██║██║███████╗██║   ██║███████║██║     ██║  ███╔╝ █████╗  ██████╔╝
╚██╗ ██╔╝██║╚════██║██║   ██║██╔══██║██║     ██║ ███╔╝  ██╔══╝  ██╔══██╗
 ╚████╔╝ ██║███████║╚██████╔╝██║  ██║███████╗██║███████╗███████╗██║  ██║
  ╚═══╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝╚══════╝╚══════╝╚═╝  ╚═╝"#;

const BLOCK4: &str = r#"
┌─────────────────────────────────────────────────────────────────────┐
│      Welcome to TOGISOFT's SPECTACULAR Algorithm Visualizer!        │
│             Watch algorithms dance before your eyes!                │
│                 Beautiful, Colorful, Interactive!                   │
│                  Powered by TOGISOFT Technology                     │
└─────────────────────────────────────────────────────────────────────┘"#;

pub fn print_welcome_banner() {
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.flush().unwrap();

    // Print the banner blocks with proper spacing
    print_centered_block(BLOCK1, Color::Cyan, 2);
    print_centered_block(BLOCK2, Color::Magenta, 9);
    print_centered_block(BLOCK3, Color::Yellow, 16);
    print_centered_block(BLOCK4, Color::Green, 23);

    // Print instruction at the bottom
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    let instruction = "Press ENTER to continue...";
    let x = if width > instruction.len() as u16 {
        (width - instruction.len() as u16) / 2
    } else { 0 };

    stdout.execute(MoveTo(x, height - 5)).unwrap();
    stdout.execute(SetForegroundColor(Color::White)).unwrap();
    stdout.execute(Print(instruction)).unwrap();
    stdout.flush().unwrap();

    // Wait for Enter key press
    loop {
        if poll(Duration::from_millis(100)).unwrap_or(false) {
            match read().unwrap_or(Event::Key(KeyCode::Esc.into())) {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    if key_event.code == KeyCode::Enter {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    stdout.execute(LeaveAlternateScreen).unwrap();
    stdout.flush().unwrap();
}