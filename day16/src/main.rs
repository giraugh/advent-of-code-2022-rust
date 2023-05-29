use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};

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

#[derive(Default, Hash, Eq, PartialEq, Clone, Debug)]
pub struct OpenValves(u64);

impl OpenValves {
    fn open(&self, id: ValveID) -> Self {
        Self(self.0 | 1 << id.0)
    }

    #[allow(dead_code)]
    fn close(&self, id: ValveID) -> Self {
        Self(self.0 & 0 << id.0)
    }

    fn invert(&self) -> Self {
        Self(!self.0)
    }

    fn is_open(&self, id: ValveID) -> bool {
        (self.0 >> id.0) & 1 == 1
    }

    fn iter(&self) -> impl Iterator<Item = ValveID> + '_ {
        (0..64).filter(|i| (self.0 >> i) & 1 == 1).map(ValveID)
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct ValveID(usize);

impl From<usize> for ValveID {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct ValveNetwork {
    start_position: ValveID,
    flow_rates: HashMap<ValveID, usize>,
    edges: HashMap<ValveID, Vec<ValveID>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub enum ValveAction {
    MoveTo(ValveID),
    Open,
}

mod part1 {
    use super::*;

    #[derive(Clone)]
    pub struct NetworkPlan<'a> {
        network: &'a ValveNetwork,
        actions: Vec<ValveAction>,
    }

    impl<'a> NetworkPlan<'a> {
        pub fn total_pressure_released(&self, minutes: usize) -> Result<usize, &'static str> {
            let mut released = 0;
            let mut open_valves = OpenValves::default();
            let mut current_position = self.network.start_position;

            for minute in 0..minutes - 1 {
                // Perform action
                if let Some(action) = self.actions.get(minute) {
                    match action {
                        ValveAction::MoveTo(valve_id) => {
                            if !self.network.edges[&current_position].contains(valve_id) {
                                return Err("Cannot move to valve from current valve");
                            }
                            current_position = *valve_id;
                        }
                        ValveAction::Open => {
                            open_valves = open_valves.open(current_position);
                        }
                    }
                }

                // Add to flow rate
                released += open_valves
                    .iter()
                    .map(|valve_id| self.network.flow_rates[&valve_id])
                    .sum::<usize>();
            }

            Ok(released)
        }

        /// Find the sequence of actions which maximises the flow rate
        pub fn solve(network: &ValveNetwork, action_count: usize, minutes: usize) -> NetworkPlan {
            let initial_state = NetworkState {
                current_position: network.start_position,
                open_valves: OpenValves::default(),
                parent: None,
                action: None,
                depth: 0,
            };
            let mut frontier: VecDeque<Rc<NetworkState>> = vec![Rc::new(initial_state)].into();
            let mut flow_rates_cache: HashMap<Rc<NetworkState>, usize> = HashMap::new();

            // Explore graph
            while let Some(state) = frontier.pop_front() {
                // Expand frontier with children
                if state.depth <= action_count {
                    for child in NetworkState::expand(Rc::clone(&state), network) {
                        let child = Rc::new(child);
                        let rate = NetworkState::total_pressure_released(
                            Rc::clone(&child),
                            network,
                            minutes,
                        );
                        if let Some(current_flow_rate) = flow_rates_cache.get(&child) {
                            if rate > *current_flow_rate {
                                flow_rates_cache.remove(&child);
                                flow_rates_cache.insert(Rc::clone(&child), rate);
                                frontier.push_back(child);
                            }
                        } else {
                            let child = Rc::new(child);
                            flow_rates_cache.insert(Rc::clone(&child), rate);
                            frontier.push_back(Rc::clone(&child));
                        }
                    }
                }
            }

            // Find best path
            let (best_state, _) = flow_rates_cache
                .into_iter()
                .filter(|(state, _)| state.depth == action_count)
                .sorted_by_key(|(_, rate)| *rate)
                .last()
                .unwrap();
            let actions = NetworkState::backtrack(best_state);
            debug_assert_eq!(actions.len(), action_count);

            NetworkPlan { network, actions }
        }
    }

