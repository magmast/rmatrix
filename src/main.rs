use std::thread;
use std::time::Duration;
use rand::prelude::*;
use rand::distributions::Alphanumeric;

pub struct Line {
    chars: String,
    height: i32,
    x: i32,
    y: i32,
}

impl Line {
    /// Creates random `Line` that doesn't start at y, but ends there.
    pub fn random_moved(x: i32, y: i32, max_y: i32) -> Self {
        let height = thread_rng().gen_range(5, max_y);
        Line{
            x,
            height,
            chars: thread_rng().sample_iter(&Alphanumeric).take(max_y as usize).collect(),
            y: y - height,
        }
    }
}

fn main() {
    let root = ncurses::initscr();

    let height = ncurses::getmaxy(root);
    let width = ncurses::getmaxx(root);

    let mut columns: Vec<Vec<Line>> = (0..width)
        .map(|x| {
            if thread_rng().gen_range(0, 100) > 70 {
                let line = Line::random_moved(x, 0, height);
                vec![line]
            } else {
                Vec::new()
            }
        })
        .collect();

    loop {
        ncurses::erase();

        columns = columns.into_iter()
            .map(|column| {
                column.into_iter()
                    .map(|mut line| {
                        line.y += 1;
                        line
                    })
            })
            .map(|column| column.filter(|line| line.y < height))
            .map(|column| column.collect())
            .enumerate()
            .map(|(x, mut column): (usize, Vec<Line>)| {
                match column.last() {
                    Some(line) if line.y > 3 => {
                        if thread_rng().gen_range(0, 100) > 60 {
                            let line = Line::random_moved(x as i32, 0, height);
                            column.push(line);
                        }
                    },
                    None => {
                        if thread_rng().gen_range(0, 100) > 65 {
                            let line = Line::random_moved(x as i32, 0, height);
                            column.push(line);
                        }
                    },
                    _ => {}
                }

                column
            })
            .collect();

        columns.iter()
            .for_each(|column| {
                column
                    .iter()
                    .for_each(|line| {
                        line.chars
                            .chars()
                            .enumerate()
                            .filter(|(y, _)| *y as i32 >= line.y)
                            .filter(|(y, _)| (*y as i32) < line.y + line.height)
                            .for_each(|(y, ch)| {
                                ncurses::mvaddch(y as i32, line.x, ch as u32);
                            });
                    })
            });

        ncurses::refresh();
        thread::sleep(Duration::from_millis(50));
    }

    ncurses::getch();
    ncurses::endwin();
}
