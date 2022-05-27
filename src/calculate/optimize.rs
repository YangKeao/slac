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

use super::{MultiOp, Term, UnaryOp};

impl Term {
    pub fn remove_none(self) -> Option<Term> {
        match self {
            Term::None => None,
            Term::Unary { atom, op } => Some(Term::Unary { atom, op }),
            Term::Multiple { terms, op } => {
                let non_empty_terms: Vec<Term> = terms
                    .into_iter()
                    .filter_map(|term| term.remove_none())
                    .collect();
                if non_empty_terms.is_empty() {
                    None
                } else {
                    Some(Term::Multiple {
                        terms: non_empty_terms,
                        op,
                    })
                }
            }
        }
    }

    // flat flatten the term to make sure that:
    // 1. The operand of union is not union
    // 2. The operand of intersect is not intersect
    // 3. The number of operand of union or intersect is more than 1
    pub fn flat(self) -> Term {
        // TODO: use a more suitable datastructure
        // flat can also be implemented on a mutable reference but need some
        // special data structures. If we put all Term in a compact linear
        // memory space, the flat can be implemented without overhead.

        // actually we don't need a aligned indexable data structure (as `Vec`),
        // as we are always iterating through it (but not index directly).
        match self {
            Term::None => Term::None,
            Term::Unary { atom, op } => Term::Unary { atom, op },
            Term::Multiple { terms, op } => {
                assert!(!terms.is_empty());

                if terms.len() == 1 {
                    terms.into_iter().next().unwrap().flat()
                } else {
                    let mut flated = Vec::new();
                    for item in terms.into_iter() {
                        let flated_child = item.flat();
                        match flated_child {
                            Term::Multiple {
                                terms: child_terms,
                                op: child_op,
                            } => {
                                if child_op == op {
                                    flated.extend(child_terms)
                                } else {
                                    flated.push(Term::Multiple {
                                        terms: child_terms,
                                        op: child_op,
                                    })
                                }
                            }
                            _ => flated.push(flated_child),
                        }
                    }
                    Term::Multiple { terms: flated, op }
                }
            }
        }
    }

    pub fn not(&mut self) {
        // not receives a mutable reference, rather than a ownership
        // because it could be implemented without any allocation / deallocation
        // it's much faster than the somehow immutable implementation
        match self {
            Term::None => {
                unreachable!()
            }
            Term::Unary { atom: _, op } => {
                if *op == UnaryOp::None {
                    *op = UnaryOp::Not;
                } else if *op == UnaryOp::Not {
                    *op = UnaryOp::None;
                }
            }
            Term::Multiple { terms, op } => {
                // according to De Morgan's laws
                for term in terms.iter_mut() {
                    term.not();
                }

                if *op == MultiOp::Intersect {
                    *op = MultiOp::Union
                } else if *op == MultiOp::Union {
                    *op = MultiOp::Intersect
                }
            }
        }
    }
}
