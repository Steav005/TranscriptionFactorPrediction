use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alpha1, multispace0, multispace1, not_line_ending, space1};
use nom::combinator::{map, map_res, opt, rest};
use nom::multi::{many0, separated_list0};
use nom::sequence::{pair, tuple};
use nom::IResult;

use crate::sequence::PlusStrand;

#[derive(Debug, Clone)]
pub struct Fasta {
    pub name: String,
    pub seq: String,
}

impl TryFrom<Fasta> for PlusStrand {
    type Error = strum::ParseError;

    fn try_from(value: Fasta) -> Result<Self, Self::Error> {
        PlusStrand::from_str(&value.name, &value.seq)
    }
}

impl Fasta {
    pub fn parse_many(input: &str) -> IResult<&str, Vec<Fasta>> {
        many0(Self::parse)(input)
    }

    pub fn parse(input: &str) -> IResult<&str, Fasta> {
        fn parse_seq(input: &str) -> IResult<&str, Vec<&str>> {
            map(
                tuple((
                    multispace0,
                    separated_list0(multispace1, alpha1),
                    multispace0,
                )),
                |(_, s, _)| s,
            )(input)
        }

        map(
            pair(
                tuple((
                    tag(">"),
                    alt((take_until(" "), take_until("\t"), not_line_ending)),
                    opt(pair(space1, not_line_ending)),
                )),
                map(
                    map_res(alt((take_until(">"), rest)), parse_seq),
                    |(_, s)| s.join(""),
                ),
            ),
            |((_, name, _), seq)| Fasta {
                name: name.to_string(),
                seq,
            },
        )(input)
    }
}
