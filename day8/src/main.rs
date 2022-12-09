use std::{collections::HashMap, fs::read_to_string};

use forest::Forest;
use take_until::TakeUntilExt;

/// Utilities for working with a 2D grid of tree heights
mod forest {
    use std::ops::Index;

    #[derive(Debug)]
    pub struct Forest {
        tree_heights: Vec<Vec<usize>>,
    }

    impl Forest {
        pub fn new(tree_heights: Vec<Vec<usize>>) -> Self {
            Self { tree_heights }
        }

        pub fn num_rows(&self) -> usize {
            self.tree_heights.len()
        }

        pub fn num_cols(&self) -> usize {
            self.tree_heights[0].len()
        }

        pub fn loc(&self, row: usize, col: usize) -> Location {
            let num_rows = self.num_rows();
            let num_cols = self.num_cols();
            assert!(row < num_rows);
            assert!(col < num_cols);
            Location {
                row,
                col,
                num_rows: self.num_rows(),
                num_cols: self.num_cols(),
            }
        }

        pub fn all_locations(&self) -> impl Iterator<Item = Location> {
            let num_cols = self.num_cols();
            let num_rows = self.num_rows();
            (0..num_cols).flat_map(move |col| {
                (0..num_rows).map(move |row| Location {
                    row,
                    col,
                    num_cols,
                    num_rows,
                })
            })
        }

        pub fn edges_with_dirs_to_center(
            &self,
        ) -> impl Iterator<Item = (Location, Direction)> + '_ {
            ALL_DIRECTIONS.iter().flat_map(|dir| {
                let locs: Vec<Location> = match dir {
                    Direction::Up => (0..self.num_cols())
                        .map(|col| self.loc(self.num_rows() - 1, col))
                        .collect(),
                    Direction::Left => (0..self.num_rows())
                        .map(|row| self.loc(row, self.num_cols() - 1))
                        .collect(),
                    Direction::Down => (0..self.num_cols()).map(|col| self.loc(0, col)).collect(),
                    Direction::Right => (0..self.num_rows()).map(|row| self.loc(row, 0)).collect(),
                };
                locs.into_iter().map(|l| (l, *dir))
            })
        }
    }

    impl Index<Location> for Forest {
        type Output = usize;
        fn index(&self, index: Location) -> &usize {
            &self.tree_heights[index.col][index.row]
        }
    }

    #[derive(Eq, PartialEq, Clone, Copy, Hash)]
    pub struct Location {
        pub row: usize,
        pub col: usize,
        num_rows: usize,
        num_cols: usize,
    }

    impl std::fmt::Debug for Location {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({}, {})", self.row, self.col)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Direction {
        Right,
        Left,
        Up,
        Down,
    }

    pub const ALL_DIRECTIONS: [Direction; 4] = [
        Direction::Right,
        Direction::Left,
        Direction::Up,
        Direction::Down,
    ];

    impl Location {
        pub fn continue_in_dir(&self, dir: Direction) -> impl Iterator<Item = Self> {
            let mut curr: Option<Location> = Some(*self);
            std::iter::from_fn(move || {
                curr = curr.and_then(|c| match dir {
                    Direction::Right => c.right(),
                    Direction::Left => c.left(),
                    Direction::Up => c.up(),
                    Direction::Down => c.down(),
                });
                curr
            })
        }

        pub fn right(&self) -> Option<Self> {
            (self.col + 1 < self.num_cols).then(|| Self {
                row: self.row,
                col: self.col + 1,
                ..*self
            })
        }

        pub fn left(&self) -> Option<Self> {
            (self.col >= 1).then(|| Self {
                row: self.row,
                col: self.col - 1,
                ..*self
            })
        }

        pub fn up(&self) -> Option<Self> {
            (self.row >= 1).then(|| Self {
                row: self.row - 1,
                col: self.col,
                ..*self
            })
        }

        pub fn down(&self) -> Option<Self> {
            (self.row + 1 < self.num_rows).then(|| Self {
                row: self.row + 1,
                col: self.col,
                ..*self
            })
        }
    }
}

fn main() {
    // Parse input
    let tree_heights: Vec<Vec<usize>> = read_to_string("./input.txt")
        .unwrap()
        .lines()
        .map(|line| line.chars().flat_map(|c| c.to_string().parse()).collect())
        .collect();

    // Create forest
    let forest = forest::Forest::new(tree_heights);

    // Compute visibility map
    let mut visibility: HashMap<forest::Location, bool> = HashMap::new();
    for (location, direction) in forest.edges_with_dirs_to_center() {
        location
            .continue_in_dir(direction)
            .fold(vec![location], |mut acc, loc| {
                let height = forest[loc];
                let prev_height = acc.last().map(|&loc| forest[loc]).unwrap_or_default();
                if height > prev_height {
                    acc.push(loc);
                }
                acc
            })
            .iter()
            .for_each(|&l| {
                visibility.insert(l, true);
            });
    }

    // Count visible trees
    let sum: usize = visibility.values().map(|&x| x as usize).sum();
    println!("[PT1] {}", sum);

    // Compute scenic scores
    let score: usize = *compute_scenic_scores(&forest).values().max().unwrap();
    println!("[PT2] {}", score);
}

fn compute_scenic_scores(forest: &Forest) -> HashMap<forest::Location, usize> {
    forest
        .all_locations()
        .map(|location| {
            let tree_height = forest[location];
            let score = forest::ALL_DIRECTIONS
                .iter()
                .map(|&direction| {
                    location
                        .continue_in_dir(direction)
                        .take_until(|&loc| forest[loc] >= tree_height)
                        .count()
                })
                .product();
            (location, score)
        })
        .collect()
}
