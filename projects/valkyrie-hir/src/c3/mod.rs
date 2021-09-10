use crate::{LinearizeError, ValkyrieRowType, row_type::IntoRowType};
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
};
use valkyrie_types::NamePath;

pub struct ValkyrieTypeGraph {
    graph: HashMap<NamePath, ValkyrieRowType>,
}

pub struct LinearizedGraph<'a> {
    source: &'a ValkyrieTypeGraph,
    result: HashMap<NamePath, Vec<NamePath>>,
}

impl<'a> Debug for LinearizedGraph<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.result.iter()).finish()
    }
}

impl ValkyrieTypeGraph {
    pub fn new(capacity: usize) -> Self {
        ValkyrieTypeGraph { graph: HashMap::with_capacity(capacity) }
    }
    pub fn register_class(&mut self, class: impl IntoRowType) -> ValkyrieRowType {
        let clazz = class.into_row();
        self.graph.insert(clazz.get_name(), clazz.clone());
        clazz
    }

    pub fn linearize(&self) -> Result<LinearizedGraph, LinearizeError> {
        let mut results = HashMap::new();
        let mut visiting = HashSet::new();

        for head in self.graph.keys() {
            self.resolve(head, &mut results, &mut visiting)?;
        }
        for (_, v) in results.iter_mut() {
            v.reverse();
        }
        Ok(LinearizedGraph { source: self, result: results })
    }

    fn resolve(
        &self,
        head: &NamePath,
        results: &mut HashMap<NamePath, Vec<NamePath>>,
        visiting: &mut HashSet<NamePath>,
    ) -> Result<Vec<NamePath>, LinearizeError> {
        if let Some(res) = results.get(head) {
            return Ok(res.clone());
        }

        if visiting.contains(head) {
            return Err(LinearizeError::Circular { node: head.to_string() });
        }
        visiting.insert(*head);

        let mut sequences: Vec<Vec<NamePath>> = Vec::new();
        if let Some(nyar) = self.graph.get(head) {
            for parent in nyar.inherit_order() {
                let res = self.resolve(&parent, results, visiting)?;
                sequences.push(res);
            }
        }

        let mut res = vec![*head];
        res.extend(merge(&mut sequences)?);
        results.insert(*head, res.clone());

        visiting.remove(head);
        Ok(res)
    }
}

fn merge(sequences: &mut Vec<Vec<NamePath>>) -> Result<Vec<NamePath>, LinearizeError> {
    let mut result = Vec::new();

    while !sequences.is_empty() {
        let mut found = false;

        for seq in sequences.clone().iter() {
            let head = &seq[0];

            // Check for "bad heads"
            if !sequences.iter().any(|s| s != seq && s[1..].contains(head)) {
                found = true;
                result.push(head.clone());

                for s in sequences.iter_mut() {
                    if let Some(pos) = s.iter().position(|x| x == head) {
                        s.remove(pos);
                    }
                }

                break;
            }
        }

        sequences.retain(|s| !s.is_empty());

        if !found {
            return Err(LinearizeError::NotFound);
        }
    }

    Ok(result)
}
