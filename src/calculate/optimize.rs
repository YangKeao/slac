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

use super::{Term, ITerm};

impl<'a> Term<'a> {
    pub fn remove_none(&'a self) -> Option<Term<'a>> {
        match self {
            Term::None => None,
            Term::Atom(atom) => {
                Some(Term::Atom(atom.clone()))
            },
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
                    .map(|item| -> Box<dyn ITerm + 'a> {Box::new(item)})
                    .collect();
                if non_empty_unions.len() == 0 {
                    None
                } else {
                    Some(Term::Union(non_empty_unions))
                }
            },
            Term::Intersect(intersects) => {
                let non_empty_intersects: Vec<Box<dyn ITerm + 'a>> = intersects
                    .iter()
                    .map(|item| item.as_term())
                    .filter_map(|term| term.remove_none())
                    .map(|item| -> Box<dyn ITerm + 'a> {Box::new(item)})
                    .collect();
                if non_empty_intersects.len() == 0 {
                    None
                } else {
                    Some(Term::Intersect(non_empty_intersects))
                }
            },
        }
    }
}
