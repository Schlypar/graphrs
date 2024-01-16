use std::{
    collections::{HashMap, VecDeque},
    io,
};

use graphrs::{
    graph::{Graph, Vicinity, WithOutgoing},
    Error,
};

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    dependency_of: Option<u32>,
    task: String,
}

fn main() -> anyhow::Result<()> {
    let mut graph: Graph<String, (), u32, WithOutgoing> = Graph::default();
    let mut task_stack: VecDeque<Task> = VecDeque::default();
    let mut tasks: HashMap<u32, Task> = HashMap::default();

    let mut current_id: u32 = 0;

    println!("Which task do you want to solve?");
    let mut task = String::default();
    io::stdin().read_line(&mut task)?;

    let task = Task {
        id: current_id,
        dependency_of: None,
        task,
    };
    task_stack.push_back(task);
    current_id += 1;

    while !task_stack.is_empty() {
        let prev_task = task_stack
            .get(task_stack.len() - 1)
            .ok_or(Error::UnexpectedError)?;
        println!("Current task is: {}", &prev_task.task);
        println!("Does that depends on another task?");

        let mut depends = String::default();
        io::stdin().read_line(&mut depends)?;
        let depends: bool = {
            if depends.to_lowercase().eq("yes\n")
                || depends.to_lowercase().eq("true\n")
                || depends.to_lowercase().eq("y\n")
            {
                true
            } else {
                false
            }
        };

        if !depends {
            let curr_task = task_stack.pop_back().ok_or(Error::UnexpectedError)?;
            tasks.insert(curr_task.id, curr_task);
        } else {
            println!("Which task do you want to solve?");
            let mut task = String::default();
            io::stdin().read_line(&mut task)?;

            let task = Task {
                id: current_id,
                dependency_of: Some(prev_task.id),
                task,
            };
            task_stack.push_back(task);
            current_id += 1;
        }
    }

    tasks.iter().for_each(|(_, task)| {
        let _ = graph.add_vertex(
            task.id,
            task.task.clone(),
            Vicinity::Outgoing { edges: None },
        );
    });
    tasks.iter().for_each(|(_, task)| {
        if let Some(dependency) = task.dependency_of {
            let _ = graph.add_edge((), dependency, task.id);
        }
    });

    let dependencies = graph.topological_sort(0)?;

    println!("Order of tasks to complete:");
    let mut num = 1;
    dependencies.iter().rev().for_each(|id| {
        let task = tasks.get(id).unwrap();
        print!("{num}) {}", task.task);
        num += 1;
    });

    Ok(())
}
