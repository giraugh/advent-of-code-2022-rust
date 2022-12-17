use std::collections::{HashMap, HashSet};

use common::aoc_input;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    error::ErrorKind,
    multi::separated_list0,
    sequence::{preceded, tuple},
};
use rand::{distributions::Uniform, prelude::Distribution, seq::SliceRandom, thread_rng, Rng};
use tqdm::Iter;

type ValveID = String;

#[derive(Debug)]
struct ValveNetwork {
    flow_rates: HashMap<ValveID, usize>,
    edges: HashMap<ValveID, Vec<ValveID>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ValveAction {
    MoveTo(ValveID),
    Open,
    DoNothing,
}

#[derive(Clone)]
struct NetworkPlan<'a> {
    network: &'a ValveNetwork,
    actions: Vec<ValveAction>,
}

struct Jungle<'a> {
    population: Vec<(Option<usize>, NetworkPlan<'a>)>,
    crossover_rate: f64,
    mutation_rate: f64,
    survival_rate: f64,
}

impl<'a> Jungle<'a> {
    fn new(network: &'a ValveNetwork, population_size: usize, plan_size: usize) -> Self {
        // Generate an initial population
        // this population will never open valves, that occurs through mutation
        let mut rng = thread_rng();
        let population = (0..population_size)
            .map(|_| {
                let mut current_position = "AA";
                let actions = (0..plan_size)
                    .map(|_| {
                        // Create an "Open" command?
                        if rng.gen_bool(0.3) && network.flow_rates[current_position] > 0 {
                            ValveAction::Open
                        } else {
                            let next = network.edges[current_position].choose(&mut rng).unwrap();
                            current_position = next;
                            ValveAction::MoveTo(next.to_owned())
                        }
                    })
                    .collect();
                let plan = NetworkPlan { network, actions };
                (plan.total_pressure_released(), plan)
            })
            .collect();

        Jungle {
            population,
            crossover_rate: 0.8,
            mutation_rate: 0.001,
            survival_rate: 0.2,
        }
    }

    /// Perform one step of the genetic algorithm
    fn step(&mut self) {
        // Sort population by fitness
        self.population.sort_by_key(|(fitness, _)| *fitness);

        // Determine proportions
        let breeding_count = (self.population.len() as f64 * self.crossover_rate) as usize;
        let suriviving_count = (self.population.len() as f64 * self.survival_rate) as usize;

        // Determine which plans will breed
        let mut breeding_population = Vec::new();
        breeding_population.extend_from_slice(&self.population[0..breeding_count]);

        // Create offspring
        let mut offspring = Vec::new();
        let mut rng = thread_rng();
        let mate_range = Uniform::new(0, breeding_population.len());
        for plan_index in 0..self.population.len() - suriviving_count - 2 {
            let mate_index = mate_range.sample(&mut rng);
            let new_child = breeding_population[plan_index % breeding_population.len()]
                .1
                .crossover(&breeding_population[mate_index].1);
            offspring.push((Some(0), new_child));
        }

        // Create next generation
        let mut next_generation = Vec::new();
        next_generation.extend_from_slice(&self.population[0..suriviving_count]);
        next_generation.append(&mut offspring);

        // Add a few weak individuals to keep the genetic diversity higher
        next_generation
            .extend_from_slice(&self.population[self.population.len() - 2..self.population.len()]);

        // Mutate population
        for (_, plan) in next_generation.iter_mut() {
            if thread_rng().gen_bool(self.mutation_rate) {
                plan.mutate();
            }
        }

        // Set new population and compute fitness
        self.population = next_generation;

        // Compute population fitnesses
        for (fitness, plan) in self.population.iter_mut() {
            *fitness = plan.total_pressure_released();
        }
    }

    fn best_plan(&self) -> Option<&NetworkPlan<'a>> {
        self.population
            .iter()
            .max_by_key(|(fitness, _)| fitness)
            .map(|(_, plan)| plan)
    }
}

impl<'a> NetworkPlan<'a> {
    fn total_pressure_released(&self) -> Option<usize> {
        let mut released = 0;
        let mut open_valves = HashSet::new();
        let mut current_position = "AA";
        for action in &self.actions {
            // Add to flow rate
            released += open_valves
                .iter()
                .map(|&valve_id| self.network.flow_rates[valve_id])
                .sum::<usize>();

            // Do action
            match action {
                ValveAction::DoNothing => {}
                ValveAction::MoveTo(valve_id) => {
                    if !self.network.edges[current_position].contains(valve_id) {
                        return None;
                    }
                    current_position = valve_id;
                }
                ValveAction::Open => {
                    open_valves.insert(current_position);
                }
            }
        }

        Some(released)
    }

    fn len(&self) -> usize {
        self.actions.len()
    }

    /* GA Stuff */

