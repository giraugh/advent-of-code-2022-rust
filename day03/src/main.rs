use std::collections::HashSet;

struct Rucksack {
    compartment_1: Vec<char>,
    compartment_2: Vec<char>,
}

pub fn common_char(groups_it: impl IntoIterator<Item = Vec<char>>) -> Option<char> {
    groups_it
        .into_iter()
        .map(|group| HashSet::from_iter(group.into_iter()))
        .reduce(|intersection, set| {
            intersection
                .into_iter()
                .filter(|c| set.contains(c))
                .collect::<HashSet<_>>()
        })
        .and_then(|set| set.into_iter().next())
}

impl Rucksack {
    pub fn common_item(&self) -> Option<char> {
        common_char(vec![self.compartment_1.clone(), self.compartment_2.clone()])
    }

    pub fn all_items(&self) -> Vec<char> {
        let mut items = self.compartment_1.clone();
        items.extend(self.compartment_2.iter());
        items
    }

    pub fn common_item_in_group(rucksacks: &[Rucksack]) -> Option<char> {
        common_char(rucksacks.iter().map(|rucksack| rucksack.all_items()))
    }

    pub fn item_priority(ch: char) -> u8 {
        let ord = ch as u8;
        if ch.is_uppercase() {
            ord - b'A' + 27
        } else {
            ord - b'a' + 1
        }
    }
}

fn main() {
    // Parse input into rucksacks
    let rucksacks = include_str!("../input.txt").lines().map(|line| {
        let comp_size = line.len() / 2;
        Rucksack {
            compartment_1: line.chars().take(comp_size).collect(),
            compartment_2: line.chars().skip(comp_size).take(comp_size).collect(),
        }
    });

    part1(rucksacks.clone());
    part2(rucksacks.clone());
}

fn part1(rucksacks: impl Iterator<Item = Rucksack>) {
    // Sum priorities
    let prio_sum: usize = rucksacks
        .map(|r| Rucksack::item_priority(r.common_item().unwrap()) as usize)
        .sum();
    dbg!(prio_sum);
}

fn part2(rucksacks: impl Iterator<Item = Rucksack>) {
    let rucksacks: Vec<_> = rucksacks.collect();
    let prio_sum: usize = rucksacks
        .as_slice()
        .chunks_exact(3)
        .map(|group| Rucksack::common_item_in_group(group).unwrap())
        .map(|item| Rucksack::item_priority(item) as usize)
        .sum();
    dbg!(prio_sum);
}

#[cfg(test)]
#[test]
fn test_item_prio() {
    assert_eq!(Rucksack::item_priority('a'), 1);
    assert_eq!(Rucksack::item_priority('p'), 16);
    assert_eq!(Rucksack::item_priority('t'), 20);
    assert_eq!(Rucksack::item_priority('A'), 27);
    assert_eq!(Rucksack::item_priority('Z'), 52);
}
