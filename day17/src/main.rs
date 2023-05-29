use std::collections::{HashMap, VecDeque};

use colored::{Color, Colorize};
use common::aoc_input;
use itertools::Itertools;
use once_cell::sync::Lazy;
use shape_macro::shape;

const WORLD_WIDTH: usize = 7;

static COLORS: Lazy<Vec<Color>> = Lazy::new(|| {
    vec![
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ]
});

static ROCK_SHAPES: Lazy<Vec<RockShape>> = Lazy::new(|| {
    vec![
        shape!(
            @@@@,
        ),
        shape!(
            .@.,
            @@@,
            .@.,
        ),
        shape!(
            ..@,
            ..@,
            @@@,
        ),
        shape!(
            @,
            @,
            @,
            @,
        ),
        shape!(
            @@,
            @@,
        ),
    ]
    .into_iter()
    .map(|segments| {
        let height = segments.iter().map(|p| p.1).max().unwrap();
        RockShape {
            segments: segments
                .into_iter()
                .map(|(x, y)| Position { x, y: height - y })
                .collect_vec(),
        }
    })
    .collect_vec()
});

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Direction {
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct JetDirection(Direction);

#[derive(
    Hash,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
)]
struct Position {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone)]
struct RockShape {
    /// Segments of rock shape, relative to top left
    segments: Vec<Position>,
}

#[derive(Debug)]
struct Rock {
    shape_index: usize,
    position: Position,
}

#[derive(Debug, Default)]
struct RockWorld {
    rock_map: HashMap<Position, usize>,
    falling_rock: Option<Rock>,
    settled_rocks: usize,
    jets: VecDeque<JetDirection>,
    highest_rock: isize,
}

#[derive(Debug)]
enum RockMovement {
    FromJet,
    FromGravity,
}
use tqdm::Iter;
use RockMovement::*;

macro_rules! position {
    ($v: expr) => {
        Position {
            x: $v as isize,
            y: $v as isize,
        }
    };
    ($x: expr, $y: expr) => {
        Position {
            x: $x as isize,
            y: $y as isize,
        }
    };
}

impl Direction {
    fn to_position(&self) -> Position {
        match self {
            Direction::Down => position!(0, -1),
            Direction::Left => position!(-1, 0),
            Direction::Right => position!(1, 0),
        }
    }
}

impl RockWorld {
    pub fn new(jets: Vec<JetDirection>) -> Self {
        Self {
            jets: jets.into(),
            ..Default::default()
        }
    }

    /// Attempt to move the rock and return whether it did
    pub fn try_move_falling(&mut self, direction: Direction) -> bool {
        let rock = self
            .falling_rock
            .as_mut()
            .expect("Can't move falling rock as there isn't any");
        let can_move = rock
            .to_positions()
            .iter()
            .map(|&p| p + direction.to_position())
            .all(|p| {
                self.rock_map.get(&p).is_none()
                    && p.y > 0
                    && p.x >= 0
                    && p.x < (WORLD_WIDTH as isize)
            });
        if can_move {
            rock.position += direction.to_position();
        }
        can_move
    }

    pub fn highest_rock(&self) -> isize {
        // self.rock_map.keys().map(|pos| pos.y).max().unwrap_or(0)
        self.highest_rock
    }

    pub fn settled_rocks(&self) -> usize {
        self.settled_rocks
    }

    fn rock_spawn_pos(&self) -> Position {
        position!(2, self.highest_rock() + 4)
    }

    pub fn step(&mut self) {
        // Spawn a new rock if we dont have one
        if self.falling_rock.is_none() {
            self.falling_rock = Some(Rock::new(self.settled_rocks(), self.rock_spawn_pos()));
        }

        // Move rock until settled
        for movement in vec![FromJet, FromGravity].iter().cycle() {
            match movement {
                FromJet => {
                    // Move from jet
                    let jet = self.jets.pop_front().unwrap();
                    self.try_move_falling(jet.0);

                    // Cycle jets
                    self.jets.push_back(jet);
                }
                FromGravity => {
                    let hit_ground = !self.try_move_falling(Direction::Down);
                    if hit_ground {
                        // Convert rock to settled rock
                        let rock = self.falling_rock.take().unwrap();
                        for pos in rock.to_positions() {
                            self.rock_map.insert(pos, self.settled_rocks() + 1);
                        }
                        self.highest_rock = self.highest_rock.max(rock.position.y + rock.height());

                        // Increment counter
                        self.settled_rocks += 1;

                        // End of step
                        break;
                    }
                }
            }
        }
    }
}

impl Rock {
    pub fn new(shape_index: usize, position: Position) -> Self {
        Self {
            shape_index: shape_index % ROCK_SHAPES.len(),
            position,
        }
    }

    pub fn shape(&self) -> &RockShape {
        &ROCK_SHAPES[self.shape_index]
    }

