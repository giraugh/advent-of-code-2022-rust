use common::aoc_input;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    combinator::{all_consuming, map},
    multi::separated_list0,
    sequence::delimited,
    IResult,
};
use std::{cmp::Ordering, str::FromStr};

struct PacketPair {
    left: Packet,
    right: Packet,
}

#[derive(Clone, PartialEq, Eq)]
enum Packet {
    Number(u32),
    List(Vec<Packet>),
}

fn main() {
    // Parse input
    let input = aoc_input!();
    let pairs: Vec<PacketPair> = input
        .trim_end()
        .split("\n\n")
        .flat_map(FromStr::from_str)
        .collect();

    // Part 1
    let correct_pair_ind_sum: usize = pairs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.correct_order())
        .map(|(i, _)| i + 1)
        .sum();
    println!(
        "[PT1] Sum of indices of correct pairs is {}",
        correct_pair_ind_sum
    );

    // Part 2
    // Get all packets
    let mut all_packets = pairs
        .into_iter()
        .flat_map(|p| [p.left, p.right])
        .collect_vec();

    // Add divider packets
    let divider_packets = vec!["[[2]]", "[[6]]"]
        .iter()
        .map(|s| Packet::parse(s).unwrap().1)
        .collect_vec();
    all_packets.extend(divider_packets.clone().into_iter());

    // Sort packets and find dividers
    all_packets.sort();
    let decoder_key: usize = all_packets
        .iter()
        .enumerate()
        .filter(|&(_, p)| divider_packets.contains(p))
        .map(|(i, _)| i + 1)
        .product();
    println!("[PT2] The decoder key is {}", decoder_key);
}

impl PacketPair {
    fn correct_order(&self) -> bool {
        Packet::correct_order(&self.left, &self.right)
    }
}

impl Packet {
    fn correct_order(x: &Packet, y: &Packet) -> bool {
        match (x, y) {
            (Packet::Number(a), Packet::Number(b)) => a.le(b),
            (Packet::List(list_a), Packet::List(list_b)) => {
                let mut a = list_a.iter();
                let mut b = list_b.iter();
                loop {
                    match (a.next(), b.next()) {
                        (Some(a), Some(b)) if a != b => break Self::correct_order(a, b),
                        (None, Some(_)) => break true,
                        (Some(_), None) => break false,
                        (None, None) => break false,
                        _ => {}
                    }
                }
            }

            // If only one is a list, wrap it in a list
            (Packet::Number(_), Packet::List(_)) => Self::correct_order(&x.wrap(), y),
            (Packet::List(_), Packet::Number(_)) => Self::correct_order(x, &y.wrap()),
        }
    }

    fn wrap(&self) -> Self {
        Packet::List(vec![self.clone()])
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(character::complete::u32, Packet::Number),
            map(
                delimited(tag("["), separated_list0(tag(","), Packet::parse), tag("]")),
                Packet::List,
            ),
        ))(input)
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        Some(match Packet::correct_order(self, other) {
            true => Ordering::Less,
            false => Ordering::Greater,
        })
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        match Packet::correct_order(self, other) {
            true => Ordering::Less,
            false => Ordering::Greater,
        }
    }
}

impl FromStr for Packet {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is a shrine to Max
        all_consuming(Packet::parse)(s)
            .map(|res| res.1)
            .map_err(|_| "Failed to parse packet")
    }
}

impl FromStr for PacketPair {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split('\n')
            .map(|line| line.parse().unwrap())
            .collect_tuple()
            .unwrap();
        Ok(Self { left, right })
    }
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Number(num) => write!(f, "{}", num),
            Packet::List(elements) => write!(
                f,
                "[{}]",
                elements.iter().map(|el| format!("{:?}", el)).join(",")
            ),
        }
    }
}

impl std::fmt::Debug for PacketPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "L{:?}", self.left)?;
        writeln!(f, "R{:?}", self.right)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    macro_rules! assert_correct {
        ($a: expr, $b: expr) => {{
            let a = Packet::from_str($a).unwrap();
            let b = Packet::from_str($b).unwrap();
            assert!(Packet::correct_order(&a, &b));
        }};
    }

    macro_rules! assert_incorrect {
        ($a: expr, $b: expr) => {{
            let a = Packet::from_str($a).unwrap();
            let b = Packet::from_str($b).unwrap();
            assert!(!Packet::correct_order(&a, &b));
        }};
    }

    #[test]
    fn test_pair_correctness() {
        assert_correct!("[1,1,3,1,1]", "[1,1,5,1,1]");
        assert_correct!("[[1],[2,3,4]]", "[[1],4]");
        assert_incorrect!("[9]", "[[8,7,6]]");
        assert_correct!("[[4,4],4,4]", "[[4,4],4,4,4]");
        assert_incorrect!("[7,7,7,7]", "[7,7,7]");
        assert_correct!("[]", "[3]");
        assert_incorrect!("[[[]]]", "[[]]");
        assert_incorrect!("[1,[2,[3,[4,[5,6,7]]]],8,9]", "[1,[2,[3,[4,[5,6,0]]]],8,9]");
    }

    #[test]
    fn test_parse_input_full() {
        let input = read_to_string("./sample.txt").unwrap();
        let pairs: Vec<PacketPair> = input
            .trim_end()
            .split("\n\n")
            .flat_map(FromStr::from_str)
            .collect();
        let correct_pair_ind_sum: usize = pairs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.correct_order())
            .map(|(i, _)| i + 1)
            .sum();
        assert_eq!(correct_pair_ind_sum, 13);
    }
}
