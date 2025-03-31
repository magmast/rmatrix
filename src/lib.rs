use std::{thread, time::Duration};

use ncurses as nc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

const FPS: u64 = 20;
const MIN_X: i32 = 0;
const MIN_LINE_HEIGHT: i32 = 3;
const MAYBE_CHANCE: i32 = 5;
const COLUMN_STEP: usize = 2;
const MIN_LINES_GAP: i32 = 4;
const GREEN: i16 = 1;
const LIGHT_GREEN: i16 = 2;

pub struct Matrix {
    window: nc::WINDOW,
    columns: Vec<Column>,
}

impl Matrix {
    pub fn new(window: nc::WINDOW) -> Self {
        let max_x = nc::getmaxx(window);
        let max_y = nc::getmaxy(window);

        let columns = (MIN_X..max_x)
            .step_by(COLUMN_STEP)
            .map(|_| Column::new(max_y))
            .collect();

        Self { window, columns }
    }

    pub fn run(&mut self) {
        loop {
            nc::attron(nc::COLOR_PAIR(GREEN));
            nc::werase(self.window);
            nc::attroff(nc::COLOR_PAIR(GREEN));
            self.print();
            self.update();
            nc::wrefresh(self.window);
            thread::sleep(Duration::from_millis(1000 / FPS));
        }
    }

    fn update(&mut self) {
        self.move_lines();
        self.generate_lines();
        self.remove_invisible_lines();
    }

    fn move_lines(&mut self) {
        for column in self.columns.iter_mut() {
            column.move_lines();
        }
    }

    fn remove_invisible_lines(&mut self) {
        for column in self.columns.iter_mut() {
            column.remove_invisible_lines();
        }
    }

    fn generate_lines(&mut self) {
        for column in self.columns.iter_mut() {
            column.generate_line();
        }
    }

    fn print(&self) {
        let max_x = nc::getmaxx(self.window);
        let columns = (MIN_X..max_x).step_by(COLUMN_STEP).zip(&self.columns);
        for (x, column) in columns {
            column.print_at(self.window, x);
        }
    }
}

pub struct Column {
    lines: Vec<Sequence>,
    height: i32,
}

impl Column {
    pub fn new(height: i32) -> Self {
        let lines = if let Some(line) = Sequence::default().maybe() {
            let line = line
                .with_random_content(height as usize)
                .with_random_height(height);
            vec![line]
        } else {
            Vec::new()
        };

        Self { lines, height }
    }

    pub fn move_lines(&mut self) {
        for line in self.lines.iter_mut() {
            line.increment_offset();
        }
    }

    pub fn remove_invisible_lines(&mut self) {
        self.lines = self
            .lines
            .iter()
            .filter(|line| line.is_visible())
            .cloned()
            .collect();
    }

    pub fn generate_line(&mut self) {
        if let Some(line) = Sequence::default().maybe() {
            if self.lines.iter().any(|line| line.offset() < MIN_LINES_GAP) {
                return;
            }
            let line = line
                .with_random_content(self.height as usize)
                .with_random_height(self.height);
            self.lines.push(line);
        }
    }

    pub fn print_at(&self, window: nc::WINDOW, x: i32) {
        for line in self.lines.iter() {
            line.print_at(window, x);
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Sequence {
    offset: i32,
    height: i32,
    content: String,
}

impl Sequence {
    pub fn with_random_height(mut self, max_height: i32) -> Self {
        self.height = thread_rng().gen_range(MIN_LINE_HEIGHT, max_height);
        self.offset -= self.height;
        self
    }

    pub fn with_random_content(mut self, length: usize) -> Self {
        self.content = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .collect();
        self
    }

    pub fn maybe(self) -> Option<Self> {
        if thread_rng().gen_range(0, 100) < MAYBE_CHANCE {
            Some(self)
        } else {
            None
        }
    }

    pub fn increment_offset(&mut self) {
        self.offset += 1;
    }

    pub fn is_visible(&self) -> bool {
        self.content.len() as i32 > self.offset
    }

    pub fn print_at(&self, window: nc::WINDOW, x: i32) {
        for (y, ch) in self.visible_content() {
            let color = if y == self.offset + self.height - 1 {
                LIGHT_GREEN
            } else {
                GREEN
            };
            nc::attron(nc::COLOR_PAIR(color));
            nc::mvwaddch(window, y, x, ch as u32);
            nc::attroff(nc::COLOR_PAIR(color));
        }
    }

    fn visible_content(&self) -> Vec<(i32, char)> {
        self.content
            .char_indices()
            .map(|(y, ch)| (y as i32, ch))
            .filter(|(y, _)| *y >= self.offset)
            .filter(|(y, _)| *y < self.offset + self.height)
            .collect()
    }

    fn offset(&self) -> i32 {
        self.offset
    }
}
