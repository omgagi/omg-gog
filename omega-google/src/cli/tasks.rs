//! Tasks CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Tasks service commands.
#[derive(Args, Debug)]
pub struct TasksArgs {
    #[command(subcommand)]
    pub command: TasksCommand,
}

#[derive(Subcommand, Debug)]
pub enum TasksCommand {
    /// Task list operations
    Lists(TasksListsArgs),
    /// List tasks in a task list
    List(TasksListArgs),
    /// Get a task
    Get(TasksGetArgs),
    /// Add a new task
    Add(TasksAddArgs),
    /// Update a task
    Update(TasksUpdateArgs),
    /// Mark a task as done
    Done(TasksDoneArgs),
    /// Mark a task as not done
    Undo(TasksUndoArgs),
    /// Delete a task
    Delete(TasksDeleteArgs),
    /// Clear completed tasks
    Clear(TasksClearArgs),
}

// ---------------------------------------------------------------
// Task lists subcommands
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct TasksListsArgs {
    #[command(subcommand)]
    pub command: TasksListsCommand,
}

#[derive(Subcommand, Debug)]
pub enum TasksListsCommand {
    /// List all task lists
    List(TasksListsListArgs),
    /// Create a new task list
    Create(TasksListsCreateArgs),
}

#[derive(Args, Debug)]
pub struct TasksListsListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct TasksListsCreateArgs {
    /// Task list title
    pub title: String,
}

// ---------------------------------------------------------------
// Task operations
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct TasksListArgs {
    /// Task list ID
    pub tasklist: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct TasksGetArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task ID
    pub task: String,
}

#[derive(Args, Debug)]
pub struct TasksAddArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task title
    #[arg(long)]
    pub title: String,
    /// Task notes
    #[arg(long)]
    pub notes: Option<String>,
    /// Due date (RFC3339 or YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,
    /// Parent task ID (for subtasks)
    #[arg(long)]
    pub parent: Option<String>,
    /// Previous sibling task ID (for ordering)
    #[arg(long)]
    pub previous: Option<String>,
}

#[derive(Args, Debug)]
pub struct TasksUpdateArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task ID
    pub task: String,
    /// New title
    #[arg(long)]
    pub title: Option<String>,
    /// New notes
    #[arg(long)]
    pub notes: Option<String>,
    /// New due date
    #[arg(long)]
    pub due: Option<String>,
    /// New status (needsAction, completed)
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Args, Debug)]
pub struct TasksDoneArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task ID
    pub task: String,
}

#[derive(Args, Debug)]
pub struct TasksUndoArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task ID
    pub task: String,
}

#[derive(Args, Debug)]
pub struct TasksDeleteArgs {
    /// Task list ID
    pub tasklist: String,
    /// Task ID
    pub task: String,
}

#[derive(Args, Debug)]
pub struct TasksClearArgs {
    /// Task list ID
    pub tasklist: String,
}
