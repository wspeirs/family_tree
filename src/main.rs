#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::collections::HashMap;

use csv::{Trim, ReaderBuilder};
use petgraph::Graph;
use petgraph::visit::Topo;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Person {
    id: usize,              // 0
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

        let node_id = family_tree.add_node(person);  // add the node to the graph

        person_map.insert(person_id, node_id);
    }

    let mut edges = Vec::new();

    // go through the people, and add the mother/father edges
    for node in family_tree.raw_nodes() {
//        debug!("{:?}", node);

        let person = &node.weight;

        if person.mother != 0 && person_map.contains_key(&person.mother) {
            let person_index = person_map.get(&person.id).unwrap();
            let mother_index = person_map.get(&person.mother).unwrap();

            edges.push( (*person_index, *mother_index, Parent::Mother) );
        }
    }

    for (person, parent, p_type) in edges {
        family_tree.add_edge(person, parent, p_type);
    }

    let topo = Topo::new(family_tree);

    while let Some(person) = topo.next() {
        debug!("{:#?}", person);
    }

}