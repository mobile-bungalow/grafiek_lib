use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::ops;
use crate::traits::{OperationFactory, OperationFactoryTable};
use crate::{OpCategory, error::Error, node::Node, traits::QualifiedName};
use petgraph::prelude::*;

pub struct ExecutionContext {}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source_slot: usize,
    pub sink_slot: usize,
}

#[derive(Debug, Clone)]
pub struct Engine {
    graph: StableDiGraph<Node, Edge>,
    op_register: HashMap<QualifiedName, OperationFactoryTable>,
}

impl Engine {
    pub fn init() -> Result<Self, Error> {
        let mut out = Self {
            graph: StableDiGraph::new(),
            op_register: HashMap::new(),
        };
        log::info!("loading grafiek::core operators");
        out.register_op::<ops::Input>()?;
        Ok(out)
    }

    /// List all operators qualified names
    pub fn iter_operators(&self) -> impl Iterator<Item = &QualifiedName> {
        self.op_register.keys()
    }

    /// List all Operator Categories - for display
    pub fn iter_categories(&self) -> impl Iterator<Item = OpCategory> {
        OpCategory::iter()
    }

    /// List all operators registered under a given category
    pub fn operators_by_category(
        &self,
        category: OpCategory,
    ) -> impl Iterator<Item = &QualifiedName> {
        self.op_register
            .iter()
            .filter(move |(_, factory)| factory.category == category)
            .map(|(name, _)| name)
    }

    /// Registers and op such that it can be instantiated by name later
    pub fn register_op<T: OperationFactory>(&mut self) -> Result<(), Error> {
        if self.op_register.contains_key(&T::qualified_name()) {
            return Err(Error::DuplicateOperationType(T::qualified_name()));
        }
        self.op_register
            .insert(T::qualified_name(), OperationFactoryTable::new::<T>());

        Ok(())
    }
}
