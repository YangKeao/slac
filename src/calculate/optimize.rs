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

use super::Term;

impl Term {
    pub fn remove_none(self) -> Option<Term> {
        match self {
            Term::None => None,
            Term::Atom(atom) => Some(Term::Atom(atom.clone())),
            Term::Not(term) => {
                if let Some(subterm) = term.remove_none() {
                    Some(Term::Not(Box::new(subterm)))
                } else {
                    None
                }
            }
            Term::Union(unions) => {
                let non_empty_unions: Vec<Term> = unions
                    .into_iter()
                    .filter_map(|term| term.remove_none())
                    .collect();
                if non_empty_unions.len() == 0 {
                    None
                } else {
                    Some(Term::Union(non_empty_unions))
                }
            }
            Term::Intersect(intersects) => {
                let non_empty_intersects: Vec<Term> = intersects
                    .into_iter()
                    .filter_map(|term| term.remove_none())
                    .collect();
                if non_empty_intersects.len() == 0 {
                    None
                } else {
                    Some(Term::Intersect(non_empty_intersects))
                }
            }
        }
    }

    // flat flatten the term to make sure that:
    // 1. The operand of union is not union
    // 2. The operand of intersect is not intersect
    // 3. The number of operand of union or intersect is more than 1
    pub fn flat(self) -> Term {
        match self {
            Term::None => Term::None,
            Term::Atom(atom) => Term::Atom(atom.clone()),
            Term::Not(term) => Term::Not(Box::new(term.flat())),
            Term::Union(unions) => {
                assert!(unions.len() > 0);

                if unions.len() == 1 {
                    unions.into_iter().next().unwrap().flat()
                } else {
                    let mut flat_union: Vec<Term> = Vec::new();
                    for item in unions.into_iter() {
                        let flat_child = item.flat();
                        match flat_child {
                            Term::Union(union) => flat_union.extend(union),
                            _ => flat_union.push(flat_child),
                        }
                    }

                    Term::Union(flat_union)
                }
            }
            Term::Intersect(intersects) => {
                assert!(intersects.len() > 0);

                if intersects.len() == 1 {
                    intersects.into_iter().next().unwrap().flat()
                } else {
                    let mut flat_intersect: Vec<Term> = Vec::new();
                    for item in intersects.into_iter() {
                        let flat_child = item.flat();
                        match flat_child {
                            Term::Intersect(union) => flat_intersect.extend(union),
                            _ => flat_intersect.push(flat_child),
                        }
                    }

                    Term::Intersect(flat_intersect)
                }
            }
        }
    }

    pub fn not_push_down(self) -> Term {
        match self {
            Term::None => Term::None,
            Term::Atom(_) => self,
            Term::Not(subterm) => {
                match *subterm {
                    Term::None => Term::None,
                    Term::Atom(atom) => Term::Atom(atom),
                    Term::Not(subterm) => subterm.not_push_down(),
                    Term::Intersect(intersects) => {
                        // according to De Morgan's laws
                        let mut unions: Vec<Term> = Vec::new();
                        for item in intersects.into_iter() {
                            let not_item = Term::Not(Box::new(item));
                            let not_item = not_item.not_push_down();
                            unions.push(not_item);
                        }

                        Term::Union(unions)
                    }
                    Term::Union(unions) => {
                        // according to De Morgan's laws
                        let mut intersects: Vec<Term> = Vec::new();
                        for item in unions.into_iter() {
                            let not_item = Term::Not(Box::new(item));
                            let not_item = not_item.not_push_down();
                            intersects.push(not_item);
                        }

                        Term::Intersect(intersects)
                    }
                }
            }
            Term::Union(unions) => Term::Union(
                unions
                    .into_iter()
                    .map(|item| item.not_push_down())
                    .collect(),
            ),
            Term::Intersect(intersects) => Term::Intersect(
                intersects
                    .into_iter()
                    .map(|item| item.not_push_down())
                    .collect(),
            ),
        }
    }
}
