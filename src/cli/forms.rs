//! Forms CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Forms service commands.
#[derive(Args, Debug)]
pub struct FormsArgs {
    #[command(subcommand)]
    pub command: FormsCommand,
}

#[derive(Subcommand, Debug)]
pub enum FormsCommand {
    /// Get form metadata
    Get(FormsGetArgs),
    /// Create a new form
    Create(FormsCreateArgs),
    /// Form responses operations
    Responses(FormsResponsesArgs),
}

#[derive(Args, Debug)]
pub struct FormsGetArgs {
    /// Form ID
    pub form_id: String,
}

#[derive(Args, Debug)]
pub struct FormsCreateArgs {
    /// Form title
    #[arg(long)]
    pub title: String,
    /// Form description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct FormsResponsesArgs {
    #[command(subcommand)]
    pub command: FormsResponsesCommand,
}

#[derive(Subcommand, Debug)]
pub enum FormsResponsesCommand {
    /// List form responses
    List(FormsResponsesListArgs),
    /// Get a specific response
    Get(FormsResponsesGetArgs),
}

#[derive(Args, Debug)]
pub struct FormsResponsesListArgs {
    /// Form ID
    pub form_id: String,
    /// Maximum results
    #[arg(long)]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Filter expression
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Args, Debug)]
pub struct FormsResponsesGetArgs {
    /// Form ID
    pub form_id: String,
    /// Response ID
    pub response_id: String,
}
