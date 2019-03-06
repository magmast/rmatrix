use std::thread;
use std::time::Duration;
use rand::prelude::*;
use rand::distributions::Alphanumeric;

pub struct Line {
    chars: String,
    height: i32,
    y: i32,
}

impl Line {
    /// Creates random `Line` that doesn't start at y, but ends there.
    pub fn random_moved(y: i32, max_y: i32) -> Self {
        let height = thread_rng().gen_range(5, max_y);
        Line{
            height,
            chars: thread_rng().sample_iter(&Alphanumeric).take(max_y as usize).collect(),
            y: y - height,
        }
    }

    pub fn visible_chars<'a>(&'a self) -> impl Iterator<Item = (i32, char)> + 'a {
        let line_y = self.y;
        let height = self.height;

        self.chars
            .chars()
            .enumerate()
            .map(|(y, ch)| (y as i32, ch))
            .filter(move |(y, _)| *y >= line_y)
            .filter(move |(y, _)| *y < line_y + height)
    }
}

fn main() {
    let root = ncurses::initscr();

    let height = ncurses::getmaxy(root);
    let width = ncurses::getmaxx(root);

    let mut columns = (0..width)
        .step_by(2)
        .map(|x| {
            if thread_rng().gen_range(0, 100) > 70 {
                let line = Line::random_moved(0, height);
                (x, vec![line])
            } else {
                (x, Vec::new())
            }
        })
        .collect::<Vec<(i32, Vec<Line>)>>();

    loop {
        ncurses::erase();

        columns = columns.into_iter()
            .map(|(x, column)| {
                let column = column
                    .into_iter()
                    .map(|mut line| {
                        line.y += 1;
                        line
                    });
                (x, column)
            })
            .map(|(x, column)| (x, column.filter(|line| line.y < height)))
            .map(|(x, column)| (x, column.collect()))
            .map(|(x, mut column): (i32, Vec<Line>)| {
                match column.last() {
                    Some(line) if line.y > 3 => {
                        if thread_rng().gen_range(0, 100) > 60 {
                            let line = Line::random_moved(0, height);
                            column.push(line);
                        }
                    },
                    None => {
                        if thread_rng().gen_range(0, 100) > 65 {
                            let line = Line::random_moved(0, height);
                            column.push(line);
                        }
                    },
                    _ => {}
                }

                (x, column)
            })
            .collect();

        columns.iter()
            .for_each(|(x, column)| {
                column
                    .iter()
                    .for_each(|line| {
                        line.visible_chars()
                            .for_each(|(y, ch)| {
                                ncurses::mvaddch(y as i32, *x, ch as u32);
                            });
                    })
            });

        ncurses::refresh();
        thread::sleep(Duration::from_millis(50));
    }
}
