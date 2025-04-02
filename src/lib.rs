use std::{
    mem,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    path::PathBuf,
};

use app::Speed;
use bon::bon;
use clap::Parser;
use crossterm::style::Color;
use rand::{
    Rng,
    distr::{Alphanumeric, Distribution},
};

pub mod app;
pub mod term;

#[derive(Debug)]
pub struct Matrix<R: Rng> {
    descriptor: MatrixDescriptor,
    gap: u16,
    rng: R,
    cols: Vec<Column>,
}

#[bon]
impl<R: Rng> Matrix<R> {
    #[builder]
    pub fn new(
        mut rng: R,
        #[builder(default = 80)] width: u16,
        #[builder(default = 24)] height: u16,
        #[builder(default = 1)] gap: u16,
        #[builder(into, default = SequenceHeightBounds::RangeFull)]
        sequence_height_bounds: SequenceHeightBounds,
        #[builder(default = 0.05)] sequence_probability: f64,
    ) -> Self {
        let descriptor = MatrixDescriptor {
            width,
            height,
            sequence_height_bounds,
            sequence_probability,
        };

        let cols: Vec<_> = (0..width)
            .step_by(usize::from(gap) + 1)
            .map(|x| Column::new(&mut rng, x, &descriptor))
            .collect();

        Self {
            descriptor,
            gap,
            rng,
            cols,
        }
    }

    pub fn update(&mut self) {
        for col in &mut self.cols {
            col.update(&mut self.rng, &self.descriptor);
        }
    }

    pub fn chars(&self) -> impl Iterator<Item = ([u16; 2], CharType, char)> {
        self.cols.iter().flat_map(|col| col.chars())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let prev_width = self.descriptor.height;

        self.descriptor.width = width;
        self.descriptor.height = height;

        self.cols = mem::take(&mut self.cols)
            .into_iter()
            .filter(|col| col.x < width)
            .chain(
                ((prev_width + 1)..width)
                    .step_by(usize::from(self.gap))
                    .map(|x| Column::new(&mut self.rng, x, &self.descriptor)),
            )
            .collect();
    }
}

#[derive(Debug)]
struct MatrixDescriptor {
    #[allow(unused)]
    width: u16,
    height: u16,
    sequence_height_bounds: SequenceHeightBounds,
    sequence_probability: f64,
}

#[derive(Debug)]
struct Column {
    x: u16,
    seqs: Vec<Sequence>,
}

impl Column {
    fn new(mut rng: impl Rng, x: u16, descriptor: &MatrixDescriptor) -> Self {
        Self {
            x,
            seqs: rng
                .random_bool(descriptor.sequence_probability)
                .then(|| Sequence::new(rng, descriptor))
                .into_iter()
                .collect(),
        }
    }

    fn update(&mut self, mut rng: impl Rng, descriptor: &MatrixDescriptor) {
        self.seqs = mem::take(&mut self.seqs)
            .into_iter()
            .map(|mut seq| {
                seq.update();
                seq
            })
            .filter(|seq| seq.offset <= i32::from(descriptor.height))
            .chain(
                (self.seqs.iter().all(|seq| seq.offset > 0)
                    && rng.random_bool(descriptor.sequence_probability))
                .then(|| Sequence::new(rng, descriptor)),
            )
            .collect();
    }

    fn chars(&self) -> impl Iterator<Item = ([u16; 2], CharType, char)> {
        self.seqs
            .iter()
            .flat_map(|seq| seq.chars().map(|(y, ty, ch)| ([self.x, y], ty, ch)))
    }
}

#[derive(Debug)]
struct Sequence {
    offset: i32,
    height: u16,
    chars: Vec<char>,
}

impl Sequence {
    fn new(mut rng: impl Rng, descriptor: &MatrixDescriptor) -> Self {
        let chars = Alphanumeric
            .sample_iter(&mut rng)
            .take(usize::from(descriptor.height))
            .map(char::from)
            .collect();

        let height = descriptor
            .sequence_height_bounds
            .sample(&mut rng, descriptor.height);

        Sequence {
            offset: -i32::from(height),
            height,
            chars,
        }
    }

    fn update(&mut self) {
        self.offset += 1;
    }

    fn chars(&self) -> impl Iterator<Item = (u16, CharType, char)> {
        let last_visible_y = self.offset + i32::from(self.height) - 1;

        (self.offset..(self.offset + i32::from(self.height)))
            .filter(|&y| y >= 0 && (y as usize) < self.chars.len())
            .map(move |y| {
                (
                    y as u16,
                    if y == last_visible_y {
                        CharType::Head
                    } else {
                        CharType::Tail
                    },
                    self.chars[y as usize],
                )
            })
    }
}

