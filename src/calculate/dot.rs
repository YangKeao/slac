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

use std::borrow::Cow;

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
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example1").unwrap()
    }

    fn node_id(&'a self, n: &TermNode) -> dot::Id<'a> {
        dot::Id::new(format!("n{}", n.id)).unwrap()
    }

    fn node_label(&'a self, n: &TermNode) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(Cow::Owned(n.content.clone()))
    }
}

impl<'a> Term<'a> {
    fn node(&self) -> TermNode {
        match self {
            Term::None => TermNode {
                content: "None".to_string(),
                id: self as *const Term as usize,
            },
            Term::Not(_) => TermNode {
                content: "Not".to_string(),
                id: self as *const Term as usize,
            },
            Term::Atom(atom) => TermNode {
                content: atom.name(),
                id: self as *const Term as usize,
            },
            Term::Union(_) => TermNode {
                content: "Union".to_string(),
                id: self as *const Term as usize,
            },
            Term::Intersect(_) => TermNode {
                content: "Intersect".to_string(),
                id: self as *const Term as usize,
            },
        }
    }
}

impl<'a> dot::GraphWalk<'a, TermNode, TermEdge> for Term<'a> {
    fn nodes(&self) -> dot::Nodes<'a, TermNode> {
        // (assumes that |N| \approxeq |E|)
        let mut nodes = Vec::new();

        match self {
            Term::None => {
                nodes.push(self.node());
            }
            Term::Atom(_) => {
                nodes.push(self.node());
            }
            Term::Not(term) => {
                nodes.push(self.node());
                nodes.extend(term.as_term().nodes().into_owned());
            }
            Term::Union(unions) => {
                nodes.push(self.node());
                for union in unions
                    .iter()
                    .map(|item| item.as_term().nodes().into_owned())
                {
                    nodes.extend(union);
                }
            }
            Term::Intersect(intersects) => {
                nodes.push(self.node());
                for intersect in intersects
                    .iter()
                    .map(|item| item.as_term().nodes().into_owned())
                {
                    nodes.extend(intersect);
                }
            }
        }

        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, TermEdge> {
        let mut edges = Vec::new();

        match self {
            Term::None => {}
            Term::Atom(_) => {}
            Term::Not(term) => {
                let from = self.node();

                edges.push(TermEdge {
                    source: from.clone(),
                    target: term.as_term().node().clone(),
                });
                edges.extend(term.as_term().edges().into_owned());
            }
            Term::Union(unions) => {
                let from = self.node();

                for union in unions.iter() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: union.as_term().node().clone(),
                    });
                    edges.extend(union.as_term().edges().into_owned());
                }
            }
            Term::Intersect(intersects) => {
                let from = self.node();

                for intersect in intersects.iter() {
                    edges.push(TermEdge {
                        source: from.clone(),
                        target: intersect.as_term().node().clone(),
                    });
                    edges.extend(intersect.as_term().edges().into_owned());
                }
            }
        }

        Cow::Owned(edges)
    }

    fn source(&self, e: &TermEdge) -> TermNode {
        e.source.clone()
    }

    fn target(&self, e: &TermEdge) -> TermNode {
        e.target.clone()
    }
}
