#[macro_use]
extern crate log;
extern crate clap;
#[macro_use]
extern crate serde_derive;

use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};
use std::cmp::max;
use std::fmt::{Display, Formatter, Error};

use clap::{App, Arg, SubCommand};
use csv::{StringRecord, Trim, ReaderBuilder, WriterBuilder, Writer};

mod person;
mod graph;

use person::Person;
use graph::{FamilyTree};

fn main() {
    let matches = App::new("family_tree")
        .version("1.0")
        .about("Constructs a family tree from a CSV file")
        .author("William Speirs <bill.speirs@gmail.com>")
        .subcommand(
            SubCommand::with_name("insert")
                .about("Inserts amount number of people after a specified ID")
                .arg(Arg::with_name("amount").required(true)
                    .help("The number of people to insert"))
                .arg(Arg::with_name("after").required(true)
                    .help("The ID to insert after"))
        )
        .arg(Arg::with_name("CSV")
            .help("The input CSV file to read")
            .required(true)
            .index(1))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();

    let log_level = match matches.occurrences_of("v") {
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Warn
    };

    simple_logging::log_to_stderr(log_level);

    debug!("MATCHES: {:?}", matches);

    let csv_file = matches.value_of("CSV").unwrap();

    let mut people = Person::from_file(csv_file);

    debug!("PEOPLE: {:?}", people);

    // decide what to do based upon command-line args
    let (sub_cmd, args) = matches.subcommand();

    match sub_cmd {
        "insert" => {
            let amount = args.unwrap().value_of("amount").unwrap().parse::<usize>().unwrap();
            let after = args.unwrap().value_of("after").unwrap().parse::<usize>().unwrap();

            info!("Inserting {} people after person with ID {}", amount, after);

            let mut id_map = HashMap::new();
            let first = people.first().unwrap().record.id;
            let last = people.last().unwrap().record.id;

            debug!("FIRST: {}\tLAST: {}", first, last);

            // construct the ID mapping
            for i in first..=last {
                if i <= after {
                    id_map.insert(i, i);
                } else {
                    id_map.insert(i, i+amount);
                }
            }

            for i in 0..people.len() {
                let new_id = id_map.get(&people[i].record.id).expect(format!("No mapping for {}", people[i].record.id).as_str());

                people.get_mut(i).unwrap().record.id = *new_id;

                if let Some(mother_id) = people[i].record.mother {
                    let new_mother = id_map.get(&mother_id).expect(format!("No mapping for mother {}", mother_id).as_str());
                    people.get_mut(i).unwrap().record.mother = Some(*new_mother);
                }

                if let Some(father_id) = people[i].record.father {
                    let new_father = id_map.get(&father_id).expect(format!("No mapping for father {}", father_id).as_str());
                    people.get_mut(i).unwrap().record.father = Some(*new_father);
                }
            }

            let mut csv_writer = WriterBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_writer(std::io::stdout());

            for person in people {
                person.write_csv(&mut csv_writer);
            }
        },
        _ => {
            // construct and print the family tree
            let family_tree = FamilyTree::new(people);

            family_tree.print_tree();
        }
    }
}