use std::collections::HashSet;
use std::fmt::{Display, Debug};
use std::io::{self, Write};

pub use super::Alphabet;
use super::dot::{Family, Edge, GraphWriter};
use super::regex::Regex;

/// A node handle.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Node(pub usize);

pub struct Dfa<A: Alphabet> {
    edges: Vec<Vec<(A, Node)>>,
    finals: Vec<Node>,
}

impl<A: Alphabet> Dfa<A> {
    /// Build a dfa from the connecting edges and final states.
    ///
    /// States are numbered in an arbitrary order. The automaton will deduce the used alphabet
    /// subset automatically and test whether it has been used consistently.
    pub fn from_edges<I, V>(edge_iter: I, finals: V) -> Dfa<A>
    where 
        I: IntoIterator<Item=(usize, A, usize)>,
        V: IntoIterator<Item=usize>, 
        A: Clone + Debug,
    {
        let mut edges = Vec::new();
        let mut check = Vec::new();
        let mut states = HashSet::new();

        for (from, a, to) in edge_iter.into_iter() {
            edges.resize(from + 1, Vec::new());
            check.resize(from + 1, HashSet::new());
            
            edges[from].push((a.clone(), Node(to)));
            check[from].insert(a);
            states.insert(from);
            states.insert(to);
        }

        let finals = finals.into_iter()
            .inspect(|c| check.resize(c + 1, HashSet::new()))
            .map(Node)
            .collect();

        let alphabet = check.pop();
        if let Some(sample) = alphabet.as_ref() {
            if let Some(err) = check.iter().find(|&s| s != sample) {
                panic!("Different outgoing edges alphabet: {:?} vs {:?}", &sample, &err);
            }
        }

        Dfa {
            edges,
            finals,
        }
    }

    pub fn to_regex(self) -> Regex {
        unimplemented!()
    }

    pub fn write_to(&self, output: &mut Write) -> io::Result<()> 
        where for<'a> &'a A: Display
    {
        let mut writer = GraphWriter::new(output, Family::Directed, None)?;

        for (from, edges) in self.edges.iter().enumerate() {
            for (label, to) in edges.iter() {
                let edge = Edge { 
                    label: Some(format!("{}", label).into()),
                    .. Edge::none()
                };

                writer.segment([from, to.0].iter().cloned(), Some(edge))?;
            }
        }

        writer.end_into_inner().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_format() {
        let automaton = Dfa::from_edges(vec![
            (0, '0', 0),
            (0, '1', 1),
            (1, '0', 2),
            (1, '1', 0),
            (2, '0', 1),
            (2, '1', 2),
        ], vec![1]);

        let mut output = Vec::new();
        automaton.write_to(&mut output)
            .expect("failed to format to dot file");
        let output = String::from_utf8(output)
            .expect("output should be utf8 encoded");
        assert_eq!(output, r#"digraph {
	0 -> 0 [label=0];
	0 -> 1 [label=1];
	1 -> 2 [label=0];
	1 -> 0 [label=1];
	2 -> 1 [label=0];
	2 -> 2 [label=1];
}
"#);
    }
}