    #[derive(Eq, Clone)]
    struct NetworkState {
        current_position: ValveID,
        open_valves: OpenValves,
        parent: Option<Rc<NetworkState>>,
        action: Option<ValveAction>,
        depth: usize,
    }

    impl PartialEq for NetworkState {
        fn eq(&self, other: &Self) -> bool {
            (self.current_position == other.current_position)
                && (self.open_valves == other.open_valves)
                && (self.depth == other.depth)
        }
    }

    impl Hash for NetworkState {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.current_position.hash(state);
            self.open_valves.hash(state);
            self.depth.hash(state);
        }
    }

    impl NetworkState {
        fn backtrack(state: Rc<NetworkState>) -> Vec<ValveAction> {
            let mut current = state;
            let mut actions = vec![current.action.unwrap()];
            while let Some(node) = &current.parent {
                current = Rc::clone(node);
                if let Some(action) = &current.action {
                    actions.push(*action);
                }
            }
            actions.reverse();
            actions
        }

        fn expand(parent: Rc<NetworkState>, network: &ValveNetwork) -> Vec<NetworkState> {
            let mut children = Vec::new();

            // Add open commands
            // (only open if not already open and flow rate > 0)
            if !parent.open_valves.is_open(parent.current_position)
                && network.flow_rates[&parent.current_position] > 0
            {
                let state = NetworkState {
                    open_valves: parent.open_valves.open(parent.current_position),
                    parent: Some(Rc::clone(&parent)),
                    action: Some(ValveAction::Open),
                    depth: parent.depth + 1,
                    ..*parent
                };
                children.push(state);
            }

            // Add move commands
            let possible_positions = &network.edges[&parent.current_position];
            for location in possible_positions {
                let state = NetworkState {
                    current_position: *location,
                    open_valves: parent.open_valves.clone(),
                    parent: Some(Rc::clone(&parent)),
                    action: Some(ValveAction::MoveTo(*location)),
                    depth: parent.depth + 1,
                };
                children.push(state);
            }

            children
        }

        fn total_pressure_released(
            state: Rc<NetworkState>,
            network: &ValveNetwork,
            minutes: usize,
        ) -> usize {
            let actions = Self::backtrack(Rc::clone(&state));
            let plan = NetworkPlan { network, actions };
            plan.total_pressure_released(minutes).unwrap()
        }
    }

    impl<'a> std::fmt::Debug for NetworkPlan<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.actions)
        }
    }

    impl std::fmt::Debug for NetworkState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "depth={} action={:?} cp={:?} parent?={}",
                self.depth,
                self.action,
                self.current_position,
                self.parent.is_some()
            )
        }
    }

    #[cfg(test)]
    mod test_with_sample {
        use super::*;

        const SAMPLE_INPUT: &str = include_str!("../sample.txt");

        fn get_sample_plan() -> Vec<ValveAction> {
            vec![
                ValveAction::MoveTo(3.into()),
                ValveAction::Open,
                ValveAction::MoveTo(2.into()),
                ValveAction::MoveTo(1.into()),
                ValveAction::Open,
                ValveAction::MoveTo(0.into()),
                ValveAction::MoveTo(8.into()),
                ValveAction::MoveTo(9.into()),
                ValveAction::Open,
                ValveAction::MoveTo(8.into()),
                ValveAction::MoveTo(0.into()),
                ValveAction::MoveTo(3.into()),
                ValveAction::MoveTo(4.into()),
                ValveAction::MoveTo(5.into()),
                ValveAction::MoveTo(6.into()),
                ValveAction::MoveTo(7.into()),
                ValveAction::Open,
                ValveAction::MoveTo(6.into()),
                ValveAction::MoveTo(5.into()),
                ValveAction::MoveTo(4.into()),
                ValveAction::Open,
                ValveAction::MoveTo(3.into()),
                ValveAction::MoveTo(2.into()),
                ValveAction::Open,
            ]
        }

        #[test]
        fn test_parse_sample() {
            let network = SAMPLE_INPUT.parse::<ValveNetwork>();
            assert!(network.is_ok(), "Failed to parse sample network");
        }

        #[test]
        fn test_flow_rate_calc() {
            let network = SAMPLE_INPUT.parse::<ValveNetwork>().unwrap();
            let actions = get_sample_plan();
            let plan = NetworkPlan {
                network: &network,
                actions,
            };
            assert_eq!(plan.total_pressure_released(30), Ok(1651));
        }

        #[test]
        fn test_solve_sample() {
            let network = SAMPLE_INPUT.parse::<ValveNetwork>().unwrap();
            let plan = NetworkPlan::solve(&network, 30, 30);
            dbg!(&plan);
            let pressure_released = plan.total_pressure_released(30).unwrap_or(0);
            assert_eq!(pressure_released, 1651);
            assert_eq!(
                plan.actions.into_iter().take(24).collect_vec(),
                get_sample_plan()
            )
        }
    }
}

