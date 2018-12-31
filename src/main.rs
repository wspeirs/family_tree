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


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Person {
    id: usize,              // 0
    generation: isize,
    first_name: String,     // 1
//    middle_name: String,  // 2
    last_name: String,      // 3
    mother: usize,          // 4
    father: usize,          // 5
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
        simple_logger::init_with_level(log::Level::Debug).unwrap();
//    simple_logger::init_with_level(log::Level::Info).unwrap();

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

        let record  = result.expect("Error reading CSV file");

        // go through each field so we can handle blanks more gracefully
        let person_id = if let Some(id) = record.get(0) {
            id.parse::<usize>().unwrap()
        } else {
            panic!("Every person must have an ID: {:#?}", record.position().unwrap())
        };

        let person = Person {
            id: person_id,
            generation: std::isize::MIN,  // start everyone in an unknown generation
            first_name: String::from(if let Some(first_name) = record.get(1) { first_name} else { "?" }),
            last_name: String::from(if let Some(ln) = record.get(3) { ln } else { "?" }),
            mother: if let Some(m) = record.get(4) { m.parse::<usize>().unwrap_or_default() } else { 0 },
            father: if let Some(m) = record.get(5) { m.parse::<usize>().unwrap_or_default() } else { 0 },
            dob: String::from(if let Some(ln) = record.get(6) { ln } else { "?" }),
            dod: String::from(if let Some(ln) = record.get(7) { ln } else { "?" }),
        };

        // check for empty people
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
//        debug!("{:?}", node);

        let person = &node.weight;

        // add in the mother to our list of edges
        if person.mother != 0 && person_map.contains_key(&person.mother) {
            let person_index = person_map.get(&person.id).unwrap();
            let mother_index = person_map.get(&person.mother).unwrap();

            edges.push( (*person_index, *mother_index, Parent::Mother) );
        }

        // add in the father to our list of edges
        if person.father != 0 && person_map.contains_key(&person.father) {
            let person_index = person_map.get(&person.id).unwrap();
            let father_index = person_map.get(&person.father).unwrap();

            edges.push( (*person_index, *father_index, Parent::Father) );
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

    // pick one of the children
    let base_child = children.pop().expect("No children found in family tree");

    let mut generation_map : HashMap<isize, HashSet<NodeIndex<u32>>> = HashMap::new();
    let mut cur_generation = 0;
    let mut min_generation = 0;

    // DFS through the tree setting the generations
    let mut dfs = Dfs::new(&family_tree, base_child);

    while let Some(node) = dfs.next(&family_tree) {
        println!("Discovered: {:?} - {}", family_tree[node], cur_generation);
        family_tree[node].generation = cur_generation;  // set the generation on the node

        if generation_map.contains_key(&cur_generation) {
            generation_map.get_mut(&cur_generation).unwrap().insert(node);
        } else {
            let mut s = HashSet::new();
            s.insert(node);
            generation_map.insert(cur_generation, s);
        }

        cur_generation += 1;
    }

    family_tree.reverse(); // flip the graph

    let parents = generation_map.get(&(min_generation+1)).unwrap().to_owned();
    let parents = parents.iter().collect::<Vec<_>>();

    for parent in parents {
        let mut dfs = Dfs::new(&family_tree, *parent);
        cur_generation = min_generation + 1;

        while let Some(node) = dfs.next(&family_tree) {
            println!("Discovered: {:?} - {}", family_tree[node], cur_generation);
            family_tree[node].generation = cur_generation;  // set the generation on the node

            if generation_map.contains_key(&cur_generation) {
                generation_map.get_mut(&cur_generation).unwrap().insert(node);
            } else {
                let mut s = HashSet::new();
                s.insert(node);
                generation_map.insert(cur_generation, s);
            }

            cur_generation -= 1;
        }
    }

}