    /// Merge two solutions to create a new child solution
    fn crossover(&self, other: &Self) -> Self {
        // Choose crossover point
        let mut rng = thread_rng();
        let shorter_len = self.len().min(other.len());
        let crossover_point = Uniform::new(0, shorter_len).sample(&mut rng);

        // Split dna
        let my_dna = &self.actions[0..crossover_point];
        let other_dna = &other.actions[crossover_point..];

        // Generate child dna
        let mut child_actions = Vec::new();
        child_actions.extend_from_slice(my_dna);
        child_actions.extend_from_slice(other_dna);

        NetworkPlan {
            network: self.network,
            actions: child_actions,
        }
    }

    /// Mutate the solution by swapping the order of two actions
    fn mutate(&mut self) {
        let mut rng = thread_rng();
        let action_one = Uniform::new(0, self.len()).sample(&mut rng);
        let action_two = Uniform::new(0, self.len()).sample(&mut rng);
        self.actions.swap(action_one, action_two);
    }
}

impl ValveNetwork {
    /// Find the sequence of actions which maximises the flow rate
    fn solve(&self, action_count: usize) -> NetworkPlan {
        // This is a terrible idea
        let mut jungle = Jungle::new(self, 6000, action_count);
        for _ in (0..3000).tqdm() {
            jungle.step()
        }
        jungle.best_plan().unwrap().clone()
    }
}

fn main() {
    let input = aoc_input!();
    let network: ValveNetwork = input.parse().unwrap();
    let plan = network.solve(30);
    dbg!(&plan);
    dbg!(&plan.total_pressure_released());
}

/* Parsing */

impl std::str::FromStr for ValveNetwork {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flow_rates: HashMap<ValveID, usize> = HashMap::new();
        let mut edges: HashMap<ValveID, Vec<ValveID>> = HashMap::new();

        for line in s.trim_end().lines() {
            // Parse line
            let (id, flow_rate, valve_edges) = tuple::<_, _, (_, ErrorKind), _>((
                preceded(tag("Valve "), complete::alpha1),
                preceded(tag(" has flow rate="), complete::u32),
                preceded(
                    alt((
                        tag("; tunnels lead to valves "),
                        tag("; tunnel leads to valve "),
                    )),
                    separated_list0(tag(", "), complete::alpha1),
                ),
            ))(line)
            .unwrap()
            .1;

            // Add to records
            flow_rates.insert(id.to_owned(), flow_rate as usize);
            edges.insert(
                id.to_owned(),
                valve_edges.into_iter().map(|s| s.to_owned()).collect(),
            );
        }

        Ok(Self { flow_rates, edges })
    }
}

/* Display impls */

impl<'a> std::fmt::Debug for NetworkPlan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.actions)
    }
}

/* Testing */

#[cfg(test)]
mod test_with_sample {
    use itertools::Itertools;

    use crate::{ValveAction, ValveNetwork};

    const SAMPLE_INPUT: &str = include_str!("../sample.txt");

    #[test]
    fn test_parse_sample() {
        let network = SAMPLE_INPUT.parse::<ValveNetwork>();
        assert!(network.is_ok(), "Failed to parse sample network");
    }

    #[test]
    fn test_solve_sample() {
        let network = SAMPLE_INPUT.parse::<ValveNetwork>().unwrap();
        let plan = network.solve(30);
        let pressure_released = plan.total_pressure_released().unwrap_or(0);
        dbg!(plan);
        assert_eq!(pressure_released, 1651);
        // assert_eq!(
        //     plan.actions.into_iter().take(24).collect_vec(),
        //     vec![
        //         ValveAction::MoveTo("DD".to_owned()),
        //         ValveAction::Open,
        //         ValveAction::MoveTo("CC".to_owned()),
        //         ValveAction::MoveTo("BB".to_owned()),
        //         ValveAction::Open,
        //         ValveAction::MoveTo("AA".to_owned()),
        //         ValveAction::MoveTo("II".to_owned()),
        //         ValveAction::MoveTo("JJ".to_owned()),
        //         ValveAction::Open,
        //         ValveAction::MoveTo("II".to_owned()),
        //         ValveAction::MoveTo("AA".to_owned()),
        //         ValveAction::MoveTo("DD".to_owned()),
        //         ValveAction::MoveTo("EE".to_owned()),
        //         ValveAction::MoveTo("FF".to_owned()),
        //         ValveAction::MoveTo("GG".to_owned()),
        //         ValveAction::MoveTo("HH".to_owned()),
        //         ValveAction::Open,
        //         ValveAction::MoveTo("GG".to_owned()),
        //         ValveAction::MoveTo("FF".to_owned()),
        //         ValveAction::MoveTo("EE".to_owned()),
        //         ValveAction::Open,
        //         ValveAction::MoveTo("DD".to_owned()),
        //         ValveAction::MoveTo("CC".to_owned()),
        //         ValveAction::Open,
        //     ]
        // )
    }
}