#[derive(Debug)]
pub enum SequenceHeightBounds {
    Range(Range<u16>),
    RangeInclusive(RangeInclusive<u16>),
    RangeFrom(RangeFrom<u16>),
    RangeTo(RangeTo<u16>),
    RangeToInclusive(RangeToInclusive<u16>),
    RangeFull,
}

impl SequenceHeightBounds {
    fn sample(&self, mut rng: impl Rng, max_height: u16) -> u16 {
        match self {
            Self::Range(range) => rng.random_range(range.clone()),
            Self::RangeInclusive(range) => rng.random_range(range.clone()),
            Self::RangeFrom(range) => rng.random_range(range.start..max_height),
            Self::RangeTo(range) => rng.random_range(0..range.end),
            Self::RangeToInclusive(range) => rng.random_range(0..=range.end),
            Self::RangeFull => rng.random_range(0..max_height),
        }
    }
}

impl From<Range<u16>> for SequenceHeightBounds {
    fn from(range: Range<u16>) -> Self {
        Self::Range(range)
    }
}

impl From<RangeInclusive<u16>> for SequenceHeightBounds {
    fn from(range: RangeInclusive<u16>) -> Self {
        Self::RangeInclusive(range)
    }
}

impl From<RangeFrom<u16>> for SequenceHeightBounds {
    fn from(range: RangeFrom<u16>) -> Self {
        Self::RangeFrom(range)
    }
}

impl From<RangeTo<u16>> for SequenceHeightBounds {
    fn from(range: RangeTo<u16>) -> Self {
        Self::RangeTo(range)
    }
}

impl From<RangeToInclusive<u16>> for SequenceHeightBounds {
    fn from(range: RangeToInclusive<u16>) -> Self {
        Self::RangeToInclusive(range)
    }
}

impl From<RangeFull> for SequenceHeightBounds {
    fn from(_: RangeFull) -> Self {
        Self::RangeFull
    }
}

/// Describes where the character is in the sequence.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum CharType {
    /// Character the most bottom character in the sequence.
    Head,

    /// Character isn't [`CharType::Head`].
    Tail,
}

#[cfg(test)]
mod tests {
    mod sequence {
        use crate::{MatrixDescriptor, Sequence, SequenceHeightBounds};

        #[test]
        fn test_samples_correct_number_of_chars() {
            const HEIGHTS: &[u16] = &[10, 16, 24, u16::MAX];

            let mut rng = rand::rng();
            for height in HEIGHTS.iter().copied() {
                let sequence = Sequence::new(
                    &mut rng,
                    &MatrixDescriptor {
                        width: 80,
                        height,
                        sequence_height_bounds: SequenceHeightBounds::RangeFull,
                        sequence_probability: 1.0,
                    },
                );
                assert_eq!(sequence.chars.len(), usize::from(height));
            }
        }

        #[test]
        fn test_starts_above_the_matrix() {
            let mut rng = rand::rng();

            let sequence = Sequence::new(
                &mut rng,
                &MatrixDescriptor {
                    width: 80,
                    height: 24,
                    sequence_height_bounds: (5..6).into(),
                    sequence_probability: 1.0,
                },
            );

            assert_eq!(sequence.offset, -5);
        }

        #[test]
        fn test_has_no_chars_when_above_the_matrix() {
            let mut rng = rand::rng();

            let descriptor = MatrixDescriptor {
                width: 80,
                height: 24,
                sequence_height_bounds: (5..16).into(),
                sequence_probability: 1.0,
            };

            let sequence = Sequence::new(&mut rng, &descriptor);

            assert_eq!(sequence.chars().count(), 0);
        }
    }
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[arg(
        long,
        value_parser = parse_color,
        help = "Color of the first character in a falling sequence"
    )]
    pub head_color: Option<Color>,

    #[arg(
        long,
        value_parser = parse_color,
        help = "Color of the tail characters in a falling sequence"
    )]
    pub tail_color: Option<Color>,

    #[arg(short, long, help = "Speed of the falling sequences")]
    pub speed: Option<Speed>,

    #[arg(short, long, help = "Path to a log file")]
    pub log_file: Option<PathBuf>,
}

fn parse_color(s: &str) -> Result<Color, serde_plain::Error> {
    serde_plain::from_str(s)
}
