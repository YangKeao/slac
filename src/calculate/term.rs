// Copyright 2022 Chaos Mesh Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct Atom {
    probability: f64,
    name: String,
}

impl Atom {
    /// Get a reference to the atom's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the atom's probability.
    pub fn probability(&self) -> f64 {
        self.probability
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    // none is a special case that represents the empty set
    // anything calculate with none results in itself
    None,

    // one atom could be shared by multiple transforming terms
    Atom(Arc<Atom>),

    Not(Box<Term>),

    Union(Vec<Term>),
    Intersect(Vec<Term>),
}

impl Term {
    pub fn is_none(&self) -> bool {
        match self {
            Term::None => true,
            _ => false,
        }
    }
}

pub struct AtomRegistry {
    registry: HashMap<String, Arc<Atom>>,
}

impl AtomRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn new_atom(&mut self, name: String, probability: f64) -> Arc<Atom> {
        self.registry
            .entry(name.clone())
            .or_insert(Arc::new(Atom { name, probability }))
            .clone()
    }
}

pub trait DumpTerm {
    fn dump_term(&self, registry: &mut AtomRegistry) -> Term;
}
