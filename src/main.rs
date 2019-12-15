#[macro_use]
extern crate log;

use std::path::Path;
use std::collections::{HashMap, HashSet};

use csv::{Trim, ReaderBuilder};
use std::ops::{Index, IndexMut};
use std::cmp::max;
use std::fmt::{Display, Formatter, Error};

mod person;
mod graph;

use person::Person;
use graph::{FamilyTree};

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .flexible(true)
        .from_path(Path::new("family_tree.csv")).expect("Error opening family tree");

    let mut people = Vec::new();

    // read in the CSV and construct the graph
    for result in csv_reader.records() {
        if let Err(e) = result {
            panic!("Error reading CSV file: {:?}", e);
        }

        let record = result.expect("Error reading CSV file");
        let person = Person::new(record);

        if person.is_some() {
            people.push(person.unwrap());
        }
    }

    // construct the family tree
    let family_tree = FamilyTree::new(people);

    family_tree.print_tree();
}