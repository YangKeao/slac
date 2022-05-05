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

    // dnf converts the term to dnf
    pub fn dnf<'s>(&'s self) -> Term<'a> {
        match self {
            Term::None => Term::None,
            Term::Atom(atom) => Term::Atom(atom.clone()),
            Term::Union(unions) => {
                assert!(unions.len() > 0);

                Term::Union(
                    unions
                        .iter()
                        .map(|item| item.dnf())
                        .map(|item| -> Box<Term<'a>> { Box::new(item) })
                        .collect(),
                )
            }
            Term::Intersect(intersects) => {
                assert!(intersects.len() > 0);

                if intersects.len() == 1 {
                    return intersects[0].dnf();
                }

                let lhs = intersects[0].dnf();
                let lhs = lhs.flat();

                let rhs = Term::Intersect(
                    intersects[1..]
                        .iter()
                        .map(|item| item.dnf())
                        .map(|item| -> Box<Term<'a>> { Box::new(item) })
                        .collect(),
                );
                let rhs = rhs.dnf();
                match lhs {
                    Term::None => lhs,
                    Term::Atom(_) | Term::Intersect(_) => match rhs {
                        Term::None => lhs,
                        Term::Atom(_) | Term::Intersect(_) => {
                            Term::Intersect(vec![Box::new(lhs), Box::new(rhs)])
                        }
                        Term::Union(unions) => {
                            let mut distributive_unions: Vec<Box<Term<'a>>> = Vec::new();
                            for item in unions.into_iter() {
                                distributive_unions.push(Box::new(Term::Intersect(vec![
                                    Box::new(lhs.clone()),
                                    item,
                                ])));
                            }
                            Term::Union(distributive_unions)
                        }
                    },
                    Term::Union(unions) => {
                        let mut distributive_unions: Vec<Box<Term<'a>>> = Vec::new();
                        for item in unions.into_iter() {
                            let single_term = Term::Intersect(vec![item, Box::new(rhs.clone())]);
                            distributive_unions.push(Box::new(single_term.dnf()));
                        }

                        Term::Union(distributive_unions)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::calculate::Atom;

    use super::*;

    impl Atom for &str {
        fn probability(&self) -> f64 {
            0f64
        }

        fn name(&self) -> String {
            self.to_string()
        }
    }

    macro_rules! declare_atom {
        ($a:ident) => {
            let $a = Term::Atom(Arc::new(stringify!($a)));
        };
    }

    macro_rules! union {
        ($($term:ident),*) => {
            Term::Union(vec![$(Box::new($term.clone())),*])
        };
    }

    macro_rules! intersect {
        ($($term:ident),*) => {
            Term::Intersect(vec![$(Box::new($term.clone())),*])
        };
    }

    #[test]
    fn test_dnf() {
        declare_atom!(a);
        declare_atom!(b);
        declare_atom!(c);

        let term_union = union!(a, b);

        let term_intersect = intersect!(term_union, c);

        assert_eq!(
            format!("{:?}", term_intersect.dnf()),
            "Union([Intersect([Atom(\"a\"), Atom(\"c\")]), Intersect([Atom(\"b\"), Atom(\"c\")])])"
        )
    }

    #[test]
    fn test_deep_dnf() {
        declare_atom!(a);
        declare_atom!(b);
        declare_atom!(c);
        declare_atom!(d);
        declare_atom!(e);
        declare_atom!(f);
        declare_atom!(g);

        let a_b = intersect!(a, b);
        let c_d = intersect!(c, d);
        let a_b_c_d = union!(a_b, c_d);

        let f_g = intersect!(f, g);
        let e_f_g = union!(e, f_g);

        let final_term = intersect!(a_b_c_d, e_f_g);

        assert_eq!(
            format!("{:?}", final_term.dnf().flat()),
            "Union([Intersect([Atom(\"a\"), Atom(\"b\"), Atom(\"e\")]), Intersect([Atom(\"a\"), Atom(\"b\"), Atom(\"f\"), Atom(\"g\")]), Intersect([Atom(\"c\"), Atom(\"d\"), Atom(\"e\")]), Intersect([Atom(\"c\"), Atom(\"d\"), Atom(\"f\"), Atom(\"g\")])])"
        )
    }
}