mod part2 {
    use priority_queue::PriorityQueue;

    use super::*;

    type SimultaneousAction = (ValveAction, ValveAction);

    #[derive(Clone)]
    pub struct NetworkPlan<'a> {
        network: &'a ValveNetwork,
        actions: Vec<SimultaneousAction>,
    }

    impl<'a> NetworkPlan<'a> {
        pub fn total_pressure_released(&self, minutes: usize) -> Result<usize, &'static str> {
            // Init released amount
            let mut released = 0;

            // Init graph state
            let mut open_valves = OpenValves::default();
            let mut human_position = self.network.start_position;
            let mut elephant_position = self.network.start_position;

            for minute in 0..minutes - 1 {
                // Perform action
                if let Some((human_action, elephant_action)) = self.actions.get(minute) {
                    // Resolve human action
                    match human_action {
                        ValveAction::MoveTo(valve_id) => {
                            if !self.network.edges[&human_position].contains(valve_id) {
                                return Err("Cannot move to valve from current valve");
                            }
                            human_position = *valve_id;
                        }
                        ValveAction::Open => {
                            open_valves = open_valves.open(human_position);
                        }
                    }

                    // Resolve elephant action
                    match elephant_action {
                        ValveAction::MoveTo(valve_id) => {
                            if !self.network.edges[&elephant_position].contains(valve_id) {
                                return Err("Cannot move to valve from current valve");
                            }
                            elephant_position = *valve_id;
                        }
                        ValveAction::Open => {
                            open_valves = open_valves.open(elephant_position);
                        }
                    }
                }

                // Add to flow rate
                released += open_valves
                    .iter()
                    .map(|valve_id| self.network.flow_rates[&valve_id])
                    .sum::<usize>();
            }

            Ok(released)
        }

        /// Find the sequence of actions which maximises the flow rate
        pub fn solve(network: &ValveNetwork, action_count: usize, minutes: usize) -> NetworkPlan {
            let initial_state = NetworkState {
                human_position: network.start_position,
                elephant_position: network.start_position,
                open_valves: OpenValves::default(),
                parent: None,
                action: None,
                depth: 0,
            };
            let mut frontier: PriorityQueue<Rc<NetworkState>, usize> =
                vec![(Rc::new(initial_state), 0)].into();
            let mut flow_rates_cache: HashMap<Rc<NetworkState>, usize> = HashMap::new();
            let mut best_at_depth: HashMap<usize, usize> = HashMap::new();

            // Explore graph
            while let Some((state, rate)) = frontier.pop() {
                // Expand frontier with children
                if state.depth < action_count {
                    for child in NetworkState::expand(Rc::clone(&state), network) {
                        // Compute rate of this child
                        let child = Rc::new(child);
                        let rate = NetworkState::total_pressure_released(
                            Rc::clone(&child),
                            network,
                            minutes,
                        );

                        // Can we even beat the best performer?
                        let best_at_this_depth = *best_at_depth.get(&child.depth).unwrap_or(&0);
                        if rate > best_at_this_depth {
                            best_at_depth.insert(child.depth, rate);
                            eprintln!("better w/ {} @ {}", rate, child.depth);
                        }

                        // This is really hacky, I dont wanna talk about it
                        let best_at_prev_depth = *best_at_depth
                            .get(&child.depth.saturating_sub(3))
                            .unwrap_or(&0);
                        if rate < best_at_prev_depth {
                            continue;
                        }

                        // Add children
                        let current_flow_for_state = flow_rates_cache.get(&child);
                        if Some(rate) > current_flow_for_state.copied() {
                            flow_rates_cache.remove(&child);
                            flow_rates_cache.insert(Rc::clone(&child), rate);
                            frontier.push(child, rate);
                        }
                    }
                }
            }

            // Find best path
            let (best_state, _) = flow_rates_cache
                .into_iter()
                .filter(|(state, _)| state.depth == action_count)
                .sorted_by_key(|(_, rate)| *rate)
                .last()
                .unwrap();
            let actions = NetworkState::backtrack(best_state);
            // debug_assert_eq!(actions.len(), action_count);

            NetworkPlan { network, actions }
        }
    }

    #[derive(Eq, Clone)]
    struct NetworkState {
        human_position: ValveID,
        elephant_position: ValveID,
        open_valves: OpenValves,
        parent: Option<Rc<NetworkState>>,
        action: Option<SimultaneousAction>,
        depth: usize,
    }

    impl PartialEq for NetworkState {
        fn eq(&self, other: &Self) -> bool {
            let (a, b) = if self.human_position < self.elephant_position {
                (self.human_position, self.elephant_position)
            } else {
                (self.elephant_position, self.human_position)
            };

            let (oa, ob) = if other.human_position < other.elephant_position {
                (other.human_position, other.elephant_position)
            } else {
                (other.elephant_position, other.human_position)
            };

            (a == oa)
                && (b == ob)
                && (self.open_valves == other.open_valves)
                && (self.depth == other.depth)
        }
    }

    impl Hash for NetworkState {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            let (a, b) = if self.human_position < self.elephant_position {
                (self.human_position, self.elephant_position)
            } else {
                (self.elephant_position, self.human_position)
            };

            a.hash(state);
            b.hash(state);
            self.open_valves.hash(state);
            self.depth.hash(state);
        }
    }

    impl NetworkState {
        fn backtrack(state: Rc<NetworkState>) -> Vec<SimultaneousAction> {
            let mut current = state;
            let mut actions = vec![current.action.unwrap()];
            while let Some(node) = &current.parent {
                current = Rc::clone(node);
                if let Some(action) = &current.action {
                    actions.push(*action);
                }
            }
            actions.reverse();
            actions
        }

        fn possible_actions_from(
            parent: Rc<NetworkState>,
            network: &ValveNetwork,
            current_position: ValveID,
        ) -> Vec<ValveAction> {
            let mut actions = Vec::new();

            // Open command
            if !parent.open_valves.is_open(current_position)
                && network.flow_rates[&current_position] > 0
            {
                actions.push(ValveAction::Open);
            }

            // Add move commands
            let possible_positions = &network.edges[&current_position];
            for location in possible_positions {
                actions.push(ValveAction::MoveTo(*location));
            }

            actions
        }

        fn expand(parent: Rc<NetworkState>, network: &ValveNetwork) -> Vec<NetworkState> {
            // Get possible actions
            let human_actions =
                Self::possible_actions_from(Rc::clone(&parent), network, parent.human_position);
            let elephant_actions =
                Self::possible_actions_from(Rc::clone(&parent), network, parent.elephant_position);

            // Return all combinations
            Itertools::cartesian_product(human_actions.into_iter(), elephant_actions.into_iter())
                .flat_map(|(human_action, elephant_action)| {
                    if human_action == ValveAction::Open
                        && elephant_action == ValveAction::Open
                        && parent.human_position == parent.elephant_position
                    {
                        return None;
                    }

                    Some(NetworkState {
                        action: Some((human_action, elephant_action)),
                        depth: parent.depth + 1,
                        human_position: match human_action {
                            ValveAction::MoveTo(position) => position,
                            _ => parent.human_position,
                        },
                        elephant_position: match elephant_action {
                            ValveAction::MoveTo(position) => position,
                            _ => parent.elephant_position,
                        },
                        parent: Some(Rc::clone(&parent)),
                        open_valves: {
                            let mut ov = parent.open_valves.clone();
                            if human_action == ValveAction::Open {
                                ov = ov.open(parent.human_position);
                            }
                            if elephant_action == ValveAction::Open {
                                ov = ov.open(parent.elephant_position);
                            }
                            ov
                        },
                    })
                })
                .collect_vec()
        }

        fn total_pressure_released(
            state: Rc<NetworkState>,
            network: &ValveNetwork,
            minutes: usize,
        ) -> usize {
            let actions = Self::backtrack(Rc::clone(&state));
            let plan = NetworkPlan { network, actions };
            plan.total_pressure_released(minutes).unwrap()
        }
    }

    impl<'a> std::fmt::Debug for NetworkPlan<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.actions)
        }
    }

    impl std::fmt::Debug for NetworkState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "depth={} action={:?} hp={:?} ep={:?} parent?={}",
                self.depth,
                self.action,
                self.human_position,
                self.elephant_position,
                self.parent.is_some()
            )
        }
    }

    #[cfg(test)]
    mod test_with_sample {
        use super::*;

        const SAMPLE_INPUT: &str = include_str!("../sample.txt");

        macro_rules! action {
            (-> $c:expr) => {{
                let num = ((($c).to_uppercase().chars().next().unwrap() as u8) - b'A') as usize;
                ValveAction::MoveTo(num.into())
            }};
            (*) => {
                ValveAction::Open
            };
        }

        fn get_sample_plan() -> Vec<SimultaneousAction> {
            vec![
                (action!(-> "II"), action!(-> "DD")),
                (action!(-> "JJ"), action!(*)),
                (action!(*), action!(-> "EE")),
                (action!(-> "II"), action!(-> "FF")),
                (action!(-> "AA"), action!(-> "GG")),
                (action!(-> "BB"), action!(-> "HH")),
                (action!(*), action!(*)),
                (action!(-> "CC"), action!(-> "GG")),
                (action!(*), action!(-> "FF")),
                (action!(*), action!(-> "EE")),
                (action!(*), action!(*)),
            ]
        }

        #[test]
        fn test_flow_rate_calc() {
            let network = SAMPLE_INPUT.parse::<ValveNetwork>().unwrap();
            let actions = get_sample_plan();
            dbg!(&actions);
            let plan = NetworkPlan {
                network: &network,
                actions,
            };
            assert_eq!(plan.total_pressure_released(26), Ok(1707));
        }

        // #[test]
        // fn test_solve_sample() {
        //     let network = SAMPLE_INPUT.parse::<ValveNetwork>().unwrap();
        //     let plan = part1::NetworkPlan::solve(&network, 30, 30);
        //     dbg!(&plan);
        //     let pressure_released = plan.total_pressure_released(30).unwrap_or(0);
        //     assert_eq!(pressure_released, 1651);
        //     assert_eq!(
        //         plan.actions.into_iter().take(24).collect_vec(),
        //         get_sample_plan()
        //     )
        // }
    }
}

