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

use super::{Term, Atom};

use std::{borrow::Cow, ffi::c_void};

#[derive(Clone)]
pub struct TermNode {
    content: String,
    id: usize,
}

#[derive(Clone)]
pub struct TermEdge {
    source: TermNode,
    target: TermNode,
}


impl<'a> dot::Labeller<'a, TermNode, TermEdge> for Term<'a> {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &TermNode) -> dot::Id<'a> {
        dot::Id::new(format!("n{}", n.id)).unwrap()
    }

    fn node_label(&'a self, n: &TermNode) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(Cow::Owned(n.content.clone()))
    }
}

impl<'a> Term<'a> {
    fn node(&self) -> Option<TermNode> {
        match self {
            Term::None => None,
            Term::Atom(atom) => {
                Some(TermNode {
                    content: atom.name(),
                    id: atom.as_ref() as *const dyn Atom as *const c_void as usize,
                })
            }
            Term::Union { lhs: _, rhs: _ } => {
                Some(TermNode {
                    content: "Union".to_string(),
                    id: self as *const Term as usize 
                })
            }
            Term::Intersect { lhs: _, rhs: _ } => {
                Some(TermNode {
                    content: "Intersect".to_string(),
                    id: self as *const Term as usize 
                })
            }
        }
    }
}

impl<'a> dot::GraphWalk<'a, TermNode, TermEdge> for Term<'a> {
    fn nodes(&self) -> dot::Nodes<'a, TermNode> {
        // (assumes that |N| \approxeq |E|)
        let mut nodes = Vec::new();

        match self {
            Term::None => {},
            Term::Atom(_) => {
                nodes.push(self.node().unwrap());
            },
            Term::Union { lhs, rhs } => {
                nodes.push(self.node().unwrap());
                let left_nodes = lhs.as_term().nodes().into_owned();
                let right_nodes = rhs.as_term().nodes().into_owned();
                nodes.extend(left_nodes);
                nodes.extend(right_nodes);
            },
            Term::Intersect { lhs, rhs } => {
                nodes.push(self.node().unwrap());
                let left_nodes = lhs.as_term().nodes().into_owned();
                let right_nodes = rhs.as_term().nodes().into_owned();
                nodes.extend(left_nodes);
                nodes.extend(right_nodes);
            },
        }

        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, TermEdge> {
        let mut edges = Vec::new();

        match self {
            Term::None => {},
            Term::Atom(_) => {},
            Term::Union { lhs, rhs } => {
                let from = self.node().unwrap();

                if let Some(left_target) = lhs.as_term().node() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: left_target.clone(),
                    });
                    edges.extend(lhs.as_term().edges().into_owned());
                }

                if let Some(right_target) = rhs.as_term().node() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: right_target.clone(),
                    });
                    edges.extend(rhs.as_term().edges().into_owned());
                }
            },
            Term::Intersect { lhs, rhs } => {
                let from = self.node().unwrap();

                if let Some(left_target) = lhs.as_term().node() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: left_target.clone(),
                    });
                    edges.extend(lhs.as_term().edges().into_owned());
                }

                if let Some(right_target) = rhs.as_term().node() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: right_target.clone(),
                    });
                    edges.extend(rhs.as_term().edges().into_owned());
                }
            },
        }
        
        Cow::Owned(edges)
    }

    fn source(&self, e: &TermEdge) -> TermNode { e.source.clone() }

    fn target(&self, e: &TermEdge) -> TermNode { e.target.clone() }
}

