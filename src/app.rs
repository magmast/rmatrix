use std::{ops::Deref, str::FromStr, thread, time::Duration};

use anyhow::{Error, anyhow};
use bon::bon;
use rand::Rng;
use tracing::{debug, trace};

use crate::{
    Matrix, SequenceHeightBounds,
    term::{Event, Terminal},
};

#[derive(Debug)]
pub struct App<R: Rng, T: Terminal> {
    terminal: T,
    matrix: Matrix<R>,
    speed: Speed,
}

#[bon]
impl<R: Rng, T: Terminal> App<R, T> {
    #[builder]
    pub fn new(
        rng: R,
        terminal: T,
        #[builder(into, default)] speed: Speed,
        gap: Option<u16>,
        #[builder(into)] sequence_height_bounds: Option<SequenceHeightBounds>,
        sequence_probability: Option<f64>,
    ) -> Result<Self, T::Error> {
        let (width, height) = T::size()?;
        debug!(width, height, "Retrieved terminal size");

        Ok(Self {
            terminal,
            matrix: Matrix::builder()
                .rng(rng)
                .width(width)
                .height(height)
                .maybe_gap(gap)
                .maybe_sequence_height_bounds(sequence_height_bounds)
                .maybe_sequence_probability(sequence_probability)
                .build(),
            speed,
        })
    }

    pub fn run(&mut self) -> Result<(), T::Error> {
        loop {
            match self.terminal.update(&self.matrix)? {
                Some(Event::Close) => {
                    debug!("Received close event");
                    break;
                }
                Some(Event::Resize(width, height)) => {
                    debug!(width, height, "Received resize event");
                    self.matrix.resize(width, height)
                }
                Some(Event::ChangeSpeed(speed)) => {
                    debug!(?speed, "Received change speed event");
                    self.speed = speed;
                }
                None => trace!("No event received"),
            }

            self.matrix.update();

            trace!("Delaying event loop iteration");
            thread::sleep(Duration::from_millis(1000 / self.speed.fps()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Speed(u64);

impl Speed {
    pub fn new(speed: u64) -> Self {
        assert!((0..=9).contains(&speed), "Speed must be between 1 and 9");

        Self(speed)
    }

    pub fn fps(&self) -> u64 {
        **self * 5
    }
}

impl FromStr for Speed {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = s.parse::<u64>()?;
        if (1..=9).contains(&val) {
            Ok(Self::new(val))
        } else {
            Err(anyhow!("Speed must be between 1 and 9"))
        }
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::new(5)
    }
}

impl From<u64> for Speed {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl Deref for Speed {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