fn main() {
    let input = aoc_input!();
    let network: ValveNetwork = input.parse().unwrap();
    // let plan = part1::NetworkPlan::solve(&network, 30, 30);
    // println!("[PT1] {}", plan.total_pressure_released(30).unwrap());
    let plan = part2::NetworkPlan::solve(&network, 26, 26);
    println!("[PT2] {}", plan.total_pressure_released(26).unwrap());
}

/* Parsing */

impl std::str::FromStr for ValveNetwork {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flow_rates: HashMap<String, usize> = HashMap::new();
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();

        // Parse lines
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

        // Convert valve ids to integers
        let mut valve_ids: HashMap<String, ValveID> = HashMap::new();
        for valve_str_id in flow_rates.keys().sorted() {
            valve_ids.insert(valve_str_id.to_string(), valve_ids.len().into());
        }

        Ok(Self {
            start_position: valve_ids
                .iter()
                .find(|&(k, _)| k == "AA")
                .map(|(_, v)| *v)
                .unwrap(),
            flow_rates: flow_rates.iter().map(|(k, &v)| (valve_ids[k], v)).collect(),
            edges: edges
                .iter()
                .map(|(k, v)| (valve_ids[k], v.iter().map(|id| valve_ids[id]).collect()))
                .collect(),
        })
    }
}

/* Display impls */

impl std::fmt::Debug for ValveID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
