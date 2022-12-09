use std::{collections::HashSet, fs::read_to_string};

struct Action {
    offset: Vector,
    repetitions: usize,
}

fn actions_from_str(s: &str) -> Vec<Action> {
    s.lines()
        .map(|line| {
            let (dir, amt) = line.split_once(' ').unwrap();
            let offset: Vector = dir.chars().next().unwrap().into();
            let repetitions: usize = amt.parse().unwrap();
            Action {
                offset,
                repetitions,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Hash, Eq)]
struct Vector(isize, isize);

impl From<char> for Vector {
    fn from(c: char) -> Self {
        match c {
            'U' => Vector(0, -1),
            'D' => Vector(0, 1),
            'L' => Vector(-1, 0),
            'R' => Vector(1, 0),
            _ => panic!("unknown char"),
        }
    }
}

impl From<Vector> for (isize, isize) {
    fn from(v: Vector) -> Self {
        (v.0, v.1)
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Self;
    fn add(self, rhs: Vector) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Self;
    fn sub(self, rhs: Vector) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Vector {
    fn sign(&self) -> Self {
        Self(self.0.signum(), self.1.signum())
    }
    fn abs(&self) -> Self {
        Self(self.0.abs(), self.1.abs())
    }
}

struct Rope {
    knots: Vec<Vector>,
}

impl Rope {
    fn new(tail_segments: usize) -> Self {
        Self {
            knots: (0..tail_segments + 1).map(|_| Default::default()).collect(),
        }
    }

    fn head_mut(&mut self) -> &mut Vector {
        self.knots.first_mut().unwrap()
    }

    fn head(&self) -> &Vector {
        self.knots.first().unwrap()
    }

    fn tail(&self) -> &Vector {
        self.knots.last().unwrap()
    }

    pub fn track_tail_positions(&mut self, actions: &[Action]) -> HashSet<Vector> {
        actions
            .iter()
            .flat_map(|action| {
                (0..action.repetitions)
                    .map(|_| {
                        self.move_head(action.offset);
                        *self.tail()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<HashSet<_>>()
    }

    pub fn move_head(&mut self, movement: Vector) {
        // Move head
        *self.head_mut() = *self.head() + movement;

        // Move tail
        (0..self.knots.len())
            .collect::<Vec<_>>()
            .windows(2)
            .for_each(|inds| {
                // Some light hacks here to convince the
                // borrow checker to give us two refs into the vec
                let (l, r) = self.knots.split_at_mut(inds[1]);
                Self::resolve_knot_pair(&l[inds[0]], &mut r[0]);
            });
    }

    fn resolve_knot_pair(a: &Vector, b: &mut Vector) {
        let diff = *a - *b;
        let (dist_x, dist_y) = diff.abs().into();
        if dist_x > 1 || dist_y > 1 {
            *b = *b + diff.sign();
        }
    }
}

fn main() {
    // Parse input
    let input = read_to_string("./input.txt").unwrap();
    let actions = actions_from_str(&input);

    // Move rope around
    let mut rope = Rope::new(1);
    let tail_positions = rope.track_tail_positions(&actions);
    dbg!(tail_positions.len());

    // Move a bigger rope around
    let mut big_rope = Rope::new(9);
    let tail_positions = big_rope.track_tail_positions(&actions);
    dbg!(tail_positions.len());
}

#[cfg(test)]
#[test]
fn test_with_puzzle_sample() {
    let input = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
    let actions = actions_from_str(input);
    let mut rope = Rope::new(1);
    let tail_positions = rope.track_tail_positions(&actions);
    dbg!(tail_positions.len());
    assert_eq!(tail_positions.len(), 13);
}
