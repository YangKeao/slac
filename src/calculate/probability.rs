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
    // calc receives a flat term without none and returns a probability
    fn calc(&self) -> f64 {
        match self {
            Term::None => unreachable!(),
            Term::Atom(atom) => atom.probability(),
            Term::Union(unions) => {
                todo!()
            }
            Term::Intersect(intersects) => {
                todo!()
            }
        }
    }
}