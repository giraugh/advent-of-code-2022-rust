use std::{fmt::Display, fs::read_to_string, str::FromStr};

use itertools::Itertools;

// Bottom to top stack
type Stack = Vec<char>;

// Stacks from left to right
#[derive(Debug, Clone)]
struct Stacks(Vec<Stack>);

impl Display for Stacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, stack) in self.0.iter().enumerate() {
            let s = stack.iter().collect::<String>();
            f.write_str(&format!("{} {} \n", i + 1, s))?;
        }
        Ok(())
    }
}

impl Stacks {
    pub fn apply_instruction(&mut self, instruction: &Instruction, move_together: bool) {
        if move_together {
            // Drain the last N items and then push them onto the other
            let from_stack = self.0.get_mut(instruction.from).unwrap();
            let tail_items = from_stack.split_off(from_stack.len() - instruction.amount);
            for item in tail_items {
                self.0.get_mut(instruction.to).unwrap().push(item);
            }
        } else {
            // Repeatedly shift items between stacks
            (0..instruction.amount).for_each(|_| {
                let item = self.0.get_mut(instruction.from).unwrap().pop().unwrap();
                self.0.get_mut(instruction.to).unwrap().push(item);
            });
        }
    }

    pub fn get_stack_tops(&self) -> String {
        self.0
            .iter()
            .flat_map(|stack| stack.iter().last())
            .collect()
    }
}

impl FromStr for Stacks {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove decoration and convert to single row
        let stack_chars = s
            .lines()
            .take_while(|l| !l.chars().next().unwrap().is_whitespace())
            .flat_map(|line| {
                let chars = line.chars().skip(1);
                chars.step_by(4)
            })
            .collect::<String>();

        // Invert stacks to get column vectors
        let mut stacks = (0..9)
            .map(|i| {
                stack_chars
                    .chars()
                    .skip(i)
                    .step_by(9)
                    .filter(|c| !c.is_whitespace())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<_>>();

        // Reverse stacks for use as stacks
        stacks.iter_mut().for_each(|stack| stack.reverse());

        // Return stacks object
        Ok(Stacks(stacks))
    }
}

#[derive(Debug)]
struct Instruction {
    /// Amount of crates to move
    amount: usize,

    /// Index of stack to move from
    from: usize,

    /// Index of stack to move to
    to: usize,
}

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Pull out numbers from string
        let nums = s
            .chars()
            .filter(|c| c.is_whitespace() || c.is_numeric())
            .map(String::from)
            .coalesce(|a, b| {
                if !a.chars().all(|c| c.is_whitespace()) && !b.chars().all(|c| c.is_whitespace()) {
                    Ok(format!("{}{}", a, b))
                } else {
                    Err((a, b))
                }
            })
            .filter(|num| !num.chars().any(|c| c.is_whitespace()))
            .flat_map(|num| num.parse::<usize>());

        // Extract parts
        let (amount, from, to) = nums.collect_tuple().unwrap();
        Ok(Instruction {
            amount,
            from: from - 1,
            to: to - 1,
        })
    }
}

fn main() {
    // Parse input
    let input = read_to_string("./input.txt").unwrap();
    let (stacks, instructions) = input.split_once("\n\n").unwrap();
    let mut stacks: Stacks = stacks.parse().unwrap();
    let instructions: Vec<Instruction> = instructions
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();

    part1(&mut stacks.clone(), &instructions);
    part2(&mut stacks, &instructions);
}

fn part1(stacks: &mut Stacks, instructions: &Vec<Instruction>) {
    // Apply instructions
    for instruction in instructions {
        stacks.apply_instruction(instruction, false);
    }

    // Get top of each stacks
    println!("[PT1] stack tops = {}", stacks.get_stack_tops());
}

fn part2(stacks: &mut Stacks, instructions: &Vec<Instruction>) {
    // Apply instructions
    for instruction in instructions {
        stacks.apply_instruction(instruction, true);
    }

    // Get top of each stacks
    println!("[PT2] stack tops = {}", stacks.get_stack_tops());
}
