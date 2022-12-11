use itertools::Itertools;
use std::{collections::HashMap, hash::Hash, ops::AddAssign, str::FromStr};

use common::aoc_input;

#[derive(Debug, Clone, Copy)]
struct DivisibleTest(usize);

impl From<usize> for DivisibleTest {
    fn from(divisor: usize) -> Self {
        Self(divisor)
    }
}

#[derive(Clone, Copy)]
enum Operand {
    Value(usize),
    PreviousValue,
}

#[derive(Clone, Copy)]
enum Operation {
    Add(Operand, Operand),
    Mul(Operand, Operand),
}

struct MonkeyThrowResult {
    item: usize,
    to: usize,
}

impl FromStr for Operation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut symbol = None;
        let mut operands = vec![];
        for component in s.split(' ') {
            match component {
                "+" => symbol = Some(component),
                "*" => symbol = Some(component),
                "old" => operands.push(Operand::PreviousValue),
                v => operands.push(Operand::Value(v.parse::<usize>().unwrap())),
            }
        }
        Ok(match symbol {
            Some("+") => Self::Add(operands[0], operands[1]),
            Some("*") => Self::Mul(operands[0], operands[1]),
            _ => panic!("Unknown symbol"),
        })
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<usize>,
    operation: Operation,
    test: DivisibleTest,
    test_actions: (usize, usize),

    /// Whether worry level is divided by 3 after an inspection
    ///     false -> do divide,
    ///     true -> don't divide,
    extra_intimidating: bool,
}

impl FromStr for Monkey {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (starting_items, operation, test_cond, test_action_1, test_action_2) = s
            .lines()
            .skip(1)
            .collect_tuple::<(_, _, _, _, _)>()
            .ok_or("missing components")?;
        let items: Vec<usize> = starting_items
            .split(": ")
            .nth(1)
            .ok_or("missing items")?
            .split(',')
            .flat_map(|num| FromStr::from_str(num.strip_prefix(' ').unwrap_or(num)))
            .collect();
        let test: usize = take_first(test_cond).ok_or("cant parse test condition")?;
        let test_action_1 = take_first(test_action_1).ok_or("cant parse test action 1")?;
        let test_action_2 = take_first(test_action_2).ok_or("cant parse test action 2")?;
        let operation = operation.split("= ").nth(1).unwrap().parse().unwrap();
        Ok(Monkey {
            items,
            test: test.into(),
            operation,
            test_actions: (test_action_1, test_action_2),
            extra_intimidating: false,
        })
    }
}

impl Monkey {
    fn inspect_item(&self, item: usize, lcm: Option<usize>) -> MonkeyThrowResult {
        // Apply operation
        let item = self.operation.apply(item);

        // Divide by three (if not intimidating)
        let item = if self.extra_intimidating {
            if let Some(lcm) = lcm {
                item % lcm
            } else {
                item
            }
        } else {
            item / 3
        };

        // Perform test
        let to = if self.test.test(item) {
            self.test_actions.0
        } else {
            self.test_actions.1
        };

        // Return result
        MonkeyThrowResult { item, to }
    }
}

impl DivisibleTest {
    fn test(&self, value: usize) -> bool {
        value % self.0 == 0
    }
}

impl Operand {
    fn get(&self, previous: usize) -> usize {
        match self {
            Operand::Value(v) => *v,
            Operand::PreviousValue => previous,
        }
    }
}

impl Operation {
    fn apply(&self, item: usize) -> usize {
        match self {
            Operation::Add(x, y) => x.get(item) + y.get(item),
            Operation::Mul(x, y) => x.get(item) * y.get(item),
        }
    }
}

fn perform_monkey_round(monkeys: &mut [Monkey], lcm: Option<usize>) -> HashMap<usize, usize> {
    let mut inspection_counts = HashMap::new();
    for i in 0..monkeys.len() {
        // Drain monkeys current items
        let to_inspect = monkeys[i].items.drain(0..).collect_vec();

        // Inspect each item in turn and throw it to recipient monkey
        for item in to_inspect {
            let result = monkeys[i].inspect_item(item, lcm);
            monkeys[result.to].items.push(result.item);
            *inspection_counts.entry(i).or_insert(0) += 1;
        }
    }
    inspection_counts
}

