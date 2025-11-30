use std::collections::HashMap;

use crate::ops;
use crate::traits::{OperationFactory, OperationFactoryEntry};
use crate::{error::Error, node::Node};
use petgraph::prelude::*;

pub struct ExecutionContext {}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source_slot: usize,
    pub sink_slot: usize,
}

type OpRegistry = HashMap<&'static str, HashMap<&'static str, OperationFactoryEntry>>;

#[derive(Debug, Clone, Default)]
pub struct Engine {
    graph: StableDiGraph<Node, Edge>,
    registry: OpRegistry,
}

impl Engine {
    pub fn init() -> Result<Self, Error> {
        let mut out = Self::default();
        log::info!("loading grafiek::core operators");
        out.register_op::<ops::Input>()?;
        Ok(out)
    }

    pub fn node_categories(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.registry.keys().copied()
    }

    pub fn iter_category(&self, category: &str) -> impl Iterator<Item = &'static str> + '_ {
        self.registry
            .get(category)
            .into_iter()
            .flat_map(|m| m.keys().copied())
    }

    pub fn register_op<T: OperationFactory>(&mut self) -> Result<(), Error> {
        let lib = self.registry.entry(T::PATH.library).or_default();
        if lib.contains_key(T::PATH.operator) {
            return Err(Error::DuplicateOperationType(
                T::PATH.library,
                T::PATH.operator,
            ));
        }
        lib.insert(T::PATH.operator, OperationFactoryEntry::new::<T>());
        Ok(())
    }

    pub fn create_node(&mut self, library: &str, operator: &str) -> Result<NodeIndex, Error> {
        let entry = self
            .registry
            .get(library)
            .and_then(|lib| lib.get(operator))
            .ok_or_else(|| Error::UnknownOperationType(format!("{}/{}", library, operator)))?;

        todo!();
        //let operation = (entry.build)()?;
        //Node::new(operation);
        //let out = self.graph.add_node(node);

        //self.node_created();

        //Ok(out)
    }
}
