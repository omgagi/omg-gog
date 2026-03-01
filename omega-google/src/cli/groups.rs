//! Groups CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Groups service commands.
#[derive(Args, Debug)]
pub struct GroupsArgs {
    #[command(subcommand)]
    pub command: GroupsCommand,
}

#[derive(Subcommand, Debug)]
pub enum GroupsCommand {
    /// List groups
    List(GroupsListArgs),
    /// List members of a group
    Members(GroupsMembersArgs),
}

#[derive(Args, Debug)]
pub struct GroupsListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Fetch all pages
    #[arg(long)]
    pub all: bool,
    /// Fail if result is empty
    #[arg(long)]
    pub fail_empty: bool,
}

#[derive(Args, Debug)]
pub struct GroupsMembersArgs {
    /// Group email address
    pub group_email: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Fetch all pages
    #[arg(long)]
    pub all: bool,
    /// Fail if result is empty
    #[arg(long)]
    pub fail_empty: bool,
}
