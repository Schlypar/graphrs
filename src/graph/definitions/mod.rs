use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    rc::{Rc, Weak},
};

pub mod path;

use crate::graph::btree::key_value::Comparator;

pub type Shared<T> = Rc<RefCell<T>>;

#[derive(Clone)]
pub struct Observer<T>(pub Weak<RefCell<T>>);

pub type VertexFn<V, E, Id, R> = Box<dyn Fn(&Vertex<V, E, Id>) -> R>;
pub type VertexFnMut<V, E, Id, R> = Box<dyn Fn(&mut Vertex<V, E, Id>) -> R>;

impl<V, E, Id> Debug for Observer<Vertex<V, E, Id>>
where
    E: Clone + Debug,
    V: Debug,
    Id: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.upgrade() {
            Some(data) => write!(
                f,
                "Vertex: {{ id: {:?}, info: {:?}, ... }}",
                data.borrow().id,
                data.borrow().info
            ),
            None => write!(f, "(Null Pointer)"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Vicinity<V, E, Id>
where
    E: Clone,
{
    Outgoing {
        edges: Option<Vec<Edge<V, E, Id>>>,
    },
    Ingoing {
        edges: Option<Vec<Edge<V, E, Id>>>,
    },
    Both {
        ingoing_edges: Option<Vec<Edge<V, E, Id>>>,
        outgoing_edges: Option<Vec<Edge<V, E, Id>>>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct WithOutgoing;

#[derive(Debug, Clone, Default)]
pub struct WithIngoing;

#[derive(Debug, Clone, Default)]
pub struct WithBoth;

#[derive(Debug, Clone)]
pub struct Vertex<V, E, Id>
where
    E: Clone,
{
    pub id: Id,
    pub info: V,
    pub vicinity: Vicinity<V, E, Id>,
}

impl<V, E, Id> Vertex<V, E, Id>
where
    V: Clone,
    E: Clone,
    Id: Clone + PartialOrd + Ord,
{
    pub fn new(id: Id, info: V, vicinity: Vicinity<V, E, Id>) -> Self {
        Self { id, info, vicinity }
    }

    pub fn is_in_cycle(&self) -> bool {
        let mut discovered: Vec<Id> = Vec::default();
        let mut queue: VecDeque<&Vertex<V, E, Id>> = VecDeque::default();
        queue.push_back(self);

        while !queue.is_empty() {
            let current = queue.pop_front().unwrap();
            if discovered.contains(&current.id) {
                continue;
            } else {
                match &current.vicinity {
                    Vicinity::Outgoing { edges: Some(edges) } => {
                        for edge in edges {
                            if edge.get_end_id() == self.id {
                                return true;
                            }
                            queue.push_back(edge.borrow_end_vertex());
                            discovered.push(current.id.clone());
                        }
                    }
                    Vicinity::Both {
                        ingoing_edges: _,
                        outgoing_edges: Some(edges),
                    } => {
                        for edge in edges {
                            if edge.get_end_id() == self.id {
                                return true;
                            }
                            queue.push_back(edge.borrow_end_vertex());
                            discovered.push(current.id.clone());
                        }
                    }
                    Vicinity::Both {
                        ingoing_edges: _,
                        outgoing_edges: None,
                    } => continue,
                    Vicinity::Outgoing { edges: None } => continue,

                    _ => panic!("Shouldn't happent"),
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Edge<V, E, Id>
where
    E: Clone,
{
    pub info: E,
    pub start: Observer<Vertex<V, E, Id>>,
    pub end: Observer<Vertex<V, E, Id>>,
}

impl<V, E, Id> PartialEq for Edge<V, E, Id>
where
    E: Clone,
    Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let binding = self.start.0.upgrade().unwrap();
        let self_start = unsafe { &(*binding.as_ptr()).id };

        let binding = self.end.0.upgrade().unwrap();
        let self_end = unsafe { &(*binding.as_ptr()).id };

        let binding = other.start.0.upgrade().unwrap();
        let other_start = unsafe { &(*binding.as_ptr()).id };

        let binding = other.end.0.upgrade().unwrap();
        let other_end = unsafe { &(*binding.as_ptr()).id };

        (self_start == other_start) && (self_end == other_end)
    }
}

impl<V, E, Id> Eq for Edge<V, E, Id>
where
    E: Clone,
    Id: PartialEq,
{
}

impl<V, E, Id> Edge<V, E, Id>
where
    V: Clone,
    E: Clone,
    Id: Clone,
{
    pub fn new(info: E, start: Shared<Vertex<V, E, Id>>, end: Shared<Vertex<V, E, Id>>) -> Self {
        Self {
            info,
            start: Observer(Rc::downgrade(&start)),
            end: Observer(Rc::downgrade(&end)),
        }
    }

    pub fn get_start_id(&self) -> Id {
        let binding = self.start.0.upgrade().unwrap();
        let binding = binding.borrow();
        binding.id.clone()
    }

    pub fn get_end_id(&self) -> Id {
        let binding = self.end.0.upgrade().unwrap();
        let binding = binding.borrow();
        binding.id.clone()
    }

    pub fn get_start_info(&self) -> V {
        let binding = self.start.0.upgrade().unwrap();
        let binding = binding.borrow();
        binding.info.clone()
    }

    pub fn get_end_info(&self) -> V {
        let binding = self.end.0.upgrade().unwrap();
        let binding = binding.borrow();
        binding.info.clone()
    }

    pub fn get_start_all(&self) -> (Id, V) {
        let binding = self.start.0.upgrade().unwrap();
        let binding = binding.borrow();
        (binding.id.clone(), binding.info.clone())
    }

    pub fn get_end_all(&self) -> (Id, V) {
        let binding = self.end.0.upgrade().unwrap();
        let binding = binding.borrow();
        (binding.id.clone(), binding.info.clone())
    }

    pub fn borrow_start_vertex(&self) -> &Vertex<V, E, Id> {
        let binding = self.start.0.upgrade().unwrap();
        let vertex_ptr = binding.as_ptr();
        unsafe { &(*vertex_ptr) }
    }

    pub fn borrow_end_vertex(&self) -> &Vertex<V, E, Id> {
        let binding = self.end.0.upgrade().unwrap();
        let vertex_ptr = binding.as_ptr();
        unsafe { &(*vertex_ptr) }
    }

    pub fn borrow_start_vertex_mut(&mut self) -> &mut Vertex<V, E, Id> {
        let binding = self.start.0.upgrade().unwrap();
        let vertex_ptr = binding.as_ptr();
        unsafe { &mut (*vertex_ptr) }
    }

    pub fn borrow_end_vertex_mut(&mut self) -> &mut Vertex<V, E, Id> {
        let binding = self.end.0.upgrade().unwrap();
        let vertex_ptr = binding.as_ptr();
        unsafe { &mut (*vertex_ptr) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Comp {}

impl<Id: Ord> Comparator<Id> for Comp {
    fn compare(lhs: &Id, rhs: &Id) -> std::cmp::Ordering {
        lhs.cmp(rhs)
    }
}
