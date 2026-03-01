//! Apps Script CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Apps Script service commands.
#[derive(Args, Debug)]
pub struct AppScriptArgs {
    #[command(subcommand)]
    pub command: AppScriptCommand,
}

#[derive(Subcommand, Debug)]
pub enum AppScriptCommand {
    /// Get project metadata
    Get(AppScriptGetArgs),
    /// Get project content (source files)
    Content(AppScriptContentArgs),
    /// Run a script function
    Run(AppScriptRunArgs),
    /// Create a new project
    Create(AppScriptCreateArgs),
}

#[derive(Args, Debug)]
pub struct AppScriptGetArgs {
    /// Script ID or URL
    pub script_id: String,
}

#[derive(Args, Debug)]
pub struct AppScriptContentArgs {
    /// Script ID or URL
    pub script_id: String,
}

#[derive(Args, Debug)]
pub struct AppScriptRunArgs {
    /// Script ID or URL
    pub script_id: String,
    /// Function name to execute
    pub function: String,
    /// JSON parameters (array or single value)
    #[arg(long)]
    pub params: Option<String>,
    /// Run in development mode
    #[arg(long)]
    pub dev_mode: bool,
}

#[derive(Args, Debug)]
pub struct AppScriptCreateArgs {
    /// Project title
    #[arg(long)]
    pub title: String,
    /// Parent document ID (e.g., spreadsheet, doc)
    #[arg(long)]
    pub parent_id: Option<String>,
}
