use common::aoc_input;
use std::{fs::read_to_string, str::FromStr};

#[derive(Debug, Clone, Copy)]
enum Command {
    Noop,
    Add(isize),
}

impl FromStr for Command {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (command, rest) = s.split_once(' ').unwrap_or((s, ""));
        match command {
            "noop" => Ok(Command::Noop),
            "addx" => Ok(Command::Add(rest.parse().unwrap())),
            _ => Err("unknown command"),
        }
    }
}

type RegisterValue = (usize, isize); // cycle, x-register

struct Cpu {
    register_values: Vec<RegisterValue>,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            register_values: vec![(1, 1)],
        }
    }

    pub fn process_commands(&mut self, commands: &[Command]) {
        for command in commands {
            let &(cycle, x) = self.register_values.last().unwrap();
            self.register_values.extend(
                (match command {
                    Command::Noop => vec![(cycle + 1, x)],
                    Command::Add(add) => vec![(cycle + 1, x), (cycle + 2, x + add)],
                })
                .iter(),
            )
        }
    }

    pub fn signal_strength_sum(&self) -> isize {
        self.register_values
            .iter()
            .take(220)
            .skip(19)
            .step_by(40)
            .map(|&(cycle, x)| (cycle as isize) * x)
            .sum()
    }
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for &(cycle, x) in &self.register_values {
            let cycle = (cycle as isize - 1) % 40;
            let lit = (cycle - 1..=cycle + 1).any(|sp| sp == x);
            write!(f, "{}", if lit { '\u{2588}' } else { ' ' })?;
            if cycle == 39 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn main() {
    let input = aoc_input!();
    let commands: Vec<Command> = input.lines().flat_map(FromStr::from_str).collect();

    // Compute registers
    let mut register = Cpu::new();
    register.process_commands(&commands);
    println!("[PT1] {}", register.signal_strength_sum());

    // Print CRT
    println!("[PT2]\n{}", register);
}

#[test]
fn test_processing_commands_small() {
    let sample = "noop\naddx 3\naddx -5";
    let commands: Vec<Command> = sample.lines().flat_map(FromStr::from_str).collect();
    let mut register = Cpu::new();
    register.process_commands(&commands);
    assert_eq!(register.register_values.get(3), Some(&(4, 4)));
}

#[test]
fn test_processing_commands_large() {
    let sample = read_to_string("./sample.txt").unwrap();
    let commands: Vec<Command> = sample.lines().flat_map(FromStr::from_str).collect();
    let mut register = Cpu::new();
    register.process_commands(&commands);
    assert_eq!(register.register_values.get(19), Some(&(20, 21)));
    assert_eq!(register.signal_strength_sum(), 13140);
    println!("{}", register);
}
