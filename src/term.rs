use std::{
    collections::HashMap,
    io::{self, StdoutLock, Write},
    time::Duration,
};

use bon::bon;
use crossterm::{
    QueueableCommand,
    cursor::{Hide, MoveTo, Show},
    event::{
        self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, enable_raw_mode},
};
use rand::Rng;
use tracing::{error, trace};

use crate::{CharType, Matrix, app::Speed};

const DEFAULT_HEAD_COLOR: Color = Color::Rgb {
    r: 39,
    g: 215,
    b: 181,
};

const DEFAULT_TAIL_COLOR: Color = Color::Rgb {
    r: 39,
    g: 216,
    b: 93,
};

pub trait Terminal {
    type Error;

    fn size() -> Result<(u16, u16), Self::Error>;

    fn update(&mut self, matrix: &Matrix<impl Rng>) -> Result<Option<Event>, Self::Error>;
}

#[derive(Debug)]
pub enum Event {
    Close,
    Resize(u16, u16),
    ChangeSpeed(Speed),
}

pub struct CrosstermTerminal<'a> {
    stdout: StdoutLock<'a>,
    head_color: Color,
    tail_color: Color,
}

#[bon]
impl<'a> CrosstermTerminal<'a> {
    #[builder]
    pub fn new(
        mut stdout: StdoutLock<'a>,
        #[builder(default = DEFAULT_HEAD_COLOR)] head_color: Color,
        #[builder(default = DEFAULT_TAIL_COLOR)] tail_color: Color,
    ) -> io::Result<Self> {
        enable_raw_mode()?;

        execute!(
            stdout,
            Hide,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
        )?;

        Ok(Self {
            stdout,
            head_color,
            tail_color,
        })
    }

    pub fn render<R: Rng>(&mut self, matrix: &Matrix<R>) -> io::Result<()> {
        let by_type = matrix.chars().fold(
            HashMap::<CharType, Vec<([u16; 2], char)>>::new(),
            |mut acc, (coords, ty, ch)| {
                acc.entry(ty)
                    .and_modify(|entry| {
                        entry.push((coords, ch));
                    })
                    .or_insert_with(|| vec![(coords, ch)]);
                acc
            },
        );
        trace!(groups =? by_type, "Grouped matrix characters by type");

        self.stdout.queue(Clear(ClearType::All))?;

        for (ty, chs) in by_type {
            self.stdout.queue(SetForegroundColor(match ty {
                CharType::Head => self.head_color,
                CharType::Tail => self.tail_color,
            }))?;

            for ([x, y], ch) in chs {
                trace!(?ty, x, y, "Queueing character rendering: {}", ch);
                queue!(self.stdout, MoveTo(x, y), Print(ch))?;
            }

            self.stdout.flush()?;
        }

        Ok(())
    }

    pub fn event(&mut self) -> io::Result<Option<Event>> {
        if !event::poll(Duration::ZERO)? {
            return Ok(None);
        }

        match event::read()? {
            CrosstermEvent::Resize(width, height) => Ok(Some(Event::Resize(width, height))),

            CrosstermEvent::Key(
                KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Esc | KeyCode::Char('q'),
                    ..
                }
                | KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char('c' | 'd'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                },
            ) => Ok(Some(Event::Close)),

            CrosstermEvent::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char(ch @ '1'..='9'),
                ..
            }) => Ok(Some(Event::ChangeSpeed(
                u64::from(ch.to_digit(10).unwrap()).into(),
            ))),

            _ => Ok(None),
        }
    }

    fn restore_terminal(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(self.stdout, Show, PopKeyboardEnhancementFlags)
    }
}

impl Terminal for CrosstermTerminal<'_> {
    type Error = io::Error;

    fn size() -> Result<(u16, u16), Self::Error> {
        terminal::size()
    }

    fn update(&mut self, matrix: &Matrix<impl Rng>) -> Result<Option<Event>, Self::Error> {
        self.render(matrix)?;
        self.event()
    }
}

impl Drop for CrosstermTerminal<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.restore_terminal() {
            error!("Failed to restore terminal: {}", err);
        }
    }
}
