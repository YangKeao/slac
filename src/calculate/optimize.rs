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
    pub fn remove_none(&'a self) -> Option<Term<'a>> {
        match self {
            Term::None => None,
            Term::Atom(atom) => {
                Some(Term::Atom(atom.clone()))
            },
            Term::Union { lhs, rhs } => {
                let lhs = lhs.as_term().remove_none();
                let rhs = rhs.as_term().remove_none();
                if lhs.is_none() && rhs.is_some() {
                    rhs
                } else if rhs.is_none() && lhs.is_some() {
                    lhs
                } else {
                    Some(Term::Union { 
                        lhs: Box::new(lhs.unwrap()), 
                        rhs: Box::new(rhs.unwrap()),
                    })
                }
            },
            Term::Intersect { lhs, rhs } => {
                let lhs = lhs.as_term().remove_none();
                let rhs = rhs.as_term().remove_none();
                if lhs.is_none() && rhs.is_some() {
                    rhs
                } else if rhs.is_none() && lhs.is_some() {
                    lhs
                } else {
                    Some(Term::Intersect { 
                        lhs: Box::new(lhs.unwrap()), 
                        rhs: Box::new(rhs.unwrap()),
                    })
                }
            },
        }
    }
}
