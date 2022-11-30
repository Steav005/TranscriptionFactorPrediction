use std::str::FromStr;

use strum::{Display, EnumIter, EnumString, ParseError};

#[derive(Debug, EnumIter, Clone, Copy, EnumString, Display, Hash, Eq, PartialEq)]
pub enum Base {
    A = 0,
    C = 1,
    G = 2,
    T = 3,
}

impl Base {
    pub fn complement(self) -> Base {
        match self {
            Base::A => Base::T,
            Base::C => Base::G,
            Base::G => Base::C,
            Base::T => Base::A,
        }
    }
}

#[derive(Debug)]
pub enum Sequence {
    Plus(PlusStrand),
    Minus(MinusStrand),
}

impl Sequence {
    pub fn as_slice(&self) -> &[Base] {
        match self {
            Sequence::Plus(p) => p.seq.as_slice(),
            Sequence::Minus(m) => m.seq.as_slice(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Sequence::Plus(p) => p.seq.len(),
            Sequence::Minus(m) => m.seq.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn name(&self) -> &str {
        match self {
            Sequence::Plus(p) => &p.name,
            Sequence::Minus(m) => &m.name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlusStrand {
    pub name: String,
    pub seq: Vec<Base>,
}

impl PlusStrand {
    pub fn from_str(name: &str, seq: &str) -> Result<Self, ParseError> {
        seq.chars()
            .map(|c| Base::from_str(&c.to_string()))
            .collect::<Result<Vec<_>, _>>()
            .map(|seq| Self {
                name: name.to_string(),
                seq,
            })
    }
}

#[derive(Debug)]
pub struct MinusStrand {
    pub name: String,
    pub seq: Vec<Base>,
}

impl From<&PlusStrand> for MinusStrand {
    fn from(p: &PlusStrand) -> Self {
        // Self(p.0.iter().cloned().rev().collect())
        Self {
            name: p.name.clone(),
            seq: p.seq.iter().cloned().rev().collect(),
        }
    }
}
