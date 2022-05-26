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
                let mut sign: BTreeMap<&str, (bool, Arc<Atom>)> = BTreeMap::new();
                for item in intersects {
                    match item {
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
        self.clone().not_push_down().flat().inner_calc()
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

                    let intersects: Vec<Term> = ele;
                    let intersect = Term::Intersect(intersects).flat();
                    // complicated situation could cause stack overflow
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
                    for ele in intersects.into_iter().powerset() {
                        if ele.len() == 0 {
                            continue;
                        }

                        let sign = if ele.len() % 2 == 1 { -1f64 } else { 1f64 };

                        let intersects: Vec<Term> = ele
                            .into_iter()
                            .map(|item| Term::Not(Box::new(item)))
                            .collect();

                        let intersects = Term::Intersect(intersects).not_push_down().flat();
                        sum += sign * intersects.inner_calc();
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
    use crate::calculate::{AtomRegistry, Term};

    #[test]
    fn test_calc() {
        let mut registry = AtomRegistry::new();

        let prob_a = 0.5;
        let prob_b = 0.9;

        let atom_a = registry.new_atom("atom_a".to_owned(), prob_a);
        let atom_b = registry.new_atom("atom_b".to_owned(), prob_b);

        let union = Term::Union(vec![Term::Atom(atom_a.clone()), Term::Atom(atom_b.clone())]);
        assert_eq!(union.calc(), prob_a + prob_b - prob_a * prob_b);

        let intersect =
            Term::Intersect(vec![Term::Atom(atom_a.clone()), Term::Atom(atom_b.clone())]);
        assert_eq!(intersect.calc(), prob_a * prob_b);

        let intersect_of_union = Term::Intersect(vec![union.clone(), union.clone()]);
        assert_eq!(intersect_of_union.calc(), prob_a + prob_b - prob_a * prob_b);
    }
}
