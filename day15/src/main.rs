use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Range, RangeInclusive},
    str::FromStr,
};

use common::aoc_input;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character,
    combinator::all_consuming,
    sequence::{self, preceded},
    IResult,
};
use tqdm::Iter;

const PT1_TARGET_ROW: isize = 2_000_000;
const PT2_TARGET_RANGE: RangeInclusive<isize> = 0..=4_000_000;

#[derive(PartialEq, Eq, Hash)]
struct Position {
    x: isize,
    y: isize,
}

struct SensorReport(Position, Position);

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn manhattan_dist(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

impl SensorReport {
    fn new(sensor: Position, beacon: Position) -> Self {
        Self(sensor, beacon)
    }

    /// The manhattan dist between the beacon and sensor of this report
    fn distance(&self) -> usize {
        self.0.manhattan_dist(&self.1)
    }

    /// Whether a given other point is in range of this sensor
    /// i.e whether its existence would cause this report to be invalid
    fn in_influence(&self, position: &Position) -> bool {
        self.0.manhattan_dist(position) <= self.distance()
    }

    /// Get range of positions covered by this report on a single row.
    /// i.e the range of positions where a beacon cannot be, as determined by this report
    fn compute_influence_on_row(&self, row: isize) -> Range<isize> {
        // Get our properties
        let distance = self.distance();
        let (my_x, my_y) = (self.0.x, self.0.y);

        // Determine radius of influence on this row
        let y_diff = row.abs_diff(my_y);
        let radius = distance.saturating_sub(y_diff) as isize;

        -radius + my_x..radius + my_x
    }
}

fn main() {
    // Parse input
    let input = aoc_input!();
    let reports = input
        .trim_end()
        .lines()
        .map(|line| line.parse::<SensorReport>().unwrap())
        .collect_vec();

    // Compute influence on specific line
    let influence_on_line = reports
        .iter()
        .flat_map(|report| report.compute_influence_on_row(PT1_TARGET_ROW))
        .collect::<HashSet<_>>();
    println!("[PT1] {}", influence_on_line.len());

    // Find the distress beacon
    println!("Finding distress beacon...");
    for y in PT2_TARGET_RANGE.tqdm() {
        // what sensors have influence here?
        let x_ranges = reports
            .iter()
            .filter(|report| report.distance().saturating_sub(report.0.y.abs_diff(y)) > 0)
            .map(|report| report.compute_influence_on_row(y));

        // Compute union of those ranges
        let ranges_union = union_ranges(x_ranges);
        let full_range = ranges_union.get(0).unwrap();

        // Is there a gap in that range?
        if full_range.start > *PT2_TARGET_RANGE.start() || full_range.end < *PT2_TARGET_RANGE.end()
        {
            // We found it!
            let pos = Position::new(full_range.end + 1, y);
            println!("[PT2] Tuning freq is {}", pos.x * 4_000_000 + pos.y);
            break;
        }
    }
}

#[cfg(test)]
mod test_solution {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_row_influence_computation() {
        let input = read_to_string("./sample.txt").unwrap();
        let reports = input
            .trim_end()
            .lines()
            .map(|line| line.parse::<SensorReport>().unwrap())
            .collect_vec();
        let influence_on_line = reports
            .iter()
            .flat_map(|report| report.compute_influence_on_row(10))
            .collect::<HashSet<_>>();
        assert_eq!(influence_on_line.len(), 26);
    }
}

/* Parsing */

impl FromStr for SensorReport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        all_consuming(sequence::tuple((
            preceded(tag("Sensor at "), parse_labeled_position),
            preceded(tag(": closest beacon is at "), parse_labeled_position),
        )))(s)
        .map(|(_, pair)| SensorReport::new(pair.0, pair.1))
        .map_err(|_| format!("Failed to parse sensor report: '{}'", s))
    }
}

fn parse_labeled_position(s: &str) -> IResult<&str, Position> {
    let (s, x) = preceded(tag("x="), character::complete::i32)(s)?;
    let (s, _) = tag(", ")(s)?;
    let (s, y) = preceded(tag("y="), character::complete::i32)(s)?;
    Ok((s, Position::new(x as isize, y as isize)))
}

#[cfg(test)]
mod test_parsing {
    use super::*;

    #[test]
    fn test_parse_report() {
        let report = SensorReport::from_str(
            "Sensor at x=3056788, y=2626224: closest beacon is at x=3355914, y=2862466",
        )
        .unwrap();
    }

    #[test]
    fn test_parse_position() {
        let (_, p) = parse_labeled_position("x=3992558, y=1933059").unwrap();
        assert_eq!(p.x, 3992558);
        assert_eq!(p.y, 1933059);
    }
}

/* Debug Impls */

impl std::fmt::Debug for SensorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sensor{:?} Closest Beacon{:?}", self.0, self.1)
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/* Util */
trait IterRangeExt<I> {
    fn range(&mut self) -> Option<RangeInclusive<I>>;
}

impl<Iter: Iterator<Item = I>, I: Ord + Copy> IterRangeExt<I> for Iter {
    fn range(&mut self) -> Option<RangeInclusive<I>> {
        let mut min = None;
        let mut max = None;
        for value in self.by_ref() {
            if min.is_none() {
                min = Some(value);
            }
            if max.is_none() {
                max = Some(value);
            }
            min = min.map(|min| if value < min { value } else { min });
            max = max.map(|max| if value > max { value } else { max });
        }
        min.and_then(|min| max.map(|max| (min..=max)))
    }
}

trait RangeIntersectsExt {
    fn intersects(&self, other: &Self) -> bool;
}

impl<Idx: Ord + Copy> RangeIntersectsExt for Range<Idx> {
    fn intersects(&self, other: &Self) -> bool {
        self.contains(&other.start)
            || self.contains(&other.end)
            || other.contains(&self.start)
            || other.contains(&self.end)
    }
}

fn union_ranges(ranges: impl Iterator<Item = Range<isize>>) -> Vec<Range<isize>> {
    let mut range_union: Vec<Range<isize>> = Vec::new();
    for range in ranges.sorted_by_key(|range| range.start) {
        if let Some(last_range) = range_union.last_mut() {
            if last_range.intersects(&range) {
                *last_range = Range {
                    start: range.start.min(last_range.start),
                    end: range.end.max(last_range.end),
                };
                continue;
            }
        }
        range_union.push(range);
    }
    range_union
}