    pub fn height(&self) -> isize {
        self.shape()
            .segments
            .iter()
            .map(|pos| pos.y)
            .max()
            .unwrap_or(0)
    }

    pub fn overlaps_with(&self, pos: &Position) -> bool {
        let relative = *pos - self.position;
        self.shape()
            .segments
            .iter()
            .any(|&segment| segment == relative)
    }

    pub fn to_positions(&self) -> Vec<Position> {
        self.shape()
            .segments
            .iter()
            .map(|&pos| pos + self.position)
            .collect()
    }
}

impl TryFrom<char> for JetDirection {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '>' => Ok(JetDirection(Direction::Right)),
            '<' => Ok(JetDirection(Direction::Left)),
            _ => Err("Unknown character"),
        }
    }
}

fn main() {
    let input = aoc_input!();
    let jets: Vec<JetDirection> = input
        .trim_end()
        .chars()
        .map(|c| TryFrom::try_from(c).unwrap())
        .collect();

    // Part 1
    // let mut world = RockWorld::new(jets.clone());
    // while world.settled_rocks() < 2022 {
    //     world.step();
    // }
    // println!("{}\n", world);
    // println!("[PT1] tower height is {}", world.highest_rock());

    // Part 2
    // taking a sidequest to find patterns
    let mut world = RockWorld::new(jets);
    let mut map: HashMap<usize, isize> = HashMap::new();

    // hmmm
    while world.settled_rocks() < world.jets.len() * ROCK_SHAPES.len() + 1 {
        world.step();
    }

    let y = world.highest_rock();
    let world_bits: Vec<u8> = (0..WORLD_WIDTH)
        .map(|x| world.rock_map.get(&position!(x, y)).is_some().into())
        .collect_vec();
    dbg!(y, world_bits);

    // while world.settled_rocks() < 1000000 {
    //     world.step();

    //     // TODO: this doesn't work because the # of jets is too big, e.g more than 64
    //     // maybe I could store them in a vec? or something? Idk
    //     // are the jets even relevant? It feels like they would be

    //     // compute map key
    //     //   first 7 bits are row
    //     //   remaining bits are upcoming jets
    //     let y = world.highest_rock();
    //     let world_bits: Vec<u8> = (0..WORLD_WIDTH)
    //         .map(|x| world.rock_map.get(&position!(x, y)).is_some().into())
    //         .collect_vec();
    //     if (y == 1 || y == 10091) {
    //         dbg!(&world_bits);
    //     }
    //     // let jet_bits: Vec<u8> = world
    //     //     .jets
    //     //     .iter()
    //     //     .map(|j| (j.0 == Direction::Right).into())
    //     //     .collect();
    //     // let key = [jet_bits]
    //     //     .concat()
    //     //     .iter()
    //     //     .fold(0, |acc, &val| (acc << 1) | (val as usize));
    //     // eprintln!("{:#066b}", key);
    //     // eprintln!("{}", world);
    //     // if let Some(other_height) = map.get(&key) {
    //     //     println!("{} = {}", y, other_height);
    //     //     break;
    //     // } else {
    //     //     map.insert(key, y);
    //     // }
    // }
    // println!("{}", world);
    println!("[PT2] tower height is {}", world.highest_rock());
}

#[cfg(test)]
mod test_with_sample {
    use super::*;

    #[test]
    fn test_tower_height() {
        let input = include_str!("../sample.txt");
        let jets: Vec<JetDirection> = input
            .trim_end()
            .chars()
            .map(|c| TryFrom::try_from(c).unwrap())
            .collect();
        let mut world = RockWorld::new(jets);
        while world.settled_rocks() < 2022 {
            world.step();
        }
        println!("{}\n", world);
        assert_eq!(world.highest_rock(), 3068);
    }
}

impl std::fmt::Display for RockWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top = self.highest_rock().max(
            self.falling_rock
                .as_ref()
                .map(|r| r.position.y + r.height())
                .unwrap_or(0),
        );
        for y in (1..=top).rev() {
            write!(f, "|")?;
            for x in 0..WORLD_WIDTH {
                let p = position!(x, y);

                let c = if let Some(col) = self.rock_map.get(&p) {
                    "#".color(COLORS[col % COLORS.len()])
                } else if self
                    .falling_rock
                    .as_ref()
                    .map(|rock| rock.overlaps_with(&p))
                    .unwrap_or(false)
                {
                    "@".red()
                } else {
                    ".".black()
                };
                write!(f, "{}", c)?;
            }
            writeln!(
                f,
                "| {}",
                if y == top {
                    self.jets
                        .iter()
                        .take(5)
                        .map(|j| format!("{:?}", j))
                        .join("")
                } else {
                    "".to_owned()
                }
            )?;
        }
        write!(f, "+{}+", "-".repeat(WORLD_WIDTH))?;
        Ok(())
    }
}

impl std::fmt::Debug for JetDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Direction::Right => ">",
                Direction::Left => "<",
                _ => unreachable!(),
            }
        )
    }
}
