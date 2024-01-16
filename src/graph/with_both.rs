pub use super::{
    definitions::{
        path::{Path, Paths, ResultUnit, Unit},
        Vertex, VertexFn, VertexFnMut, Vicinity, WithBoth,
    },
    Graph,
};
use crate::Error;
use dot_writer::{Attributes, Color, DotWriter, Shape, Style};
use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Display,
    fs::File,
    io::Write,
    process::{Command, Stdio},
    str,
};

#[allow(dead_code)]
impl<V, E, Id> Graph<V, E, Id, WithBoth>
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
        if !matches!(
            &vicinity,
            Vicinity::Both {
                ingoing_edges: _,
                outgoing_edges: _,
            }
        ) {
            return Err(Error::MismatchedVicinity);
        } else if self.vertices.contains(id) {
            return Err(Error::VertexAlreadyExists);
        }
        let vertex = Vertex::new(id, info, vicinity);
        self.vertices.insert(id, RefCell::new(vertex).into())?;
        Ok(())
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
                acc = acc.add(map(unsafe { &(*vertex) }));

                if let Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: Some(edges),
                } = unsafe { &(*vertex).vicinity }
                {
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

                if let Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: Some(edges),
                } = unsafe { &(*vertex).vicinity }
                {
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
                acc = acc.add(map(unsafe { &mut (*vertex) }));

                if let Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: Some(edges),
                } = unsafe { &(*vertex).vicinity }
                {
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

                if let Vicinity::Both {
                    ingoing_edges: _,
                    outgoing_edges: Some(edges),
                } = unsafe { &(*vertex).vicinity }
                {
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
}
