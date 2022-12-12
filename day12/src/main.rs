use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use colored::{ColoredString, Colorize};
use common::aoc_input;
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
struct MapPosition {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

struct Map {
    heights: Vec<u8>,
    width: usize,
    height: usize,
    start_position: MapPosition,
    goal_position: MapPosition,
}

struct Path<'a> {
    map: &'a Map,
    path: Vec<MapPosition>,
}

#[derive(Debug, Clone)]
struct SearchNode {
    position: MapPosition,
    parent: Option<Rc<SearchNode>>,
}

impl SearchNode {
    pub fn new(position: MapPosition, parent: Option<&SearchNode>) -> Self {
        Self {
            position,
            parent: parent.map(|p| Rc::new(p.clone())),
        }
    }

    pub fn backtrace(&self) -> Vec<MapPosition> {
        let mut curr = Rc::new(self.clone());
        std::iter::once(self.position)
            .chain(std::iter::from_fn(move || {
                let p = curr.parent.clone();
                p.map(|parent| {
                    curr = parent;
                    curr.position
                })
            }))
            .collect()
    }
}

impl Map {
    fn all_cells(&self) -> impl Iterator<Item = MapPosition> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| MapPosition {
                x,
                y,
                width: self.width,
                height: self.height,
            })
        })
    }

    /// Get neighbors of position that are traversable (i.e height w/in 1)
    fn get_neighbors(&self, position: MapPosition) -> impl Iterator<Item = MapPosition> + '_ {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .flat_map(move |offset| position + offset)
            .filter(move |offset_pos| self[offset_pos] <= (self[position] + 1))
    }
}

impl<'a> Path<'a> {
    fn len(&self) -> usize {
        self.path.len() - 1
    }

    /// Use BFS to find a path
    fn find_path(map: &'a Map, start_position: MapPosition) -> Option<Self> {
        let mut visited: HashSet<_> = vec![start_position].into_iter().collect();
        let mut frontier: VecDeque<SearchNode> = vec![start_position.into()].into();
        while !frontier.is_empty() {
            let node = frontier.pop_front().unwrap();
            if node.position == map.goal_position {
                return Some(Self {
                    map,
                    path: node.backtrace(),
                });
            }
            for child in map.get_neighbors(node.position) {
                if !visited.contains(&child) {
                    frontier.push_back(SearchNode::new(child, Some(&node)));
                    visited.insert(child);
                }
            }
        }
        None
    }
}

fn main() {
    // Parse input as map
    let input = aoc_input!();
    let map: Map = input.parse().unwrap();
    dbg!(&map);

    // Find length of path from start
    let path = Path::find_path(&map, map.start_position).unwrap();
    println!("[PT1] length of path from S->E is {}", path.len());
    dbg!(path);

    // Find shortest path from any 'a' location
    let shortest_path: Path = map
        .all_cells()
        .filter(|cell| map[cell] == 0)
        .flat_map(|start_pos| Path::find_path(&map, start_pos))
        .min_by_key(|p| p.len())
        .unwrap();

    // Output shortest path length
    println!(
        "[PT2] length of shortest path from a->E is {}",
        shortest_path.len()
    );
    dbg!(shortest_path);
}

/* Std Implementations */

impl From<MapPosition> for SearchNode {
    fn from(position: MapPosition) -> Self {
        Self {
            position,
            parent: None,
        }
    }
}

impl std::ops::Index<MapPosition> for Map {
    type Output = u8;
    fn index(&self, position: MapPosition) -> &Self::Output {
        &self[&position]
    }
}

impl std::ops::Index<&MapPosition> for Map {
    type Output = u8;
    fn index(&self, position: &MapPosition) -> &Self::Output {
        assert!(position.x < self.width && position.y < self.height);
        &self.heights[position.y * self.width + position.x]
    }
}

impl std::ops::Add<(isize, isize)> for MapPosition {
    type Output = Option<MapPosition>;
    fn add(self, rhs: (isize, isize)) -> Self::Output {
        let x_in_bounds = (0..(self.width as isize)).contains(&((self.x as isize) + rhs.0));
        let y_in_bounds = (0..(self.height as isize)).contains(&((self.y as isize) + rhs.1));
        (x_in_bounds && y_in_bounds).then_some(Self {
            x: ((self.x as isize) + rhs.0) as usize,
            y: ((self.y as isize) + rhs.1) as usize,
            ..self
        })
    }
}

impl std::str::FromStr for Map {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start = None;
        let mut goal = None;
        let grid: Vec<Vec<_>> = s
            .lines()
            .enumerate()
            .map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(|(x, c)| {
                        let height_symbol = match c {
                            'S' => {
                                start = Some((x, y));
                                'a'
                            }
                            'E' => {
                                goal = Some((x, y));
                                'z'
                            }
                            x => x,
                        };
                        (height_symbol as u8) - b'a'
                    })
                    .collect_vec()
            })
            .collect();
        let (height, width) = (grid.len(), grid[0].len());
        let heights = grid.into_iter().flatten().collect();
        if let (Some(start), Some(goal)) = (start, goal) {
            Ok(Self {
                heights,
                height,
                width,
                start_position: MapPosition {
                    x: start.0,
                    y: start.1,
                    width,
                    height,
                },
                goal_position: MapPosition {
                    x: goal.0,
                    y: goal.1,
                    width,
                    height,
                },
            })
        } else {
            Err("Didn't find start and end")
        }
    }
}

/* Display Implementations */

impl std::fmt::Debug for MapPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl std::fmt::Debug for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        let s = self
            .map
            .heights
            .chunks(self.map.width)
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, &height)| {
                        let found = self
                            .path
                            .iter()
                            .enumerate()
                            .find(|(_, p)| p.x == x && p.y == y);
                        if let Some((i, node)) = found {
                            if let Some(next) = self.path.get(i + 1) {
                                let diffx = (next.x as isize) - (node.x as isize);
                                let diffy = (next.y as isize) - (node.y as isize);
                                match (diffx, diffy) {
                                    (1, 0) => ">",
                                    (-1, 0) => "<",
                                    (0, -1) => "^",
                                    (0, 1) => "v",
                                    _ => "?",
                                }
                                .red()
                            } else {
                                "*".green()
                            }
                        } else {
                            height_to_color_string(height).black()
                        }
                    })
                    .join("")
            })
            .join("\n");
        write!(f, "{}", s)
    }
}

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}",
            self.heights
                .chunks(self.width)
                .map(|row| row
                    .iter()
                    .map(|&height| height_to_color_string(height))
                    .join(""))
                .join("\n")
        )
    }
}

/* Util */

fn height_to_color_string(height: u8) -> ColoredString {
    let s = ((height + b'a') as char).to_string();
    match height {
        0..=1 => s.cyan(),
        2..=4 => s.green(),
        5..=12 => s.yellow(),
        13..=18 => s.red(),
        19..=26 => s.bright_magenta(),
        _ => s.white(),
    }
}
