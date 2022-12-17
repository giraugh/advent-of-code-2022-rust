use std::collections::{HashMap, HashSet};

use common::aoc_input;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    error::ErrorKind,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

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

impl<'a> NetworkPlan<'a> {
    fn total_pressure_released(&self) -> Result<usize, &'static str> {
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
                        return Err("Cannot move to valve from current valve");
                    }
                    current_position = valve_id;
                }
                ValveAction::Open => {
                    open_valves.insert(current_position);
                }
            }
        }

        Ok(released)
    }
}

impl ValveNetwork {
    /// Find the sequence of actions which maximises the flow rate
    fn solve(&self, action_count: usize) -> NetworkPlan {
        NetworkPlan {
            network: self,
            actions: vec![ValveAction::DoNothing; action_count],
        }
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
