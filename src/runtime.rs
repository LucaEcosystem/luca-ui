//! Minimal lifecycle runtime for 0.5 Alpha (`runtime/core` in ARCHITECTURE).
//!
//! Locked facts from `docs/SPEC.md`: component lifecycle is managed by the
//! runtime, and component-scoped asynchronous work is cancelled when its
//! component leaves the rendered tree. Lifecycle and task APIs are NOT
//! locked, so this module is a minimal compatible assumption
//! (see `docs/implementation-status.md`):
//!
//! - `Runtime` owns the current tree plus its `State`. `update_tree` swaps
//!   in a replacement render: it reports which component ids mounted and
//!   unmounted, merges newly declared state (existing variables keep their
//!   committed values, so state outlives any single tree), prunes listeners
//!   of removed components, and cancels tasks scoped to removed components.
//! - Scoped tasks stand in for component-scoped asynchronous work. There is
//!   no async executor yet; a task is a labelled unit of work whose
//!   cancellation flag the (later) executor will honor. Scoping a task to an
//!   unmounted component is a Luca error, and unmounting cancels its tasks.
//! - Event dispatch stays declarative (`events::dispatch`): `Runtime` only
//!   routes; Luca Code logic runs on the existing seam and feeds back solely
//!   through `State::set`/`flush`.

use std::collections::{HashMap, HashSet};

use crate::{dispatch, Component, Dispatch, LucaError, ResolvedComponent, State};

/// What one tree replacement mounted and unmounted, by component id.
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleDiff {
    pub mounted: Vec<String>,
    pub unmounted: Vec<String>,
}

impl LifecycleDiff {
    /// True when nothing mounted or unmounted.
    pub fn is_empty(&self) -> bool {
        self.mounted.is_empty() && self.unmounted.is_empty()
    }
}

/// A labelled unit of component-scoped work. Cancellation is cooperative:
/// executors must honor `cancelled` (no executor exists yet in 0.5).
#[derive(Debug, Clone, PartialEq)]
pub struct ScopedTask {
    pub id: u64,
    pub scope: String,
    pub label: String,
    pub cancelled: bool,
}

/// The 0.5 runtime: current tree, application state, mounted ids, and
/// component-scoped tasks.
#[derive(Debug)]
pub struct Runtime {
    tree: Component,
    state: State,
    mounted: HashSet<String>,
    tasks: HashMap<u64, ScopedTask>,
    next_task: u64,
}

impl Runtime {
    /// Start from a parsed tree; declares its state and marks its ids mounted.
    pub fn new(tree: Component) -> Result<Self, LucaError> {
        let state = State::from_tree(&tree)?;
        let mounted = tree.ids().into_iter().map(str::to_owned).collect();
        Ok(Self { tree, state, mounted, tasks: HashMap::new(), next_task: 1 })
    }

    /// The current tree.
    pub fn tree(&self) -> &Component {
        &self.tree
    }

    /// Application state (read).
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Application state (updates go through `set`/`flush` only).
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// Resolve UI output from committed state.
    pub fn resolve(&self) -> Result<ResolvedComponent, LucaError> {
        crate::resolve_tree(&self.tree, &self.state)
    }

    /// Route an event declared in the current tree to its handler name.
    pub fn dispatch(&self, component_id: &str, event: &str) -> Result<Dispatch, LucaError> {
        dispatch(&self.tree, component_id, event)
    }

    /// Swap in a replacement render. Reports mounted/unmounted ids, merges
    /// newly declared state, prunes listeners of removed components, and
    /// cancels tasks scoped to removed components.
    pub fn update_tree(&mut self, new_tree: Component) -> Result<LifecycleDiff, LucaError> {
        self.state.merge_tree(&new_tree)?;
        let next: HashSet<String> = new_tree.ids().into_iter().map(str::to_owned).collect();
        let mut mounted: Vec<String> = next.difference(&self.mounted).cloned().collect();
        let mut unmounted: Vec<String> = self.mounted.difference(&next).cloned().collect();
        mounted.sort();
        unmounted.sort();
        self.state.prune(&new_tree);
        self.mounted = next;
        self.tree = new_tree;
        for task in self.tasks.values_mut() {
            if !self.mounted.contains(&task.scope) {
                task.cancelled = true;
            }
        }
        Ok(LifecycleDiff { mounted, unmounted })
    }

    /// Scope labelled work to a mounted component. Errors when the component
    /// is not currently mounted.
    pub fn scope_task(&mut self, component_id: &str, label: &str) -> Result<u64, LucaError> {
        if !self.mounted.contains(component_id) {
            return Err(LucaError::new(
                format!("Cannot scope work to unmounted component '{component_id}'"),
                self.tree.line,
                self.tree.column,
            ));
        }
        let id = self.next_task;
        self.next_task += 1;
        self.tasks.insert(
            id,
            ScopedTask { id, scope: component_id.to_owned(), label: label.to_owned(), cancelled: false },
        );
        Ok(id)
    }

    /// Look up a task by id.
    pub fn task(&self, id: u64) -> Option<&ScopedTask> {
        self.tasks.get(&id)
    }

    /// Explicitly cancel one task; returns false when the id is unknown.
    pub fn cancel_task(&mut self, id: u64) -> bool {
        match self.tasks.get_mut(&id) {
            Some(task) => {
                task.cancelled = true;
                true
            }
            None => false,
        }
    }
}
