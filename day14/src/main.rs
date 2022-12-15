/**
 * My implementation is a bit lazy and slow so running in release mode recommended :)
 */
use std::{collections::HashMap, str::FromStr};

use colored::Colorize;
use common::aoc_input;
use itertools::Itertools;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum SandCell {
    Empty,
    Rock,
    Sand,
}

#[derive(Debug)]
struct SandWorld {
    cells: HashMap<Position, SandCell>,
    sand_spawn: Position,
    floor_offset: Option<isize>,
}

struct SandWorldBuilder {
    rock_sequences: Vec<RockLineSequence>,
    sand_spawn: Option<Position>,
    floor_offset: Option<isize>,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone)]
struct RockLineSequence {
    points: Vec<Position>,
}

#[derive(Debug, PartialEq)]
enum SandOutcome {
    BlockSource,
    AtRest,
    FellIntoVoid,
}

impl SandWorldBuilder {
    fn new() -> Self {
        Self {
            rock_sequences: Vec::new(),
            sand_spawn: None,
            floor_offset: None,
        }
    }

    fn rock_sequences(mut self, rock_sequences: &[RockLineSequence]) -> Self {
        self.rock_sequences = rock_sequences.to_vec();
        self
    }

    fn sand_spawn(mut self, sand_spawn: Position) -> Self {
        self.sand_spawn = Some(sand_spawn);
        self
    }

    fn floor_offset(mut self, floor_offset: isize) -> Self {
        self.floor_offset = Some(floor_offset);
        self
    }

    fn build(&self) -> Result<SandWorld, &'static str> {
        // Draw lines
        let cells = self
            .rock_sequences
            .iter()
            .flat_map(|rock_sequence| {
                let mut sequence_points = vec![];
                rock_sequence.points.windows(2).for_each(|points| {
                    let (point, next_point) = (points[0], points[1]);
                    let mut curr = point;
                    while curr != next_point {
                        sequence_points.push(curr);
                        curr.x += (next_point.x - point.x).signum();
                        curr.y += (next_point.y - point.y).signum();
                    }
                    sequence_points.push(curr);
                });
                sequence_points
            })
            .map(|position| (position, SandCell::Rock))
            .collect::<HashMap<_, _>>();

        Ok(SandWorld {
            cells,
            sand_spawn: self.sand_spawn.ok_or("Sand spawn field is required")?,
            floor_offset: self.floor_offset,
        })
    }
}

impl SandWorld {
    fn empty(&self, position: &Position) -> bool {
        self.cells
            .get(position)
            .map(|&cell| cell == SandCell::Empty)
            .unwrap_or(true)
    }

    fn lowest_rock_row(&self) -> isize {
        self.cells
            .iter()
            .filter(|&(_, &cell)| cell == SandCell::Rock)
            .map(|(pos, _)| pos.y)
            .max()
            .unwrap()
    }

    fn sand_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|&(_, &cell)| cell == SandCell::Sand)
            .count()
    }

    fn step(&mut self) -> SandOutcome {
        // Spawn location free?
        if !self.empty(&self.sand_spawn) {
            return SandOutcome::BlockSource;
        }

        // Move sand until at rest or in void
        let mut curr = self.sand_spawn;
        loop {
            // Where will sand move?
            let possible_locations = vec![curr.down(), curr.down_left(), curr.down_right()];
            let next_location = possible_locations.into_iter().find(|pos| self.empty(pos));

            // Is sand now at rest?
            if let Some(next_location) = next_location {
                curr = next_location
            } else {
                self.cells.insert(curr, SandCell::Sand);
                return SandOutcome::AtRest;
            }

            // In void?
            let lowest_rock = self.lowest_rock_row();
            if let Some(floor_offset) = self.floor_offset {
                // Hit floor?
                if curr.y >= (lowest_rock + floor_offset) - 1 {
                    self.cells.insert(curr, SandCell::Sand);
                    return SandOutcome::AtRest;
                }
            } else {
                // In void?
                if curr.y > lowest_rock + 2 {
                    break;
                }
            }
        }

        // Return result
        SandOutcome::FellIntoVoid
    }
}

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn down(&self) -> Self {
        Self::new(self.x, self.y + 1)
    }

    fn down_right(&self) -> Self {
        Self::new(self.x + 1, self.y + 1)
    }

    fn down_left(&self) -> Self {
        Self::new(self.x - 1, self.y + 1)
    }
}

