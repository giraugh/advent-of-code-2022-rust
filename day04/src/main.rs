use std::fs::read_to_string;

type Range = std::ops::RangeInclusive<usize>;

trait EncompassesExt {
    fn encompasses(&self, other: &Self) -> bool;
}

impl EncompassesExt for Range {
    fn encompasses(&self, other: &Self) -> bool {
        self.start() <= other.start() && self.end() >= other.end()
    }
}

#[cfg(test)]
#[test]
fn test_encompasses() {
    assert!((0..=10).encompasses(&(3..=5)));
    assert!(!(4..=5).encompasses(&(3..=5)));
}

trait OverlapsExt {
    fn overlaps(&self, other: &Self) -> bool;
}

impl OverlapsExt for Range {
    fn overlaps(&self, other: &Self) -> bool {
        self.start() <= other.end() && other.start() <= self.end()
    }
}

#[cfg(test)]
#[test]
fn test_overlaps() {
    assert!((0..=3).overlaps(&(2..=4)));
    assert!(!(0..=3).overlaps(&(4..=5)));
}

// this is kinda gross, wanted this to be a .parse() impl but I don't own any of the types.
// Should I have just made a transparent wrapper around Range?
fn range_from_str(s: &str) -> Result<Range, Box<dyn std::error::Error>> {
    let mut halves = s.split('-');
    let (h1, h2) = (
        halves.next().ok_or("missing portion")?,
        halves.next().ok_or("missing portion")?,
    );
    Ok((h1.parse()?)..=(h2.parse()?))
}

#[derive(Debug)]
struct Assignment(Range, Range);

impl std::str::FromStr for Assignment {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections = s.split(',');
        let (s1, s2) = (
            sections.next().ok_or("Missing section")?,
            sections.next().ok_or("Missing section")?,
        );
        let (r1, r2): (Range, Range) = (range_from_str(s1)?, range_from_str(s2)?);
        Ok(Self(r1, r2))
    }
}

fn main() {
    // Parse assignment
    let assignments: Vec<Assignment> = read_to_string("./input.txt")
        .unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    dbg!(&assignments.len());

    // Find encompassing assignments
    let encompassing = assignments
        .iter()
        .filter(|ass| ass.0.encompasses(&ass.1) || ass.1.encompasses(&ass.0));
    dbg!(encompassing.count());

    // Find overlapping assignments
    let overlapping = assignments
        .iter()
        .filter(|ass| ass.0.overlaps(&ass.1) || ass.1.overlaps(&ass.0));
    dbg!(overlapping.count());
}
