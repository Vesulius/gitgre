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
    current_branch: String,
    searchterm: String,
    all_branches: Vec<String>,
    found_branches: Vec<String>,
    exit: bool,
    selected: bool,
}

impl App {
    pub fn new(branches: Vec<String>, searchterm: Option<String>, current_branch: String) -> Self {
        App {
            index: 0,
            searchterm: searchterm.unwrap_or_default(),
            current_branch,
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
        let current_branch = Title::from(Line::from(vec![format!(
            " {} ",
            self.current_branch.clone()
        )
        .blue()
        .bold()]));
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

        let search_content = Paragraph::new(Line::from(vec![
            Span::from(self.searchterm.clone()),
            Span::styled(" ", Style::default().bg(Color::Gray)),
        ]))
        .block(search_block);

        frame.render_widget(search_content, chunks[0]);

        let branches_block = Block::default()
            .title(
                current_branch
                    .alignment(Alignment::Center)
                    .position(Position::Top),
            )
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
                    visible_branches
                        .push(ListItem::new(Line::from(branch.to_string().on_yellow())));
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
            table[x] = min(top_left + cost, min(table[x - 1] + 1, table[x] + 1));
            top_left = new_top_left;
        }
    }

    // NOTE: currently the algo does not care if letters are next to another.
    //       Improve acc by increasing score if matching letters are in a row?

    let diff = table[table.len() - 1];
    ((1.0 * diff as f32 / text.len() as f32) * 1000.0) as i32
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut current_branch = String::new();

    let output = Command::new("/usr/bin/git").arg("branch").output()?;
    let output_lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split_terminator('\n')
        .filter(|l| {
            if l.starts_with('*') {
                current_branch = l.to_string();
                false
            } else {
                true
            }
        })
        .map(|l| l.trim().to_string())
        .collect();

    if output_lines.is_empty() {
        if current_branch == String::new() {
            println!("You don't have any branches");
        } else {
            println!("You only have the current branch {}", current_branch);
        }
        return Ok(());
    }

    let mut terminal = tui::init()?;
    let mut app = App::new(output_lines, args.get(1).cloned(), current_branch);
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
