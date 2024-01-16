pub use super::{
    definitions::{
        path::{Path, Paths, ResultUnit, Unit},
        Vertex, VertexFn, VertexFnMut, Vicinity, WithOutgoing,
    },
    Graph,
};
use crate::Error;
use dot_writer::{Attributes, Color, DotWriter, Shape, Style};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::VecDeque,
    fmt::{Debug, Display},
    fs::File,
    io::Write,
    process::{Command, Stdio},
    rc::Rc,
    str,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Mark<Id>
where
    Id: Ord + Clone,
{
    Permanent(Id),
    Temporary(Id),
    Unmarked(Id),
}

#[allow(dead_code)]
impl<V, E, Id> Graph<V, E, Id, WithOutgoing>
where
    V: Clone,
    E: Clone,
    Id: PartialOrd + Ord + PartialEq + Eq + Copy + Clone,
{
    pub fn add_vertex(
        &mut self,
        id: Id,
        info: V,
        vicinity: Vicinity<V, E, Id>,
    ) -> Result<(), Error> {
        if !matches!(&vicinity, Vicinity::Outgoing { edges: _ }) {
            return Err(Error::MismatchedVicinity);
        } else if self.vertices.contains(id) {
            return Err(Error::VertexAlreadyExists);
        }
        let vertex = Vertex::new(id, info, vicinity);
        self.vertices.insert(id, RefCell::new(vertex).into())?;
        Ok(())
    }

    pub fn is_acyclic(&self) -> bool {
        for (_, vertex) in self.vertices.into_vec() {
            if vertex.borrow().is_in_cycle() {
                return false;
            }
        }
        true
    }

    pub fn depth_first_traversal<R>(
        &self,
        initial_id: Id,
        mut acc: R,
        map: VertexFn<V, E, Id, R>,
    ) -> Result<R, Error>
    where
        R: std::ops::Add<Output = R>,
    {
        let mut discovered: Vec<Id> = Vec::default();
        let mut stack: VecDeque<Id> = VecDeque::default();
        stack.push_back(initial_id);

        while !stack.is_empty() {
            let id = stack.pop_back().ok_or(Error::UnexpectedError)?;
            if !discovered.contains(&id) {
                discovered.push(id);

                let vertex = self.vertices.search(id)?.as_ptr();
                acc = acc + map(unsafe { &(*vertex) });

                if let Vicinity::Outgoing { edges: Some(edges) } = unsafe { &(*vertex).vicinity } {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().ok_or(Error::NullPointer);
                        let id = binding?.borrow().id;
                        stack.push_back(id);
                    }
                }
            } else {
                continue;
            }
        }
        Ok(acc)
    }

    pub fn breadth_first_traversal<R>(
        &self,
        initial_id: Id,
        mut acc: R,
        map: VertexFn<V, E, Id, R>,
    ) -> Result<R, Error>
    where
        R: std::ops::Add<Output = R>,
    {
        let mut discovered: Vec<Id> = Vec::default();
        let mut queue: VecDeque<Id> = VecDeque::default();
        queue.push_back(initial_id);

        while !queue.is_empty() {
            let id = queue.pop_front().ok_or(Error::UnexpectedError)?;
            if !discovered.contains(&id) {
                discovered.push(id);

                let vertex = self.vertices.search(id)?.as_ptr();
                acc = acc + map(unsafe { &(*vertex) });

                if let Vicinity::Outgoing { edges: Some(edges) } = unsafe { &(*vertex).vicinity } {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().ok_or(Error::NullPointer);
                        let id = binding?.borrow().id;
                        queue.push_back(id);
                    }
                }
            } else {
                continue;
            }
        }
        Ok(acc)
    }

    pub fn depth_first_traversal_mut<R>(
        &mut self,
        initial_id: Id,
        mut acc: R,
        map: VertexFnMut<V, E, Id, R>,
    ) -> Result<R, Error>
    where
        R: std::ops::Add<Output = R>,
    {
        let mut discovered: Vec<Id> = Vec::default();
        let mut stack: VecDeque<Id> = VecDeque::default();
        stack.push_back(initial_id);

        while !stack.is_empty() {
            let id = stack.pop_back().ok_or(Error::UnexpectedError)?;
            if !discovered.contains(&id) {
                discovered.push(id);

                let vertex = self.vertices.search(id)?.as_ptr();
                acc = acc + map(unsafe { &mut (*vertex) });

                if let Vicinity::Outgoing { edges: Some(edges) } = unsafe { &(*vertex).vicinity } {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().ok_or(Error::NullPointer);
                        let id = binding?.borrow().id;
                        stack.push_back(id);
                    }
                }
            } else {
                continue;
            }
        }
        Ok(acc)
    }

    pub fn breadth_first_traversal_mut<R>(
        &mut self,
        initial_id: Id,
        mut acc: R,
        map: VertexFnMut<V, E, Id, R>,
    ) -> Result<R, Error>
    where
        R: std::ops::Add<Output = R>,
    {
        let mut discovered: Vec<Id> = Vec::default();
        let mut queue: VecDeque<Id> = VecDeque::default();
        queue.push_back(initial_id);

        while !queue.is_empty() {
            let id = queue.pop_front().ok_or(Error::UnexpectedError)?;
            if !discovered.contains(&id) {
                discovered.push(id);

                let vertex = self.vertices.search(id)?.as_ptr();
                acc = acc + map(unsafe { &mut (*vertex) });

                if let Vicinity::Outgoing { edges: Some(edges) } = unsafe { &(*vertex).vicinity } {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().ok_or(Error::NullPointer);
                        let id = binding?.borrow().id;
                        queue.push_back(id);
                    }
                }
            } else {
                continue;
            }
        }
        Ok(acc)
    }

    pub fn all_paths_from(&self, id: Id) -> Result<Paths<V, E, Id>, Error> {
        let create_paths = |v: &Vertex<V, E, Id>| -> Paths<V, E, Id> {
            match &v.vicinity {
                Vicinity::Outgoing { edges } => {
                    if let Some(edges) = edges {
                        let mut paths: Vec<Path<V, E, Id>> = vec![];
                        for edge in edges {
                            let path = Path(vec![edge.clone()]);
                            paths.push(path);
                        }
                        Paths(paths)
                    } else {
                        Paths(vec![])
                    }
                }
                Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: edges,
                } => {
                    if let Some(edges) = edges {
                        let mut paths: Vec<Path<V, E, Id>> = vec![];
                        for edge in edges {
                            let path = Path(vec![edge.clone()]);
                            paths.push(path);
                        }
                        Paths(paths)
                    } else {
                        Paths(vec![])
                    }
                }
                _ => panic!("Cannot do if I only know about ingoing edges"),
            }
        };

        self.breadth_first_traversal(id, Paths(Vec::default()), Box::new(create_paths))
    }

    pub fn dump_to_file(&self, initial_id: Id, file: &RefCell<std::fs::File>) -> ResultUnit
    where
        Id: Display,
    {
        let file = file.as_ptr();
        let writer = RefCell::new(DotWriter::from(unsafe { &mut (*file) })).as_ptr();
        let writer = unsafe { &mut (*writer) };
        let digraph = RefCell::new(writer.digraph());

        digraph.borrow_mut().set_font("FiraCode Mone Nerd Font");
        digraph.borrow_mut().set_shape(Shape::Mrecord);
        digraph.borrow_mut().set_background_color(Color::Gray20);
        digraph.borrow_mut().set_style(Style::Filled);
        {
            let mut bind = digraph.borrow_mut();
            let mut node_attr = bind.node_attributes();
            node_attr.set_style(Style::Filled);
            node_attr.set_shape(Shape::Circle);
            node_attr.set_font("FiraCode Mono Nerd Font");
            node_attr.set_color(Color::LightGrey);
        }
        {
            let mut bind = digraph.borrow_mut();
            let mut edge_attr = bind.edge_attributes();
            edge_attr.set_color(Color::White);
        }

        let digraph = digraph.as_ptr();

        let dump = move |v: &Vertex<V, E, Id>| -> ResultUnit {
            match &v.vicinity {
                Vicinity::Outgoing { edges: Some(edges) } => {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().unwrap();
                        let edge_id = binding.borrow().id;
                        let digraph = unsafe { &mut (*digraph) };
                        digraph.edge(v.id.to_string(), edge_id.to_string());
                    }
                    Unit(()).into()
                }
                Vicinity::Ingoing { edges: Some(edges) } => {
                    for edge in edges {
                        let binding = edge.end.0.upgrade().unwrap();
                        let edge_id = binding.borrow().id;
                        let digraph = unsafe { &mut (*digraph) };
                        digraph.edge(v.id.to_string(), edge_id.to_string());
                    }
                    Unit(()).into()
                }
                Vicinity::Both {
                    ingoing_edges: Some(ingoing_edges),
                    outgoing_edges: _,
                } => {
                    for edge in ingoing_edges {
                        let binding = edge.end.0.upgrade().unwrap();
                        let edge_id = binding.borrow().id;
                        let digraph = unsafe { &mut (*digraph) };
                        digraph.edge(v.id.to_string(), edge_id.to_string());
                    }

                    Unit(()).into()
                }
                _ => Unit(()).into(),
            }
        };

        match self.breadth_first_traversal(initial_id, Unit(()).into(), Box::new(dump)) {
            Ok(_) => ResultUnit(Ok(Unit(()))),
            Err(e) => ResultUnit(Err(e.into())),
        }
    }

    pub fn dump_to_file_ext(&self, initial_id: Id, path: &std::path::Path) -> anyhow::Result<()>
    where
        Id: Display,
    {
        let file = match File::create(path) {
            Ok(file) => file,
            Err(_) => {
                return Err(Error::IoError.into());
            }
        };
        let file = RefCell::new(file);

        self.dump_to_file(initial_id, &file);

        let cat = Command::new("cat")
            .arg(path.to_str().unwrap())
            .stdout(Stdio::piped())
            .spawn()?;
        let dot = Command::new("dot")
            .stdin(Stdio::from(cat.stdout.unwrap()))
            .args(["-Tsvg"])
            .output()?;
        let mut file = File::create(path)?;
        file.sync_all()?;
        file.set_len(0)?;
        write!(file, "{}", str::from_utf8(&dot.stdout)?)?;

        Ok(())
    }

    pub fn all_paths_between(&self, start: Id, end: Id) -> Option<Vec<Path<V, E, Id>>>
    where
        Id: Default + std::ops::Add<Output = Id>,
    {
        let paths_between = self
            .all_paths_from(start)
            .ok()?
            .0
            .iter()
            .filter_map(|path| -> Option<Path<V, E, Id>> { path.subpath_between(start, end).ok() })
            .collect::<Vec<Path<V, E, Id>>>();

        if paths_between.is_empty() {
            None
        } else {
            Some(paths_between)
        }
    }

    pub fn topological_sort(&self, start_id: Id) -> Result<VecDeque<Id>, Error>
    where
        Id: Debug,
        V: Debug,
        E: Debug,
    {
        let mut marks: Vec<Mark<Id>> = vec![Mark::Unmarked(start_id)];
        let mut dependencies: VecDeque<Id> = VecDeque::default();
        let all_marks_are_permanent = |marks: &Vec<Mark<Id>>| -> bool {
            marks.iter().fold(true, |mut acc, mark| {
                if let Mark::Permanent(_) = mark {
                    acc
                } else {
                    acc = false;
                    acc
                }
            })
        };
        let first_unmarked = |marks: &Vec<Mark<Id>>| -> Id {
            for mark in marks {
                if let Mark::Unmarked(id) = mark {
                    return *id;
                }
            }
            panic!("At least one Id should be unmarked");
        };
        while !all_marks_are_permanent(&marks) {
            let unmarked_id = first_unmarked(&marks);
            let vertex = self.vertices.search(unmarked_id)?;
            Graph::visit_node(vertex, &mut marks, &mut dependencies)?;
        }
        Ok(dependencies)
    }

    fn visit_node(
        v: &Rc<RefCell<Vertex<V, E, Id>>>,
        marks: &mut Vec<Mark<Id>>,
        dependencies: &mut VecDeque<Id>,
    ) -> Result<(), Error>
    where
        Id: Debug,
        V: Debug,
        E: Debug,
    {
        let v = v.borrow();
        let v_id = v.id;
        let mut pos = marks.len();
        let mut idx = 0;
        marks.iter().for_each(|mark| match mark {
            Mark::Permanent(id) => {
                if let Ordering::Equal = id.cmp(&v_id) {
                    pos = idx
                }
                idx += 1;
            }
            Mark::Temporary(id) => {
                if let Ordering::Equal = id.cmp(&v_id) {
                    pos = idx
                }
                idx += 1;
            }
            Mark::Unmarked(id) => {
                if let Ordering::Equal = id.cmp(&v_id) {
                    pos = idx
                }
                idx += 1;
            }
        });
        if pos == marks.len() {
            marks.push(Mark::Unmarked(v_id));
        }
        match marks[pos] {
            Mark::Permanent(_) => {
                return Ok(());
            }
            Mark::Temporary(_) => {
                return Err(Error::WithMessage("Graph contains cycle"));
            }
            Mark::Unmarked(id) => match &v.vicinity {
                Vicinity::Outgoing { edges: Some(edges) } => {
                    marks[pos] = Mark::Temporary(id);
                    for edge in edges {
                        let vertex = edge.end.0.upgrade().ok_or(Error::NullPointer)?;
                        Graph::visit_node(&vertex, marks, dependencies)?;
                    }
                    marks[pos] = Mark::Permanent(id);
                    dependencies.push_front(id);
                }
                Vicinity::Outgoing { edges: None } => {
                    marks[pos] = Mark::Permanent(id);
                    dependencies.push_front(id);
                    return Ok(());
                }
                _ => {
                    return Err(Error::MismatchedVicinity);
                }
            },
        };
        Ok(())
    }
}
