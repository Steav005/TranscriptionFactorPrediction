use std::collections::HashMap;

use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, char, line_ending, not_line_ending, space0};
use nom::combinator::{map, map_res, opt};
use nom::multi::many0;
use nom::number::complete::float;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;
use ordered_float::NotNan;

use crate::matrix::Float;

pub type Css = Float;
pub type Mss = Float;

#[derive(Debug)]
pub struct Profile {
    pub css: Float,
    pub mss: Float,
    pub id: String,
}

impl Profile {
    pub fn parse(input: &str) -> IResult<&str, Profile> {
        map(
            tuple((
                tag("1"),
                delimited(space0::<&str, _>, char(','), space0),
                map_res(float, NotNan::new),
                delimited(space0, char(','), space0),
                map_res(float, NotNan::new),
                delimited(space0, char(','), space0),
                alphanumeric1,
                delimited(space0, char(','), space0),
                not_line_ending,
            )),
            |(_, _, css, _, mss, _, _, _, id)| Profile {
                css,
                mss,
                id: id.to_string(),
            },
        )(input)
    }

    pub fn parse_many(input: &str) -> IResult<&str, HashMap<String, (Css, Mss)>> {
        map(
            many0(map(
                map_res(terminated(not_line_ending, opt(line_ending)), Self::parse),
                |(_, m)| m,
            )),
            |mut p| {
                p.drain(..)
                    .map(|Profile { css, mss, id }| (id, (css, mss)))
                    .collect()
            },
        )(input)
    }
}

// #[test]
// fn parse_prof() {
//     let single = "1,0.85,0.85,M00205,V$GRE_C";

//     println!("{:?}", parse_profile(single));

//     let many = "1,0.85,0.85,M00162,V$OCT1_06
// 1,0.85,0.85,M00172,V$AP1FJ_Q2
// 1,0.85,0.85,M00179,V$CREBP1_Q2
// 1,0.85,0.85,M00189,V$AP2_Q6
// 1,0.85,0.85,M00205,V$GRE_C
// 1,0.85,0.85,M00206,V$HNF1_C
// 1,0.85,0.85,M00208,V$NFKB_C
// 1,0.85,0.85,M00209,V$NFY_C
// 1,0.85,0.85,M00221,V$SREBP1_02
// 1,0.85,0.85,M00225,V$STAT3_01";
//     println!("{:#?}", parse_profiles(many));
// }
