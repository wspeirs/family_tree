use std::fmt::{Display, Formatter, Error};
use std::path::Path;

use csv::{StringRecord, Trim, ReaderBuilder, WriterBuilder, Writer};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersonRecord {
    pub id: usize,              // 0
    first_name: String,         // 1
    middle_name: String,        // 2
    last_name: String,          // 3
    pub mother: Option<usize>,  // 4
    pub father: Option<usize>,  // 5
    dob: String,                // 6
    dod: String,                // 7
    pob: String,
    pod: String,
    notes: String
}

#[derive(Clone, Debug)]
pub struct Person {
    pub record: PersonRecord,
    pub generation: isize,
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "{} {}", self.record.first_name, self.record.last_name)?;
        write!(f, "{} - {}", self.record.dob, self.record.dod)
    }
}

#[derive(Debug)]
pub enum Parent {
    Mother,
    Father
}

impl Person {
    pub fn from_file(file: &str) -> Vec<Person> {
        // construct the CSV reader
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .flexible(true)
            .from_path(Path::new(file)).expect(format!("Error opening family tree file: {}", file).as_str());

        let rec_iter = csv_reader.deserialize();
        let mut people = Vec::new();

        for record in rec_iter {
            let person_record :PersonRecord = record.unwrap();

            // filter anyone who doesn't have a last name
            if person_record.first_name.is_empty() && person_record.last_name.is_empty() {
                continue;
            }

            // add to our list
            people.push(Person { record: person_record, generation: -1 });
        }

        // sort the list by ID
        people.sort_by_key(|p| p.record.id);

        people
    }

    pub fn write_csv<W>(&self, writer: &mut Writer<W>) where W: std::io::Write {
        // write the record
        writer.serialize(&self.record).expect("Error writing person");
    }
}