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

use std::{fmt::Debug, sync::Arc};

pub trait Atom: Debug {
    fn probability(&self) -> f64;
    fn name(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Term<'a> {
    // none is a special case that represents the empty set
    // anything calculate with none results in itself
    None,

    // one atom could be shared by multiple transforming terms
    Atom(Arc<dyn Atom + 'a>),

    Not(Box<Term<'a>>),

    Union(Vec<Box<Term<'a>>>),
    Intersect(Vec<Box<Term<'a>>>),
}

impl<'a> Term<'a> {
    pub fn is_none(&self) -> bool {
        match self {
            Term::None => true,
            _ => false,
        }
    }
}

pub trait DumpTerm {
    fn dump_term(&self) -> Term;
}
