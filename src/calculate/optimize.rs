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

use super::{ITerm, Term};

impl<'a> Term<'a> {
    pub fn remove_none(&'a self) -> Option<Term<'a>> {
        match self {
            Term::None => None,
            Term::Atom(atom) => Some(Term::Atom(atom.clone())),
            Term::Not(term) => {
                if let Some(term) = term.as_term().remove_none() {
                    Some(Term::Not(Box::new(term)))
                } else {
                    None
                }
            }
            Term::Union(unions) => {
                let non_empty_unions: Vec<Box<dyn ITerm + 'a>> = unions
                    .iter()
                    .map(|item| item.as_term())
                    .filter_map(|term| term.remove_none())
                    .map(|item| -> Box<dyn ITerm + 'a> { Box::new(item) })
                    .collect();
                if non_empty_unions.len() == 0 {
                    None
                } else {
                    Some(Term::Union(non_empty_unions))
                }
            }
            Term::Intersect(intersects) => {
                let non_empty_intersects: Vec<Box<dyn ITerm + 'a>> = intersects
                    .iter()
                    .map(|item| item.as_term())
                    .filter_map(|term| term.remove_none())
                    .map(|item| -> Box<dyn ITerm + 'a> { Box::new(item) })
                    .collect();
                if non_empty_intersects.len() == 0 {
                    None
                } else {
                    Some(Term::Intersect(non_empty_intersects))
                }
            }
        }
    }

    // binary will make sure the operand is equal or less than 2
    pub fn binary(&'a self) -> Term<'a> {
        match self {
            Term::None => Term::None,
            Term::Atom(atom) => Term::Atom(atom.clone()),
            Term::Not(term) => {
                if let Some(term) = term.as_term().remove_none() {
                    Term::Not(Box::new(term))
                } else {
                    Term::None
                }
            }
            Term::Union(unions) => {
                assert!(unions.len() > 0);
                if unions.len() == 1 {
                    unions[0].as_term().binary()
                } else {
                    let mut binary_union: Vec<Box<dyn ITerm + 'a>> = vec![
                        Box::new(unions[0].as_term().binary()),
                        Box::new(unions[1].as_term().binary()),
                    ];
                    for rhs in unions.iter().skip(2) {
                        let rhs = rhs.as_term().binary();
                        binary_union = vec![Box::new(Term::Union(binary_union)), Box::new(rhs)];
                    }
                    Term::Union(binary_union)
                }
            }
            Term::Intersect(intersects) => {
                assert!(intersects.len() > 0);
                if intersects.len() == 1 {
                    intersects[0].as_term().binary()
                } else {
                    let mut binary_intersect: Vec<Box<dyn ITerm + 'a>> = vec![
                        Box::new(intersects[0].as_term().binary()),
                        Box::new(intersects[1].as_term().binary()),
                    ];
                    for rhs in intersects.iter().skip(2) {
                        let rhs = rhs.as_term().binary();
                        binary_intersect =
                            vec![Box::new(Term::Union(binary_intersect)), Box::new(rhs)];
                    }
                    Term::Union(binary_intersect)
                }
            }
        }
    }
}
