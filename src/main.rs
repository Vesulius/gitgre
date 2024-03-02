mod tui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use std::io;
use std::process::Command;
use std::{cmp::min, env};

#[derive(Debug, Default)]
pub struct App {
    index: u32,
    searchterm: String,
    all_branches: Vec<String>,
    found_branches: Vec<String>,
    exit: bool,
    selected: bool,
}

impl App {
    pub fn new(branches: Vec<String>, searchterm: Option<String>) -> Self {
        App {
            index: 0,
            searchterm: searchterm.unwrap_or_default(),
            all_branches: branches.clone(),
            found_branches: branches,
            exit: false,
            selected: false,
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<bool> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
            self.update_search();
        }
        Ok(self.selected)
    }

    fn render_frame(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(frame.size());

        let title = Title::from(" GITGRE ".bold());
        let instructions = Title::from(Line::from(vec![
            " move up ".into(),
            "<UP>".blue().bold(),
            " move down ".into(),
            "<DOWN>".blue().bold(),
            " select ".into(),
            "<ENTER>".blue().bold(),
        ]));
        let search_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::PLAIN);

        let search_content = Paragraph::new(Text::styled(
            self.searchterm.to_string(),
            Style::default().fg(Color::Blue),
        ))
        .block(search_block);

        frame.render_widget(search_content, chunks[0]);

        let branches_block = Block::default()
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::PLAIN);

        let mut visible_branches = Vec::<ListItem>::new();
        self.found_branches
            .iter()
            .enumerate()
            .for_each(|(ind, branch)| {
                if ind == self.index as usize {
                    visible_branches.push(ListItem::new(Line::from(branch.to_string().on_red())));
                } else {
                    visible_branches.push(ListItem::new(Line::from(branch.to_string())));
                }
            });
        let branches = List::new(visible_branches).block(branches_block);

        frame.render_widget(branches, chunks[1])
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Enter => self.select(),
            KeyCode::Down => self.increment_index(),
            KeyCode::Up => self.decrement_index(),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Char(c) => self.add_char(c),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn update_search(&mut self) {
        if !self.searchterm.is_empty() {
            self.found_branches.clear();
            self.found_branches = self.all_branches.clone();
            self.found_branches.sort_by_key(|b| {
                wagner_fischer(&self.searchterm.chars().collect(), &b.chars().collect())
            });
        }
    }

    fn decrement_index(&mut self) {
        if self.index == 0 {
            self.index = self.found_branches.len() as u32 - 1;
        } else {
            self.index -= 1;
        }
    }

    fn increment_index(&mut self) {
        if self.index == self.found_branches.len() as u32 - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    fn delete_char(&mut self) {
        self.searchterm.pop();
        self.index = 0;
    }

    fn add_char(&mut self, c: char) {
        self.searchterm.push(c);
        self.index = 0;
    }

    fn select(&mut self) {
        self.exit = true;
        self.selected = true;
    }
}

fn wagner_fischer(pattern: &Vec<char>, text: &Vec<char>) -> i32 {
    let mut table: Vec<i32> = (0..=text.len() as i32).collect();
    let mut top_left = 0;
    for y in 1..=pattern.len() {
        for x in 1..=text.len() {
            let cost = if pattern[y - 1] == text[x - 1] { 0 } else { 1 };
            let new_top_left = table[x];
            table[x] = min(top_left + cost, min(table[x - 1] + 1, table[x]) + 1);
            top_left = new_top_left;
        }
    }

    table[table.len() - 1] - text.len() as i32
}

// // wagner_fischer2(pattern: &Vec<char>, text: &Vec<char>) -> u32 {
//     let mut table: Vec<Vec<u32>> = vec![(0..=text.len() as u32).collect(); pattern.len() + 1];
//     for i in 0..=pattern.len() {
//         table[i][0] = i as u32;
//     }
//     for y in 1..=pattern.len() {
//         for x in 1..=text.len() {
//             let cost = if pattern[y - 1] == text[x - 1] { 0 } else { 1 };
//             table[y][x] = min(
//                 table[y - 1][x - 1] + cost,
//                 min(table[y][x - 1] + 1, table[y - 1][x]) + 1,
//             );
//         }
//     }
//     // for t in &table {
//     //     println!("{:?}", t);
//     // }
//     table[table.len() - 1][table[0].len() - 1]
// }

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);

    let output = Command::new("/usr/bin/git").arg("branch").output()?;
    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    let output_lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split_terminator('\n')
        .filter(|l| !l.starts_with('*'))
        .map(|l| l.trim().to_string())
        .collect();

    if output_lines.len() == 1 {
        println!("You only have one branch {}", output_lines[0]);
        return Ok(());
    } else if output_lines.is_empty() {
        println!("You don't have any branches");
        return Ok(());
    }

    let mut terminal = tui::init()?;
    let mut app = App::new(output_lines, args.get(1).cloned());
    let app_result: Result<bool, io::Error> = app.run(&mut terminal);
    tui::restore()?;
    match app_result {
        Ok(selected) => {
            if selected {
                Command::new("/usr/bin/git")
                    .arg("checkout")
                    .arg::<String>(app.found_branches[app.index as usize].clone())
                    .output()?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn render() {
//         let app = App::default();
//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

//         app.render(buf.area, &mut buf);

//         let mut expected = Buffer::with_lines(vec![
//             "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//             "┃                    Value: 0                    ┃",
//             "┃                                                ┃",
//             "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//         ]);
//         let title_style = Style::new().bold();
//         let counter_style = Style::new().yellow();
//         let key_style = Style::new().blue().bold();
//         expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//         expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//         expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//         expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//         expected.set_style(Rect::new(43, 3, 4, 1), key_style);

//         // note ratatui also has an assert_buffer_eq! macro that can be used to
//         // compare buffers and display the differences in a more readable way
//         assert_eq!(buf, expected);
//     }

//     #[test]
//     fn handle_key_event() -> io::Result<()> {
//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Right.into());
//         assert_eq!(app.index, 1);

//         app.handle_key_event(KeyCode::Left.into());
//         assert_eq!(app.index, 0);

//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Char('q').into());
//         assert_eq!(app.exit, true);

//         Ok(())
//     }
// }
