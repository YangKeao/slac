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

use std::{cell::RefCell, sync::Arc};

use crate::calculate::{Atom, AtomRegistry, DumpTerm, Term};

use itertools::Itertools;

pub enum Service {
    KnownSLA { name: String, sla: f64 },
    Dependencies(Vec<Dependency>),
}

pub struct Group {
    dependencies: Vec<Arc<Service>>,
    quorum: usize,
}

// Actually, a service and a group doesn't have too much difference just left
// here for the convinience.
pub enum Dependency {
    // The user actually can cause loop reference, in which case Arc will fail
    // to work. However, this program also cannot work well under the loop
    // dependency, so the user should handle the loop reference situation by
    // themselves.
    Service(Arc<Service>),
    Group(Arc<Group>),
}

pub struct AtomAllocator {}

impl Service {
    pub fn known_sla<S: AsRef<str>>(name: S, sla: f64) -> Arc<Service> {
        Arc::new(Service::KnownSLA {
            name: name.as_ref().to_string(),
            sla,
        })
    }

    pub fn dependencies(dependencies: Vec<Dependency>) -> Arc<Service> {
        Arc::new(Service::Dependencies(dependencies))
    }
}

impl Group {
    pub fn new(dependencies: Vec<Arc<Service>>, quorum: usize) -> Arc<Group> {
        Arc::new(Group {
            dependencies,
            quorum,
        })
    }
}

impl DumpTerm for Service {
    fn dump_term(&self, registry: &mut AtomRegistry) -> Term {
        match &self {
            Service::KnownSLA { name, sla } => Term::Atom(registry.new_atom(name.clone(), *sla)),
            Service::Dependencies(dependencies) => {
                let mut intersects: Vec<Box<Term>> = Vec::new();

                for dep in dependencies {
                    match dep {
                        Dependency::Service(svc) => {
                            intersects.push(Box::new(svc.dump_term(registry)))
                        }
                        Dependency::Group(group) => {
                            intersects.push(Box::new(group.dump_term(registry)))
                        }
                    }
                }

                if intersects.len() == 0 {
                    Term::None
                } else {
                    Term::Intersect(intersects)
                }
            }
        }
    }
}

impl DumpTerm for Group {
    fn dump_term(&self, registry: &mut AtomRegistry) -> Term {
        let mut unions: Vec<Box<Term>> = Vec::new();

        let total = self.dependencies.len();
        for success_count in self.quorum..total {
            for svcs in self.dependencies.iter().combinations(success_count) {
                let mut intersects: Vec<Box<Term>> = Vec::new();
                for svc in svcs {
                    intersects.push(Box::new(svc.dump_term(registry)));
                }

                if intersects.len() != 0 {
                    unions.push(Box::new(Term::Intersect(intersects)));
                }
            }
        }

        if unions.len() == 0 {
            Term::None
        } else {
            Term::Union(unions)
        }
    }
}
