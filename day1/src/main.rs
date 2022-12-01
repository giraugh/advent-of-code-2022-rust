use std::fs::read_to_string;

fn main() {
    // Parse input
    let input_text = read_to_string("./input.txt").unwrap();
    let mut inventories: Vec<usize> = input_text
        .split("\n\n")
        .map(|chunk| chunk.lines().map(|l| l.parse::<usize>().unwrap()).sum())
        .collect();

    // Part 1
    let max = inventories.iter().max();
    dbg!(max);

    // Part 2
    inventories.sort();
    let sum: usize = inventories.iter().rev().take(3).sum();
    dbg!(sum);
}
