//! People CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google People service commands.
#[derive(Args, Debug)]
pub struct PeopleArgs {
    #[command(subcommand)]
    pub command: PeopleCommand,
}

#[derive(Subcommand, Debug)]
pub enum PeopleCommand {
    /// Get the authenticated user's profile
    Me,
    /// Get a person by resource name
    Get(PeopleGetArgs),
    /// Search people
    Search(PeopleSearchArgs),
    /// List connections/relations
    Relations(PeopleRelationsArgs),
}

#[derive(Args, Debug)]
pub struct PeopleGetArgs {
    /// Resource name (e.g., people/12345)
    pub resource_name: String,
}

#[derive(Args, Debug)]
pub struct PeopleSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct PeopleRelationsArgs {
    /// Resource name (default: "me")
    #[arg(long)]
    pub resource_name: Option<String>,
    /// Filter by relation type
    #[arg(long, name = "type")]
    pub relation_type: Option<String>,
}
