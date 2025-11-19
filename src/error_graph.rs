use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use crate::{
    ErrorRepr,
    records::{Email, ErrorReason},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub error: ErrorReason,
    pub users: HashSet<Email>,
}

impl Node {
    pub fn new(error: ErrorReason) -> Self {
        Self {
            error,
            users: Default::default(),
        }
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.error.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.error.eq(&other.error)
    }
}

impl Eq for Node {}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    nodes: Vec<Node>,
    adjaceny_list: Vec<Vec<usize>>,
}

impl Graph {
    pub fn insert_many(&mut self, email: impl ToString, errors: Vec<Vec<ErrorRepr>>) {
        for error in errors {
            self.insert(email.to_string(), error);
        }
    }

    pub fn insert(&mut self, email: impl ToString, errors: Vec<ErrorRepr>) {
        let mut prev_idx: Option<usize> = None;
        for e in errors {
            let node_idx =
                if let Some(node_idx) = self.nodes.iter().position(|x| x.error == e.reason) {
                    node_idx
                } else {
                    self.nodes.push(Node::new(e.reason));
                    self.nodes.len() - 1
                };

            self.nodes[node_idx].users.insert(email.to_string());

            while self.adjaceny_list.len() <= node_idx {
                self.adjaceny_list.push(vec![]);
            }

            if let Some(prev_idx) = prev_idx {
                self.adjaceny_list[prev_idx].push(node_idx);
            }

            prev_idx = Some(node_idx);
        }
    }
}

impl Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let idx_map = self.nodes.iter().enumerate().collect::<HashMap<_, _>>();

        let mut sorted_nodes = self.nodes.iter().enumerate().collect::<Vec<_>>();
        sorted_nodes.sort_by(|(_, a), (_, b)| b.users.len().cmp(&a.users.len()));

        write!(
            f,
            "<><><><><> All Nodes <><><><><>\n{}\n\n<><><><><> Adjacency List <><><><><>\n",
            serde_json::to_string_pretty(&sorted_nodes.iter().map(|(_, x)| x).collect::<Vec<_>>())
                .unwrap()
        )?;

        for (node_idx, node) in sorted_nodes {
            write!(f, "({} Users) {}\n", node.users.len(), node.error)?;

            if let Some(list) = self.adjaceny_list.get(node_idx) {
                let unique_list = list
                    .iter()
                    .filter_map(|i| idx_map.get(i))
                    .map(|x| &x.error)
                    .collect::<HashSet<_>>();
                writeln!(f, "linked to: \n{:#?}\n", unique_list)?;
            }
        }

        Ok(())
    }
}
