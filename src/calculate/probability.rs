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

use std::{collections::HashMap, sync::Arc};

use crate::calculate::Atom;

use super::Term;

use itertools::Itertools;

impl Term {
    // if every term in an intersection is an atom or a not atom
    // this function will calculate the probability of it directly
    // with the assumption that all of them are independent
    // or this function will return a None
    fn calc_minimum_unit(&self) -> Option<f64> {
        match self {
            Term::Intersect(intersects) => {
                // first, we look for conflict requirements
                let mut sign: HashMap<&str, (bool, Arc<Atom>)> = HashMap::new();
                for item in intersects {
                    match item.as_ref() {
                        Term::Not(not) => {
                            if let Term::Atom(atom) = not.as_ref() {
                                if let Some((sign, _)) = sign.get(&atom.name()) {
                                    if *sign {
                                        // conflict, return 0
                                        return Some(0.0);
                                    }
                                } else {
                                    sign.insert(atom.name(), (false, atom.clone()));
                                }
                            } else {
                                return None;
                            }
                        }
                        Term::Atom(atom) => {
                            if let Some((sign, _)) = sign.get(atom.name()) {
                                if !*sign {
                                    // conflict, return 0
                                    return Some(0.0);
                                }
                            } else {
                                sign.insert(atom.name(), (true, atom.clone()));
                            }
                        }
                        _ => return None,
                    }
                }

                // then, we calculate the probability
                let mut product = 1f64;
                for (_, (sign, atom)) in sign.iter() {
                    if *sign {
                        product *= atom.probability();
                    } else {
                        product *= 1.0 - atom.probability();
                    }
                }
                Some(product)
            }
            _ => unreachable!(),
        }
    }

    pub fn calc(&self) -> f64 {
        self.not_push_down().flat().inner_calc()
    }

    // inner_calc receives a flat term without none and returns a probability
    fn inner_calc(self) -> f64 {
        match self {
            Term::None => unreachable!(),
            Term::Atom(atom) => atom.probability(),
            Term::Not(subterm) => 1.0 - subterm.inner_calc(),
            Term::Union(unions) => {
                // TODO: optimize the performance
                let mut sum = 0f64;
                for ele in unions.into_iter().powerset() {
                    if ele.len() == 0 {
                        continue;
                    }

                    let sign = if ele.len() % 2 == 1 { 1f64 } else { -1f64 };

                    let intersects: Vec<Box<Term>> = ele;
                    let intersect = Term::Intersect(intersects).flat();
                    sum += sign * intersect.inner_calc();
                }
                sum
            }
            Term::Intersect(_) => {
                // TODO: optimize the performance
                let mut sum = 1f64;

                if let Some(result) = self.calc_minimum_unit() {
                    return result;
                }
                println!("failed to calculate direct {:?}", self);

                if let Term::Intersect(intersects) = self {
                    // According to De Morgan's laws
                    for ele in intersects.iter().powerset() {
                        if ele.len() == 0 {
                            continue;
                        }

                        let sign = if ele.len() % 2 == 1 { -1f64 } else { 1f64 };

                        let intersects: Vec<Box<Term>> = ele
                            .iter()
                            .map(|item| (*item).clone())
                            .map(|item| Box::new(Term::Not(item)))
                            .collect();

                        let intersect = Term::Intersect(intersects).not_push_down().flat();
                        sum += sign * intersect.inner_calc();
                    }

                    sum
                } else {
                    unreachable!()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use float_cmp::approx_eq;
    // use rand::Rng;

    // use crate::{calculate::DumpTerm, sla::*};

    // #[test]
    // fn test_calc() {
    //     fn test_calc_impl(infra_sla: f64, connection_sla: f64) -> f64 {
    //         let ec2_infra = || -> Infra { Infra::new(infra_sla) };
    //         let aws_connection = || -> Connection { Connection::new(connection_sla) };

    //         let infra_a = ec2_infra();
    //         let program_a = Program::new(&infra_a);

    //         let infra_b = ec2_infra();
    //         let program_c = Program::new(&infra_b);

    //         let infra_c = ec2_infra();
    //         let program_d = Program::new(&infra_c);
    //         let program_e = Program::new(&infra_c);
    //         let program_b = Program::new(&infra_c);

    //         let infra_e = ec2_infra();
    //         let program_g = Program::new(&infra_e);

    //         let group_a = Group::new(2);
    //         unsafe {
    //             group_a.add(&program_d);
    //             group_a.add(&program_e);
    //             group_a.add(&program_c);
    //             group_a.add(&program_g);
    //         }

    //         let svc_a = Service::Internal(InternalService::new(GroupOrProgram::Group(&group_a)));
    //         let svc_b =
    //             Service::Internal(InternalService::new(GroupOrProgram::Program(&program_c)));

    //         let connection_a = aws_connection();
    //         let connection_b = aws_connection();
    //         unsafe {
    //             program_a.depend(&connection_a, &svc_a);
    //             program_a.depend(&connection_b, &svc_b);
    //         }

    //         let infra_d = ec2_infra();
    //         let program_f = Program::new(&infra_d);
    //         let svc_c =
    //             Service::Internal(InternalService::new(GroupOrProgram::Program(&program_a)));
    //         let svc_d =
    //             Service::Internal(InternalService::new(GroupOrProgram::Program(&program_b)));
    //         let connection_c = aws_connection();
    //         let connection_d = aws_connection();
    //         unsafe {
    //             program_f.depend(&connection_c, &svc_c);
    //             program_f.depend(&connection_d, &svc_d);
    //         }

    //         let end_svc =
    //             Service::Internal(InternalService::new(GroupOrProgram::Program(&program_f)));

    //         let term = end_svc.dump_term();

    //         term.calc()
    //     }
    //     fn test_calc_expected(infra_sla: f64, connection_sla: f64) -> f64 {
    //         1f64 - infra_sla.powi(4) * connection_sla.powi(4)
    //     }

    //     let mut rng = rand::thread_rng();
    //     for _ in 0..100 {
    //         let infra_sla = rng.gen();
    //         let connection_sla = rng.gen();

    //         let expected = test_calc_expected(infra_sla, connection_sla);
    //         let got = test_calc_impl(infra_sla, connection_sla);
    //         assert!(approx_eq!(f64, expected, got, epsilon = 0.0000001f64))
    //     }
    // }
}
