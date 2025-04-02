use std::{
    collections::HashMap,
    io::{self, StdoutLock, Write},
    time::Duration,
};

use bon::bon;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, enable_raw_mode},
    tty::IsTty,
};
use rand::Rng;
use tracing::error;

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
    previous_state: HashMap<[u16; 2], (CharType, char)>,
}

#[bon]
impl<'a> CrosstermTerminal<'a> {
    #[builder]
    pub fn new(
        mut stdout: StdoutLock<'a>,
        #[builder(default = DEFAULT_HEAD_COLOR)] head_color: Color,
        #[builder(default = DEFAULT_TAIL_COLOR)] tail_color: Color,
    ) -> io::Result<Self> {
        if !stdout.is_tty() {
            return Err(io::Error::new(io::ErrorKind::Other, "stdout is not a TTY"));
        }

        enable_raw_mode()?;

        execute!(
            stdout,
            Clear(ClearType::All),
            Hide,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
        )?;

        Ok(Self {
            stdout,
            head_color,
            tail_color,
            previous_state: HashMap::new(),
        })
    }

    pub fn render<R: Rng>(&mut self, matrix: &Matrix<R>) -> io::Result<()> {
        let current_state: HashMap<_, _> = matrix
            .chars()
            .map(|(coords, ty, ch)| (coords, (ty, ch)))
            .collect();

        self.render_update(&current_state)?;
        self.render_clear(&current_state)?;
        self.stdout.flush()?;

        self.previous_state = current_state;

        Ok(())
    }

    /// Renders new or updated characters.
    fn render_update(
        &mut self,
        current_state: &HashMap<[u16; 2], (CharType, char)>,
    ) -> io::Result<()> {
        for (coords, (ty, ch)) in current_state {
            let should_update = match self.previous_state.get(coords) {
                Some((prev_ty, prev_ch)) => prev_ty != ty || prev_ch != ch,
                None => true,
            };
            if !should_update {
                continue;
            }

            queue!(
                self.stdout,
                MoveTo(coords[0], coords[1]),
                SetForegroundColor(match ty {
                    CharType::Head => self.head_color,
                    CharType::Tail => self.tail_color,
                }),
                Print(*ch)
            )?;
        }

        Ok(())
    }

    /// Removes characters that are no longer present in the current state.
    fn render_clear(
        &mut self,
        current_state: &HashMap<[u16; 2], (CharType, char)>,
    ) -> io::Result<()> {
        for coords in self.previous_state.keys() {
            if !current_state.contains_key(coords) {
                queue!(self.stdout, MoveTo(coords[0], coords[1]), Print(" "))?;
            }
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
