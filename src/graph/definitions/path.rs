use std::fmt::Debug;

use itertools::*;

use crate::Error;

use super::Edge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unit(pub ());

impl std::ops::Add for Unit {
    type Output = Self;
    fn add(self, _: Self) -> Self::Output {
        Self(())
    }
}

impl From<()> for Unit {
    fn from(_: ()) -> Self {
        Self(())
    }
}

pub struct ResultUnit(pub anyhow::Result<Unit>);

impl std::ops::Add for ResultUnit {
    type Output = Self;
    fn add(self, _: Self) -> Self::Output {
        match self.0 {
            Ok(_) => Self(Ok(Unit(()))),
            Err(_) => Self(Err(Error::UnexpectedError.into())),
        }
    }
}

impl From<()> for ResultUnit {
    fn from(_: ()) -> Self {
        Self(Ok(Unit(())))
    }
}

impl<T> From<std::io::Result<T>> for ResultUnit {
    fn from(value: std::io::Result<T>) -> Self {
        match value {
            Ok(_) => Self(Ok(Unit(()))),
            Err(_) => Self(Err(Error::IoError.into())),
        }
    }
}

impl From<Unit> for ResultUnit {
    fn from(value: Unit) -> Self {
        Self(Ok(value))
    }
}

impl<T> From<anyhow::Result<T>> for ResultUnit {
    fn from(value: anyhow::Result<T>) -> Self {
        match value {
            Ok(_) => Self(Ok(Unit(()))),
            Err(_) => Self(Err(Error::UnexpectedError.into())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path<V, E, Id>(pub Vec<Edge<V, E, Id>>)
where
    E: Clone,
    V: Clone;

impl<V, E, Id> Path<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: Clone,
{
    pub fn start_with(&self) -> Id {
        self.0.get(0).expect("Path is empty").get_start_id()
    }

    pub fn ends_with(&self) -> Id {
        self.0.get(0).expect("Path is empty").get_end_id()
    }
}

impl<V, E, Id> Path<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: Clone + PartialEq + Eq,
{
    pub fn contains(&self, id: Id) -> bool {
        let edges = &self.0;
        if edges.is_empty() {
            return false;
        }
        for edge in edges {
            if edge.get_start_id() == id || edge.get_end_id() == id {
                return true;
            }
        }
        false
    }

    pub fn subpath_between(&self, start: Id, end: Id) -> Result<Path<V, E, Id>, Error> {
        let edges = &self.0;
        if edges.is_empty() {
            return Err(Error::KeyWasNotFound);
        }

        let (abs_start, abs_end) = (self.start_with(), self.ends_with());
        if abs_start == start && abs_end == end {
            return Ok(Path(edges[..].to_owned()));
        }

        let mut found = (false, false);
        let (mut index_start, mut index_end) = (0, 0);
        for (i, edge) in edges.iter().enumerate() {
            if edge.get_start_id() == start {
                found.0 = true;
                index_start = i;
            }
            if edge.get_end_id() == end {
                found.1 = true;
                index_end = i;
            }
        }

        if let (true, true) = found {
            Ok(Path(edges[index_start..=index_end].to_owned()))
        } else {
            Err(Error::KeyWasNotFound)
        }
    }
}

impl<V, E, Id> PartialEq for Path<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        for (lhs, rhs) in self.0.iter().zip(&other.0) {
            if lhs != rhs {
                return false;
            }
        }
        true
    }
}

impl<V, E, Id> Eq for Path<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: PartialEq + Eq,
{
}

impl<V, E, Id> std::ops::Add for Path<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: PartialOrd + Ord + PartialEq + Eq + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if self.0.is_empty() {
            return rhs;
        }

        let self_end = self.0.last();
        if self_end.is_none() {
            return self;
        }
        let binding = self_end.unwrap().end.0.upgrade();
        let binding = binding.unwrap();
        let self_end = &binding.borrow().id;

        let other_start = rhs.0.last();
        if other_start.is_none() {
            return self;
        }
        let binding = other_start.unwrap().start.0.upgrade();
        let binding = binding.unwrap();
        let other_start = &binding.borrow().id;

        match self_end.cmp(other_start) {
            std::cmp::Ordering::Equal => Path([self.0, rhs.0].concat()),
            _ => self,
        }
    }
}

impl<V, E, Id> From<Path<V, E, Id>> for Vec<(V, E)>
where
    E: Clone,
    V: Clone,
    Id: Clone,
{
    fn from(value: Path<V, E, Id>) -> Self {
        let mut result = value
            .0
            .iter()
            .map(|edge| (edge.borrow_start_vertex().info.clone(), edge.info.clone()))
            .collect::<Vec<(V, E)>>();
        result.push((
            value.0.last().unwrap().borrow_end_vertex().info.clone(),
            value.0.last().unwrap().info.clone(), // unneccassary edge info
        ));
        result
    }
}

impl<V, E, Id> PartialEq for Paths<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: PartialEq + Eq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        for (lhs, rhs) in self.0.iter().zip(&other.0) {
            if lhs != rhs {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Paths<V, E, Id>(pub Vec<Path<V, E, Id>>)
where
    E: Clone,
    V: Clone,
    Id: Clone;

impl<V, E, Id> std::ops::Add for Paths<V, E, Id>
where
    E: Clone,
    V: Clone,
    Id: PartialOrd + Ord + PartialEq + Eq + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if self.0.is_empty() {
            rhs
        } else if rhs.0.is_empty() {
            self
        } else {
            iproduct!(self.0, rhs.0).fold(
                Paths(Vec::default()),
                |mut acc: Paths<V, E, Id>, (lhs, rhs)| {
                    let path = lhs + rhs;
                    acc.0.push(path);
                    acc
                },
            )
        }
    }
}
