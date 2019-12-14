#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::collections::{HashMap, HashSet};

use csv::{Trim, ReaderBuilder};
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, IntoNodeReferences};
use petgraph::dot::{Dot, Config};
use std::ops::{Index, IndexMut};


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Person {
    id: usize,              // 0
    generation: isize,
    first_name: String,     // 1
//    middle_name: String,  // 2
    last_name: String,      // 3
    mother: Option<usize>,    // 4
    father: Option<usize>,    // 5
    dob: String,            // 6
    dod: String,            // 7
//    pob: String,
//    pod: String,
//    notes: String
}

#[derive(Debug)]
enum Parent {
    Mother,
    Father
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .flexible(true)
        .from_path(Path::new("family_tree.csv")).expect("Error opening family tree");

    let mut person_map = HashMap::new();
    let mut family_tree = Graph::<Person, Parent>::new();

    // read in the CSV and construct the graph
    for result in csv_reader.records() {
        if let Err(e) = result {
            panic!("Error reading CSV file: {:?}", e);
        }

        let record = result.expect("Error reading CSV file");

        // go through each field so we can handle blanks more gracefully
        let person_id = if let Some(id) = record.get(0) {
            id.parse::<usize>().unwrap()
        } else {
            panic!("Every person must have an ID: {:#?}", record.position().unwrap())
        };

        let person = Person {
            id: person_id,
            generation: -1,  // start everyone with -1 as their generation
            first_name: String::from(if let Some(first_name) = record.get(1) { first_name } else { "?" }),
            last_name: String::from(if let Some(ln) = record.get(3) { ln } else { "?" }),
            mother: if let Some(m) = record.get(4) { m.parse::<usize>().ok() } else { None },
            father: if let Some(m) = record.get(5) { m.parse::<usize>().ok() } else { None },
            dob: String::from(if let Some(ln) = record.get(6) { ln } else { "?" }),
            dod: String::from(if let Some(ln) = record.get(7) { ln } else { "?" }),
        };

        // filter out "empty" people
        if person.first_name.is_empty() && person.last_name.is_empty() {
            continue;
        }

        // add the node to the graph
        let node_id = family_tree.add_node(person);

        // insert the person_id -> node_id
        person_map.insert(person_id, node_id);
    }

    let mut edges = Vec::new();

    // go through the people, and add the mother/father edges
    for node in family_tree.raw_nodes() {
        let person = &node.weight;

        // add in the mother to our list of edges
        if person.mother.is_some() && person_map.contains_key(&person.mother.unwrap()) {
            let person_index = person_map.get(&person.id).unwrap();
            let mother_index = person_map.get(&person.mother.unwrap()).unwrap();

            edges.push((*person_index, *mother_index, Parent::Mother));
        }

        // add in the father to our list of edges
        if person.father.is_some() && person_map.contains_key(&person.father.unwrap()) {
            let person_index = person_map.get(&person.id).unwrap();
            let father_index = person_map.get(&person.father.unwrap()).unwrap();

            edges.push((*person_index, *father_index, Parent::Father));
        }
    }

    // add all of the edges for mothers and fathers to the graph
    for (person, parent, p_type) in edges {
        family_tree.add_edge(person, parent, p_type);
    }

    // find all the children
    let mut children = family_tree.node_indices().filter(|n| {
        if family_tree.neighbors_directed(*n, petgraph::Incoming).fold(0, |a, _e| a + 1) == 0 {
            true
        } else {
            false
        }
    }).collect::<Vec<_>>();

    // DEBUG: print all the children
    for child in &children {
        debug!("CHILD: {:?}", family_tree.index(*child));
    }

    // get a unique set of all parents for all the children
    let mut parents = HashSet::new();

    for child in &children {
        let father = person_map.get(&family_tree.index(*child).father.unwrap()).unwrap();
        let mother = person_map.get(&family_tree.index(*child).mother.unwrap()).unwrap();

        parents.insert(*father);
        parents.insert(*mother);
    }

    // DEBUG: print all parents
    for parent in &parents {
        debug!("PARENT: {:?}", family_tree.index(*parent))
    }

    // pick a random child as our base
    let mut base_child= children.pop().expect("No children found in family tree!");
    let mut base_child_length = -1;

    // DFS from child -> parent, keeping track of the longest one
    for child in &children {
        let mut dfs = Dfs::new(&family_tree, *child);
        let mut cur_length = 0;

        while let Some(node) = dfs.next(&family_tree) {
            if parents.contains(&node) {
                cur_length += 1;
            } else {
                break;
            }
        }

        if cur_length > base_child_length {
            base_child = *child;
            base_child_length = cur_length;
        }
    }

    // make sure we actually found a base child
    assert_ne!(base_child_length, -1, "Did not find a base child");

    // set their generation to 0
    family_tree[base_child].generation = 0;

    debug!("BASE CHILD: {:?}", family_tree[base_child]);

    let mut dfs = Dfs::new(&family_tree, base_child);

    // DFS from the base child to everyone, setting their generations
    while let Some(node) = dfs.next(&family_tree) {
        let cur_generation = family_tree[node].generation + 1;

        if let Some(father) = family_tree[node].father {
            let father_node = *person_map.get(&father).unwrap();
            family_tree.index_mut(father_node).generation = cur_generation;
            debug!("FATHER: {:?}", family_tree[father_node]);
        }

        if let Some(mother) = family_tree[node].mother {
            let mother_node = *person_map.get(&mother).unwrap();
            family_tree.index_mut(mother_node).generation = cur_generation;
            debug!("MOTHER: {:?}", family_tree[mother_node]);
        }
    }

    // get all of the unassigned
    let mut unassigned = family_tree.node_indices().filter(|n| {
        family_tree[*n].generation == -1
    }).collect::<Vec<_>>();

    // assign them from their parents
    while !unassigned.is_empty() {
        debug!("UNASSIGNED: {:?}", unassigned);

        for person in &unassigned {
            if let Some(m) = family_tree[*person].mother {
                let mother_generation = family_tree[*person_map.get(&m).unwrap()].generation;

                if mother_generation != -1 {
                    family_tree[*person].generation = mother_generation - 1;
                }
            } else if let Some(f) = family_tree[*person].father {
                let father_generation = family_tree[*person_map.get(&f).unwrap()].generation;

                if father_generation != -1 {
                    family_tree[*person].generation = father_generation - 1;
                }
            }
        }

        unassigned = family_tree.node_indices().filter(|n| {
            family_tree[*n].generation == -1
        }).collect::<Vec<_>>();
    }

    println!("{:?}", Dot::with_config(&family_tree, &[Config::EdgeNoLabel]));
}