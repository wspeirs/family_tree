use std::fmt::{Display, Formatter, Error};

use csv::StringRecord;

#[derive(Clone, Debug)]
pub struct Person {
    pub id: usize,              // 0
    pub generation: isize,
    first_name: String,     // 1
//    middle_name: String,  // 2
    last_name: String,      // 3
    pub mother: Option<usize>,    // 4
    pub father: Option<usize>,    // 5
    dob: String,            // 6
    dod: String,            // 7
//    pob: String,
//    pod: String,
//    notes: String
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "{} {}", self.first_name, self.last_name);
        write!(f, "{} - {}", self.dob, self.dod)
    }
}

#[derive(Debug)]
pub enum Parent {
    Mother,
    Father
}

impl Person {
    pub fn new(record: StringRecord) -> Option<Person> {
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
            None
        } else {
            Some(person)
        }
    }
}