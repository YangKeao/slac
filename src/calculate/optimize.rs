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

impl<'a> Term<'a> {
    pub fn remove_none<'s>(&'s self) -> Option<Term<'a>> {
        match self {
            Term::None => None,
            Term::Atom(atom) => Some(Term::Atom(atom.clone())),
            Term::Union(unions) => {
                let non_empty_unions: Vec<Box<Term<'a>>> = unions
                    .iter()
                    .filter_map(|term| term.remove_none())
                    .map(|item| -> Box<Term<'a>> { Box::new(item) })
                    .collect();
                if non_empty_unions.len() == 0 {
                    None
                } else {
                    Some(Term::Union(non_empty_unions))
                }
            }
            Term::Intersect(intersects) => {
                let non_empty_intersects: Vec<Box<Term<'a>>> = intersects
                    .iter()
                    .filter_map(|term| term.remove_none())
                    .map(|item| -> Box<Term<'a>> { Box::new(item) })
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
    pub fn flat<'s>(&'s self) -> Term<'a> {
        match self {
            Term::None => Term::None,
            Term::Atom(atom) => Term::Atom(atom.clone()),
            Term::Union(unions) => {
                assert!(unions.len() > 0);

                if unions.len() == 1 {
                    unions[0].flat()
                } else {
                    let mut flat_union: Vec<Box<Term<'a>>> = Vec::new();
                    for item in unions.iter() {
                        let flat_child = item.flat();
                        match flat_child {
                            Term::Union(union) => flat_union.extend(union),
                            _ => flat_union.push(Box::new(flat_child)),
                        }
                    }

                    Term::Union(flat_union)
                }
            }
            Term::Intersect(intersects) => {
                assert!(intersects.len() > 0);

                if intersects.len() == 1 {
                    intersects[0].flat()
                } else {
                    let mut flat_intersect: Vec<Box<Term<'a>>> = Vec::new();
                    for item in intersects.iter() {
                        let flat_child = item.flat();
                        match flat_child {
                            Term::Intersect(union) => flat_intersect.extend(union),
                            _ => flat_intersect.push(Box::new(flat_child)),
                        }
                    }

                    Term::Intersect(flat_intersect)
                }
            }
        }
    }
}
