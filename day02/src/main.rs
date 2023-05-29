use std::fs::read_to_string;

enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Outcome {
    pub fn score(&self) -> usize {
        match self {
            Outcome::Win => 6,
            Outcome::Draw => 3,
            Outcome::Loss => 0,
        }
    }
}

impl From<&str> for Outcome {
    fn from(string: &str) -> Self {
        match string.chars().next() {
            Some('X') => Outcome::Loss,
            Some('Y') => Outcome::Draw,
            Some('Z') => Outcome::Win,
            _ => panic!("unknown move"),
        }
    }
}

#[derive(Clone, Copy)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    pub fn score(&self) -> usize {
        match self {
            Move::Rock => 1,
            Move::Paper => 2,
            Move::Scissors => 3,
        }
    }

    pub fn outcome_against(&self, other: &Move) -> Outcome {
        match (self, other) {
            (Move::Rock, Move::Paper) => Outcome::Loss,
            (Move::Paper, Move::Rock) => Outcome::Win,
            (Move::Rock, Move::Scissors) => Outcome::Win,
            (Move::Scissors, Move::Rock) => Outcome::Loss,
            (Move::Paper, Move::Scissors) => Outcome::Loss,
            (Move::Scissors, Move::Paper) => Outcome::Win,
            _ => Outcome::Draw,
        }
    }

    pub fn for_outcome_against(&self, outcome: &Outcome) -> Self {
        match (self, outcome) {
            (_, Outcome::Draw) => *self,
            (Move::Rock, Outcome::Win) => Move::Paper,
            (Move::Rock, Outcome::Loss) => Move::Scissors,
            (Move::Paper, Outcome::Win) => Move::Scissors,
            (Move::Paper, Outcome::Loss) => Move::Rock,
            (Move::Scissors, Outcome::Win) => Move::Rock,
            (Move::Scissors, Outcome::Loss) => Move::Paper,
        }
    }
}

impl From<&str> for Move {
    fn from(string: &str) -> Self {
        match string.chars().next() {
            Some('A') | Some('X') => Move::Rock,
            Some('B') | Some('Y') => Move::Paper,
            Some('C') | Some('Z') => Move::Scissors,
            _ => panic!("unknown move"),
        }
    }
}

fn main() {
    let input_text = read_to_string("./input.txt").unwrap();
    part1(&input_text);
    part2(&input_text);
}

fn part1(input_text: &str) {
    // Parse input
    let strategy: Vec<Vec<Move>> = input_text
        .lines()
        .map(|line| line.split(' ').map(|s| s.into()).collect())
        .collect();

    // Compute final score
    let final_score: usize = strategy
        .iter()
        .map(|moves| {
            let (my_move, opp_move) = (&moves[1], &moves[0]);
            my_move.score() + my_move.outcome_against(opp_move).score()
        })
        .sum();

    println!("[PT1] Final Score is {}", final_score);
}

fn part2(input_text: &str) {
    // Parse input
    let strategy: Vec<(Move, Outcome)> = input_text
        .lines()
        .map(|line| {
            let mut segments = line.split(' ');
            (
                segments.next().unwrap().into(),
                segments.next().unwrap().into(),
            )
        })
        .collect();

    // Compute final score
    let final_score: usize = strategy
        .iter()
        .map(|(opp_move, outcome)| {
            let my_move = Move::for_outcome_against(opp_move, outcome);
            outcome.score() + my_move.score()
        })
        .sum();

    println!("[PT2] Final Score is {}", final_score);
}
