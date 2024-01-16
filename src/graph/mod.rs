use crate::Error;
use std::{fmt::Debug, marker::PhantomData, rc::Rc};

pub mod definitions;
use definitions::{Comp, Edge, Shared, Vertex};
pub use definitions::{VertexFn, VertexFnMut, Vicinity, WithBoth, WithIngoing, WithOutgoing};

pub mod with_both;
pub mod with_ingoing;
pub mod with_outgoing;

pub mod btree;
use btree::BTree;

#[derive(Default, Debug, Clone)]
pub struct Graph<V, E, Id, S = WithBoth>
where
    V: Clone,
    E: Clone,
    Id: Ord + Copy,
{
    vertices: BTree<Id, Shared<Vertex<V, E, Id>>, Comp>,
    state: PhantomData<S>,
}

impl<V, E, Id, S> Graph<V, E, Id, S>
where
    V: Clone,
    E: Clone,
    Id: PartialOrd + Ord + PartialEq + Eq + Copy,
{
    pub fn add_edge(&mut self, info: E, start: Id, end: Id) -> Result<(), Error> {
        if !self.vertices.contains(start) || !self.vertices.contains(end) {
            return Err(Error::KeyWasNotFound);
        }

        let (start, end) = (self.vertices.search(start)?, self.vertices.search(end)?);
        let (mut start_borrowed, mut end_borrowed) = (start.borrow_mut(), end.borrow_mut());

        match (&mut start_borrowed.vicinity, &mut end_borrowed.vicinity) {
            (Vicinity::Outgoing { edges }, Vicinity::Outgoing { edges: _ }) => {
                match edges {
                    Some(edges) => edges.push(Edge::new(info, Rc::clone(start), Rc::clone(end))),
                    None => {
                        *edges = Some(vec![Edge::new(info, Rc::clone(start), Rc::clone(end))]);
                    }
                };
                Ok(())
            }
            (Vicinity::Ingoing { edges: _ }, Vicinity::Ingoing { edges }) => {
                match edges {
                    Some(edges) => edges.push(Edge::new(info, Rc::clone(start), Rc::clone(end))),
                    None => {
                        *edges = Some(vec![Edge::new(info, Rc::clone(start), Rc::clone(end))]);
                    }
                };
                Ok(())
            }
            (
                Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: outgoing,
                },
                Vicinity::Both {
                    ingoing_edges: ingoing,
                    outgoing_edges: _,
                },
            ) => {
                match outgoing {
                    Some(edges) => {
                        edges.push(Edge::new(info.clone(), Rc::clone(start), Rc::clone(end)))
                    }
                    None => {
                        *outgoing = Some(vec![Edge::new(
                            info.clone(),
                            Rc::clone(start),
                            Rc::clone(end),
                        )]);
                    }
                };
                match ingoing {
                    Some(edges) => edges.push(Edge::new(info, Rc::clone(start), Rc::clone(end))),
                    None => {
                        *ingoing = Some(vec![Edge::new(info, Rc::clone(start), Rc::clone(end))]);
                    }
                };

                Ok(())
            }
            _ => Err(Error::MismatchedVicinity),
        }
    }
}
