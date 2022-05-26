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

use std::sync::Arc;

use crate::calculate::{AtomRegistry, DumpTerm, Term};

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
                let mut intersects: Vec<Term> = Vec::new();

                for dep in dependencies {
                    match dep {
                        Dependency::Service(svc) => intersects.push(svc.dump_term(registry)),
                        Dependency::Group(group) => intersects.push(group.dump_term(registry)),
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
        let mut unions: Vec<Term> = Vec::new();

        let total = self.dependencies.len();
        for success_count in self.quorum..total {
            for svcs in self.dependencies.iter().combinations(success_count) {
                let mut intersects: Vec<Term> = Vec::new();
                for svc in svcs {
                    intersects.push(svc.dump_term(registry));
                }

                if intersects.len() != 0 {
                    unions.push(Term::Intersect(intersects));
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

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use rand::Rng;

    use crate::{
        calculate::{AtomRegistry, DumpTerm},
        sla::*,
    };

    #[test]
    fn test_calc() {
        fn test_calc_impl(infra_sla: f64, connection_sla: f64) -> f64 {
            macro_rules! ec2_infra {
                ($name: ident) => {
                    let $name = Service::known_sla(stringify!($name), infra_sla);
                };
            }

            macro_rules! aws_connection {
                ($name: ident) => {
                    let $name = Service::known_sla(stringify!($name), connection_sla);
                };
            }

            ec2_infra!(infra_a);
            ec2_infra!(infra_b);
            ec2_infra!(infra_c);
            ec2_infra!(infra_d);
            ec2_infra!(infra_e);

            aws_connection!(connection_a);
            aws_connection!(connection_b);
            aws_connection!(connection_c);
            aws_connection!(connection_d);

            let svc_c = Service::dependencies(vec![Dependency::Service(infra_b)]);

            let svc_d = Service::dependencies(vec![Dependency::Service(infra_c.clone())]);
            let svc_e = Service::dependencies(vec![Dependency::Service(infra_c.clone())]);
            let svc_b = Service::dependencies(vec![Dependency::Service(infra_c)]);

            let svc_g = Service::dependencies(vec![Dependency::Service(infra_e)]);

            let group_a = Group::new(vec![svc_d, svc_e, svc_c.clone(), svc_g], 2);

            let svc_a = Service::dependencies(vec![
                Dependency::Service(infra_a),
                Dependency::Service(connection_a),
                Dependency::Group(group_a),
                Dependency::Service(connection_b),
                Dependency::Service(svc_c),
            ]);

            let svc_f = Service::dependencies(vec![
                Dependency::Service(infra_d),
                Dependency::Service(connection_c),
                Dependency::Service(svc_a),
                Dependency::Service(connection_d),
                Dependency::Service(svc_b),
            ]);

            let mut atom_registry = AtomRegistry::new();
            let term = svc_f.dump_term(&mut atom_registry);

            term.calc()
        }
        fn test_calc_expected(infra_sla: f64, connection_sla: f64) -> f64 {
            1f64 - infra_sla.powi(4) * connection_sla.powi(4)
        }

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let infra_sla = rng.gen();
            let connection_sla = rng.gen();

            let expected = test_calc_expected(infra_sla, connection_sla);
            let got = test_calc_impl(infra_sla, connection_sla);
            assert!(approx_eq!(f64, expected, got, epsilon = 0.0000001f64))
        }
    }
}
