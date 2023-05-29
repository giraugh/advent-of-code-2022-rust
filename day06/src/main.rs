use std::{collections::HashSet, fs::read_to_string};

fn main() {
    let input = read_to_string("./input.txt").unwrap();
    println!("[PT1] {}", find_packet_start(input.chars(), 4).unwrap());
    println!("[PT2] {}", find_packet_start(input.chars(), 14).unwrap());
}

fn find_packet_start(stream: impl Iterator<Item = char>, buffer_size: usize) -> Option<usize> {
    stream
        .collect::<Vec<_>>()
        .windows(buffer_size)
        .enumerate()
        .take_while(|(_, window)| window.iter().collect::<HashSet<_>>().len() < buffer_size)
        .last()
        .map(|(i, _)| i + buffer_size + 1)
}
