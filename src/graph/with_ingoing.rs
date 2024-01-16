use std::cell::RefCell;
pub use super::{definitions::{Vicinity, WithIngoing, Vertex}, Graph};
use crate::Error;


#[allow(dead_code)]
impl<V, E, Id> Graph<V, E, Id, WithIngoing>
where
    V: Clone,
    E: Clone,
    Id: PartialOrd + Ord + PartialEq + Eq + Copy,
{
    pub fn add_vertex(
        &mut self,
        id: Id,
        info: V,
        vicinity: Vicinity<V, E, Id>,
    ) -> Result<(), Error> {
        if !matches!(&vicinity, Vicinity::Ingoing { edges: _ }) {
            return Err(Error::MismatchedVicinity);
        } else if self.vertices.contains(id) {
            return Err(Error::VertexAlreadyExists);
        }
        let vertex = Vertex::new(id, info, vicinity);
        self.vertices.insert(id, RefCell::new(vertex).into())?;
        Ok(())
    }
}