fn main() {
    let input = aoc_input!();
    let rock_sequences: Vec<RockLineSequence> = input
        .trim_end()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect_vec();

    // Part 1
    let mut world = SandWorldBuilder::new()
        .rock_sequences(&rock_sequences)
        .sand_spawn(Position::new(500, 0))
        .build()
        .unwrap();
    while SandOutcome::AtRest == world.step() {}
    println!("{}", world);
    println!("[PT1] Sand count is {}", world.sand_count());

    // Part 2
    let mut world = SandWorldBuilder::new()
        .rock_sequences(&rock_sequences)
        .sand_spawn(Position::new(500, 0))
        .floor_offset(2)
        .build()
        .unwrap();
    loop {
        match world.step() {
            SandOutcome::BlockSource => break,
            SandOutcome::AtRest => continue,
            SandOutcome::FellIntoVoid => break,
        }
    }
    println!("{}", world);
    println!("[PT2] Sand count is {}", world.sand_count());
}

#[cfg(test)]
mod test_world {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_sim_sand() {
        let input = read_to_string("./sample.txt").unwrap();
        let rock_sequences: Vec<RockLineSequence> = input
            .trim_end()
            .lines()
            .map(|line| line.parse().unwrap())
            .collect_vec();
        let mut world = SandWorldBuilder::new()
            .rock_sequences(&rock_sequences)
            .sand_spawn(Position::new(500, 0))
            .build()
            .unwrap();
        while SandOutcome::AtRest == world.step() {}
        println!("{}", world);
        assert_eq!(world.sand_count(), 24);

        // Part 2
        let mut world = SandWorldBuilder::new()
            .rock_sequences(&rock_sequences)
            .sand_spawn(Position::new(500, 0))
            .floor_offset(2)
            .build()
            .unwrap();
        loop {
            match world.step() {
                SandOutcome::BlockSource => break,
                SandOutcome::AtRest => continue,
                SandOutcome::FellIntoVoid => break,
            }
        }
        println!("{}", world);
        assert_eq!(world.sand_count(), 93);
    }
}

/* Parsing */
impl FromStr for RockLineSequence {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points = s
            .split(" -> ")
            .map(|pair| {
                let (x, y) = pair
                    .split(',')
                    .flat_map(FromStr::from_str)
                    .collect_tuple::<(_, _)>()
                    .unwrap();
                Position { x, y }
            })
            .collect_vec();
        Ok(Self { points })
    }
}

/* Debug Impls */

impl std::fmt::Display for SandWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_cells = self
            .cells
            .iter()
            .filter(|&(_, &cell)| cell != SandCell::Empty)
            .map(|(pos, _)| pos);
        let min_x = full_cells.clone().map(|pos| pos.x).min().unwrap();
        let max_x = full_cells.clone().map(|pos| pos.x).max().unwrap();
        let min_y = full_cells.clone().map(|pos| pos.y).min().unwrap();
        let max_y = full_cells.clone().map(|pos| pos.y).max().unwrap();
        (min_y..=max_y).for_each(|y| {
            (min_x..=max_x).for_each(|x| {
                let c = match self.cells.get(&Position::new(x, y)) {
                    Some(SandCell::Rock) => "\u{2592}".white(),
                    Some(SandCell::Sand) => "o".yellow(),
                    Some(SandCell::Empty) => " ".white(),
                    None => " ".white(),
                };
                write!(f, "{}", c).unwrap();
            });
            writeln!(f).unwrap();
        });
        Ok(())
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
