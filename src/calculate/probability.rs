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

use std::{collections::BTreeMap, sync::Arc};

use crate::calculate::{Atom, MultiOp, UnaryOp};

use super::Term;

use itertools::Itertools;

trait CalcMinimumUnit {
    fn calc_minimum_unit(&self) -> Option<f64>;
}

impl CalcMinimumUnit for Vec<Term> {
    // if every term in an intersection is an atom or a not atom
    // this function will calculate the probability of it directly
    // with the assumption that all of them are independent
    // or this function will return a None
    fn calc_minimum_unit(&self) -> Option<f64> {
        // first, we look for conflict requirements
        let mut sign: BTreeMap<&str, (bool, Arc<Atom>)> = BTreeMap::new();
        for item in self {
            match item {
                Term::Unary {
                    atom,
                    op: UnaryOp::Not,
                } => {
                    if let Some((sign, _)) = sign.get(&atom.name()) {
                        if *sign {
                            // conflict, return 0
                            return Some(0.0);
                        }
                    } else {
                        sign.insert(atom.name(), (false, atom.clone()));
                    }
                }
                Term::Unary {
                    atom,
                    op: UnaryOp::None,
                } => {
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
}

impl Term {
    pub fn calc(&self) -> f64 {
        self.clone().flat().inner_calc()
    }

    // inner_calc receives a flat term without none and returns a probability
    fn inner_calc(self) -> f64 {
        match self {
            Term::None => unreachable!(),
            Term::Unary {
                atom,
                op: UnaryOp::None,
            } => atom.probability(),
            Term::Unary {
                atom,
                op: UnaryOp::Not,
            } => 1.0 - atom.probability(),
            Term::Multiple {
                terms,
                op: MultiOp::Union,
            } => {
                // TODO: optimize the performance
                let mut sum = 0f64;
                for ele in terms.into_iter().powerset() {
                    if ele.is_empty() {
                        continue;
                    }

                    let sign = if ele.len() % 2 == 1 { 1f64 } else { -1f64 };

                    let intersects: Vec<Term> = ele;
                    let intersect = Term::intersect(intersects).flat();
                    // complicated situation could cause stack overflow
                    sum += sign * intersect.inner_calc();
                }
                sum
            }
            Term::Multiple {
                terms,
                op: MultiOp::Intersect,
            } => {
                // TODO: optimize the performance
                let mut sum = 1f64;

                if let Some(result) = terms.calc_minimum_unit() {
                    return result;
                }

                // According to De Morgan's laws
                for ele in terms.into_iter().powerset() {
                    if ele.is_empty() {
                        continue;
                    }

                    let sign = if ele.len() % 2 == 1 { -1f64 } else { 1f64 };

                    let intersects: Term = Term::intersect(
                        ele.into_iter()
                            .map(|mut item| {
                                item.not();
                                item
                            })
                            .collect(),
                    );

                    let intersects = intersects.flat();
                    sum += sign * intersects.inner_calc();
                }

                sum
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::calculate::{AtomRegistry, Term};

    #[test]
    fn test_calc() {
        let mut registry = AtomRegistry::default();

        let prob_a = 0.5;
        let prob_b = 0.9;

        let atom_a = registry.new_atom("atom_a".to_owned(), prob_a);
        let atom_b = registry.new_atom("atom_b".to_owned(), prob_b);

        let union = Term::union(vec![Term::atom(atom_a.clone()), Term::atom(atom_b.clone())]);
        assert_eq!(union.calc(), prob_a + prob_b - prob_a * prob_b);

        let intersect = Term::intersect(vec![Term::atom(atom_a), Term::atom(atom_b)]);
        assert_eq!(intersect.calc(), prob_a * prob_b);

        let intersect_of_union = Term::intersect(vec![union.clone(), union]);
        assert_eq!(intersect_of_union.calc(), prob_a + prob_b - prob_a * prob_b);
    }
}