fn main() {
    // Parse input
    let input = aoc_input!();
    let monkeys: Vec<_> = input.split("\n\n").flat_map(Monkey::from_str).collect();
    part1(monkeys.clone());
    part2(monkeys);
}

fn part1(mut monkeys: Vec<Monkey>) {
    // Perform 20 monkey rounds
    let inspection_counts = sum_hashmaps(
        (0..20)
            .map(|_| perform_monkey_round(&mut monkeys, None))
            .collect(),
    )
    .unwrap();

    // Find busiest monkeys
    let monkey_business: usize = inspection_counts.values().sorted().rev().take(2).product();
    println!("[PT1] level of monkey business is {}", monkey_business);
}

fn part2(mut monkeys: Vec<Monkey>) {
    // Set monkeys as intimidating
    for monkey in monkeys.iter_mut() {
        monkey.extra_intimidating = true;
    }

    // Compute LCM of divisors
    let lcm: usize = monkeys.iter().map(|monkey| monkey.test.0).product();

    // Perform 10000 monkey rounds
    let inspection_counts = sum_hashmaps(
        (0..10000)
            .map(|_| perform_monkey_round(&mut monkeys, Some(lcm)))
            .collect(),
    )
    .unwrap();

    // Find busiest monkeys
    let monkey_business: usize = inspection_counts.values().sorted().rev().take(2).product();
    println!("[PT2] level of monkey business is {}", monkey_business);
}

/* Util */

/// Take first whitespace-seperated segment of string that can be parsed into desired type
fn take_first<V>(s: &str) -> Option<V>
where
    V: FromStr,
{
    s.split(' ').flat_map(|v| v.parse()).next()
}

/// Combine hashmaps by summing corresponding values
fn sum_hashmaps<K: Eq + Hash, V: AddAssign>(maps: Vec<HashMap<K, V>>) -> Option<HashMap<K, V>> {
    maps.into_iter().reduce(|mut a, b| {
        for (key, val) in b {
            a.entry(key).and_modify(|x| *x += val);
        }
        a
    })
}

/* Display Implementations */

impl std::fmt::Display for Monkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Items: {}", self.items.iter().join(", "))
    }
}

impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add(x, y) => write!(f, "{:?} + {:?}", x, y),
            Operation::Mul(x, y) => write!(f, "{:?} * {:?}", x, y),
        }
    }
}

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Value(value) => write!(f, "{}", value),
            Operand::PreviousValue => write!(f, "old"),
        }
    }
}

impl std::fmt::Debug for MonkeyThrowResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "throw {} to {}", self.item, self.to)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_monkey_inspection_single_round() {
        let mut monkeys: Vec<_> = read_to_string("./sample.txt")
            .unwrap()
            .split("\n\n")
            .flat_map(Monkey::from_str)
            .collect();
        perform_monkey_round(&mut monkeys, None);
        assert_eq!(monkeys[0].items, vec![20, 23, 27, 26]);
        assert_eq!(monkeys[1].items, vec![2080, 25, 167, 207, 401, 1046]);
        assert!(monkeys[2].items.is_empty());
        assert!(monkeys[3].items.is_empty());
    }

    #[test]
    fn test_monkey_inspection_twenty_rounds() {
        let mut monkeys: Vec<_> = read_to_string("./sample.txt")
            .unwrap()
            .split("\n\n")
            .flat_map(Monkey::from_str)
            .collect();
        let inspection_counts = sum_hashmaps(
            (0..20)
                .map(|_| perform_monkey_round(&mut monkeys, None))
                .collect(),
        )
        .unwrap();
        let monkey_business: usize = inspection_counts.values().sorted().rev().take(2).product();
        assert_eq!(inspection_counts[&0], 101);
        assert_eq!(inspection_counts[&1], 95);
        assert_eq!(inspection_counts[&2], 7);
        assert_eq!(inspection_counts[&3], 105);
        assert_eq!(monkey_business, 10605);
    }
}
