use nalgebra::Matrix1x4;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alphanumeric1, digit1, line_ending, not_line_ending, space1};
use nom::combinator::{map, map_res, rest};
use nom::multi::{many0, separated_list0};
use nom::number::complete::float;
use nom::sequence::{pair, separated_pair};
use nom::IResult;
use thiserror::Error;

use crate::matrix::{Float, PwmMatrix, PwmMatrixInner};

#[derive(Debug, Clone)]
pub enum TransfacTag {
    Id(String),
    Po(Vec<String>),
    Row(usize, Vec<Float>),
    Ignore,
}

impl TransfacTag {
    pub fn get_po(&self) -> Option<&Vec<String>> {
        if let TransfacTag::Po(po) = self {
            return Some(po);
        }
        None
    }

    pub fn get_name(&self) -> Option<&str> {
        if let TransfacTag::Id(n) = self {
            return Some(n);
        }
        None
    }

    pub fn get_row(&self) -> Option<(usize, &Vec<Float>)> {
        if let TransfacTag::Row(i, v) = self {
            return Some((*i, v));
        }
        None
    }
}

#[derive(Debug, Error)]
pub enum TransfacParseError {
    #[error("Encountered unexpected Base: {0}")]
    UnexpectedBase(String),
    #[error("No 'PO' found")]
    NoPo,
    #[error("No 'ID' found")]
    NoId,
    #[error("No value row found")]
    NoRows,
}

impl TryFrom<Vec<TransfacTag>> for PwmMatrix {
    type Error = TransfacParseError;

    fn try_from(value: Vec<TransfacTag>) -> Result<Self, Self::Error> {
        // let po: Vec<_> = value
        //     .iter()
        //     .find_map(TransfacTag::get_po)
        //     .ok_or(TransfacParseError::NoPo)?
        //     .iter()
        //     .map(|b| match Base::from_str(b) {
        //         Ok(b) => Ok(b),
        //         Err(_) => Err(TransfacParseError::UnexpectedBase(b.to_string())),
        //     })
        //     .collect::<Result<_, _>>()?;
        // TODO check Base order and everything else

        let name = value
            .iter()
            .find_map(TransfacTag::get_name)
            .ok_or(TransfacParseError::NoId)?
            .to_string();

        let rows = value.iter().filter_map(TransfacTag::get_row);
        let nrows = rows
            .clone()
            .max_by_key(|(i, _)| *i)
            .ok_or(TransfacParseError::NoRows)?
            .0
            + 1;
        let mut pwm = PwmMatrixInner::zeros(nrows);

        for (i, v) in rows {
            pwm.set_row(i, &Matrix1x4::from_iterator(v.iter().cloned()))
        }

        Ok(PwmMatrix { name, matrix: pwm })
    }
}

pub fn ignore_line(input: &str) -> IResult<&str, TransfacTag> {
    map(not_line_ending, |_| TransfacTag::Ignore)(input)
}

pub fn parse_id(input: &str) -> IResult<&str, TransfacTag> {
    map(
        separated_pair(tag("ID"), space1::<&str, _>, not_line_ending),
        |(_, id)| TransfacTag::Id(id.to_string()),
    )(input)
}

pub fn parse_po(input: &str) -> IResult<&str, TransfacTag> {
    map(
        pair(
            tag("PO"),
            many0(map(pair(space1, alphanumeric1::<&str, _>), |(_, n)| {
                n.to_string()
            })),
        ),
        |(_, names)| TransfacTag::Po(names),
    )(input)
}

pub fn parse_row(input: &str) -> IResult<&str, TransfacTag> {
    map(
        pair(
            map_res(digit1::<&str, _>, |v| v.parse::<usize>()),
            many0(map_res(pair(space1, float), |(_, v)| Float::new(v))),
        ),
        |(index, values)| TransfacTag::Row(index, values),
    )(input)
}

pub fn parse_matrix(input: &str) -> IResult<&str, Vec<TransfacTag>> {
    separated_list0(
        line_ending,
        alt((parse_id, parse_po, parse_row, ignore_line)),
    )(input)
}

pub fn parse_matrices(input: &str) -> IResult<&str, Vec<Vec<TransfacTag>>> {
    map(
        separated_list0(
            tag("//"),
            map(
                map_res(alt((take_until("//"), rest)), parse_matrix),
                |(_, mut m)| {
                    m.drain(..)
                        .filter(|t| !matches!(t, TransfacTag::Ignore))
                        .collect::<Vec<TransfacTag>>()
                },
            ),
        ),
        |mut m| m.drain(..).filter(|m| !m.is_empty()).collect(),
    )(input)
}
