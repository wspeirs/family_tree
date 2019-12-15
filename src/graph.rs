use std::collections::{HashMap, HashSet};
use std::ops::IndexMut;

use petgraph::{Graph};
use petgraph::graph::{NodeIndex};
use petgraph::visit::{Dfs, IntoNodeReferences};
use petgraph::dot::{Dot, Config};

use crate::person::{Person, Parent};

pub struct FamilyTree {
    person_map: HashMap<usize, NodeIndex>,
    family_tree: Graph<Person, Parent>
}

impl FamilyTree {
    pub fn new(people: Vec<Person>) -> FamilyTree {
        let mut person_map = HashMap::new();
        let mut family_tree = Graph::<Person, Parent>::new();

        // add all the people as nodes, and construct the lookup mapping: id -> NodeId
        for person in people {
            let person_id = person.id;
            let node_id = family_tree.add_node(person);

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

        // construct our struct
        let mut ret = FamilyTree {
            person_map,
            family_tree
        };

        // get the "youngest" generation child in the graph
        let youngest_child = ret.get_youngest_child();

        // assign a generation to everyone
        ret.assign_generations(youngest_child);

        ret
    }

    fn id2index(&self, id: usize) -> NodeIndex {
        *self.person_map.get(&id).unwrap()
    }

    fn index2person(&self, index: NodeIndex) -> &Person {
        &self.family_tree[index]
    }

    fn id2person(&self, id: usize) -> &Person {
        self.index2person(self.id2index(id))
    }

    fn father_index(&self, index: NodeIndex) -> Option<NodeIndex> {
        self.index2person(index).father.and_then(|id| Some(self.id2index(id)))
    }

    fn mother_index(&self, index: NodeIndex) -> Option<NodeIndex> {
        self.index2person(index).mother.and_then(|id| Some(self.id2index(id)))
    }

    ///
    /// Finds the "youngest" (by generation) child in the graph
    ///
    fn get_youngest_child(&mut self) -> NodeIndex {
        // find all the children
        let mut children = self.family_tree.node_indices().filter(|n| {
            if self.family_tree.neighbors_directed(*n, petgraph::Incoming).fold(0, |a, _e| a + 1) == 0 {
                true
            } else {
                false
            }
        }).collect::<Vec<_>>();

        // DEBUG: print all the children
        for child in &children {
            debug!("CHILD: {:?}", self.index2person(*child));
        }

        // get a unique set of all parents for all the children
        let mut parents = HashSet::new();

        for child in &children {
            parents.insert(self.father_index(*child).unwrap());
            parents.insert(self.mother_index(*child).unwrap());
        }

        // DEBUG: print all parents
        for parent in &parents {
            debug!("PARENT: {:?}", self.index2person(*parent))
        }

        // pick a random child as our base
        let mut base_child = children.pop().expect("No children found in family tree!");
        let mut base_child_length = -1;

        // DFS from child -> parent, keeping track of the longest one
        for child in &children {
            let mut dfs = Dfs::new(&self.family_tree, *child);
            let mut cur_length = 0;

            while let Some(node) = dfs.next(&self.family_tree) {
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
        self.family_tree[base_child].generation = 0;

        debug!("YOUNGEST CHILD: {:?}", self.family_tree[base_child]);

        base_child
    }

    ///
    /// Assigns the proper generation to each node
    ///
    fn assign_generations(&mut self, youngest_child: NodeIndex) {
        let mut dfs = Dfs::new(&self.family_tree, youngest_child);

        // DFS from the base child to everyone, setting their generations
        while let Some(node) = dfs.next(&self.family_tree) {
            let cur_generation = self.index2person(node).generation + 1;

            if let Some(father) = self.father_index(node) {
                self.family_tree.index_mut(father).generation = cur_generation;
                debug!("FATHER: {:?}", self.index2person(father));
            }

            if let Some(mother) = self.mother_index(node) {
                self.family_tree.index_mut(mother).generation = cur_generation;
                debug!("MOTHER: {:?}", self.index2person(mother));
            }
        }

        // get all of the unassigned
        let mut unassigned = self.family_tree.node_indices().filter(|n| {
            self.family_tree[*n].generation == -1
        }).collect::<Vec<_>>();

        // assign them from their parents
        while !unassigned.is_empty() {
            debug!("UNASSIGNED: {:?}", unassigned);

            for person in &unassigned {
                if let Some(m) = self.index2person(*person).mother {
                    let mother_generation = self.id2person(m).generation;

                    if mother_generation != -1 {
                        self.family_tree[*person].generation = mother_generation - 1;
                    }
                } else if let Some(f) = self.index2person(*person).father {
                    let father_generation = self.id2person(f).generation;

                    if father_generation != -1 {
                        self.family_tree[*person].generation = father_generation - 1;
                    }
                }
            }

            unassigned = self.family_tree.node_indices().filter(|n| {
                self.family_tree[*n].generation == -1
            }).collect::<Vec<_>>();
        }
    }

    ///
    /// Prints out the graph in Dot format
    ///
    pub fn print_tree(&self) {
        // find the max generation
        let max_generation = self.family_tree.node_indices().fold(-1, |max_gen, index| {
            std::cmp::max(max_gen, self.index2person(index).generation)
        });

        // print out our family tree
        println!("digraph {{");

        // go through generation-by-generation, starting with the max
        for cur_generation in (0..max_generation).rev() {
            let members = self.family_tree.node_indices().filter(|n| {
                self.index2person(*n).generation == cur_generation
            }).collect::<Vec<_>>();

            println!("{{ rank=same; ");

            for member in &members {
                let p = self.index2person(*member);

                println!("{} [shape=rect label=\"{}\"]", p.id, p);
            }

            println!("}};");

            for member in &members {
                let p = self.index2person(*member);

                if let Some(m) = p.mother {
                    println!("{} -> {}", p.id, m);
                }

                if let Some(f) = p.father {
                    println!("{} -> {}", p.id, f);
                }
            }
        }

        println!("}}");
    }
}