use crate::error::{WorkflowError, WorkflowResult};
use crate::task::Task;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct TaskGraph {
    pub tasks: HashMap<String, Task>,
}

impl TaskGraph {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks: tasks.into_iter().map(|t| (t.id.clone(), t)).collect() }
    }

    pub fn validate(&self, workflow_id: Uuid) -> WorkflowResult<()> {
        for task in self.tasks.values() {
            for dep in &task.depends_on {
                if !self.tasks.contains_key(dep) {
                    return Err(WorkflowError::UnknownTask { workflow_id, task: dep.clone() });
                }
            }
        }
        self.detect_cycles(workflow_id)?;
        Ok(())
    }

    fn detect_cycles(&self, workflow_id: Uuid) -> WorkflowResult<()> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for id in self.tasks.keys() {
            if !visited.contains(id) {
                self.visit(id, &mut visited, &mut in_stack, workflow_id)?;
            }
        }
        Ok(())
    }

    fn visit(&self, id: &str, visited: &mut HashSet<String>, in_stack: &mut HashSet<String>, workflow_id: Uuid) -> WorkflowResult<()> {
        visited.insert(id.to_string());
        in_stack.insert(id.to_string());

        if let Some(task) = self.tasks.get(id) {
            for dep in &task.depends_on {
                if in_stack.contains(dep) {
                    return Err(WorkflowError::CircularDependency(workflow_id, dep.clone()));
                }
                if !visited.contains(dep) {
                    self.visit(dep, visited, in_stack, workflow_id)?;
                }
            }
        }
        in_stack.remove(id);
        Ok(())
    }

    pub fn ready_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|t| {
            t.state == crate::task::TaskState::Created
                && t.depends_on.iter().all(|d| {
                    self.tasks.get(d).map(|dep| dep.state == crate::task::TaskState::Completed).unwrap_or(false)
                })
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_no_cycle_in_linear_chain() {
        let graph = TaskGraph::new(vec![
            Task::new("a", "step a", "cap.a"),
            Task::new("b", "step b", "cap.b").depends_on("a"),
        ]);
        assert!(graph.validate(Uuid::new_v4()).is_ok());
    }

    #[test]
    fn detects_direct_cycle() {
        let graph = TaskGraph::new(vec![
            Task::new("a", "step a", "cap.a").depends_on("b"),
            Task::new("b", "step b", "cap.b").depends_on("a"),
        ]);
        assert!(graph.validate(Uuid::new_v4()).is_err());
    }

    #[test]
    fn unknown_dependency_is_rejected() {
        let graph = TaskGraph::new(vec![
            Task::new("a", "step a", "cap.a").depends_on("ghost"),
        ]);
        assert!(graph.validate(Uuid::new_v4()).is_err());
    }

    #[test]
    fn ready_tasks_respects_dependencies() {
        let graph = TaskGraph::new(vec![
            Task::new("a", "step a", "cap.a"),
            Task::new("b", "step b", "cap.b").depends_on("a"),
        ]);
        let ready = graph.ready_tasks();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "a");
    }
}
