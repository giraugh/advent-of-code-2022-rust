use common::aoc_input;
use itertools::Itertools;
use std::{collections::HashSet, convert::Infallible, str::FromStr};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct Cube(i32, i32, i32);

impl Cube {
    pub fn sides(&self) -> Vec<Cube> {
        vec![
            Cube(self.0 - 1, self.1, self.2),
            Cube(self.0 + 1, self.1, self.2),
            Cube(self.0, self.1 - 1, self.2),
            Cube(self.0, self.1 + 1, self.2),
            Cube(self.0, self.1, self.2 - 1),
            Cube(self.0, self.1, self.2 + 1),
        ]
    }
}

impl FromStr for Cube {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: (i32, i32, i32) = s
            .splitn(3, ',')
            .map(|s| s.parse().unwrap())
            .collect_tuple()
            .unwrap();
        Ok(Cube(nums.0, nums.1, nums.2))
    }
}

fn main() {
    // Parse input points
    let cubes: HashSet<Cube> = aoc_input!()
        .lines()
        .map(FromStr::from_str)
        .collect::<Result<HashSet<_>, Infallible>>()
        .unwrap();

    // Stupid solution first (Part 1)
    let surface_area_pt1 = cubes
        .iter()
        .flat_map(|cube| cube.sides())
        .filter(|side| !cubes.contains(side))
        .count();

    println!("PT1: {}", surface_area_pt1);

    // Find bounds of particle
    // (I cheated and found a much larger bounding box, could be shrunk down by doing min/max on
    // each axis seperately)
    let values = cubes.iter().flat_map(|cube| [cube.0, cube.1, cube.2]);
    let (min, max) = (values.clone().min().unwrap(), values.max().unwrap());
    let bounds = min - 1..=max + 1;

    // FLood fill
    let mut air_cubes = HashSet::with_capacity(cubes.len());
    let mut frontier = Vec::new();
    frontier.push(Cube(min - 1, min - 1, min - 1));

    while let Some(cube) = frontier.pop() {
        air_cubes.insert(cube.clone());
        cube.sides()
            .iter()
            .filter(|spot| {
                !cubes.contains(spot)
                    && !air_cubes.contains(spot)
                    && bounds.contains(&spot.0)
                    && bounds.contains(&spot.1)
                    && bounds.contains(&spot.2)
            })
            .for_each(|cube| frontier.push(cube.clone()));
    }

    let surface_area_pt2 = cubes
        .iter()
        .flat_map(|cube| cube.sides())
        .filter(|side| air_cubes.contains(side))
        .count();

    println!("PT2: {}", surface_area_pt2);
}
