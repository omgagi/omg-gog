//! Contacts CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Contacts service commands.
#[derive(Args, Debug)]
pub struct ContactsArgs {
    #[command(subcommand)]
    pub command: ContactsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ContactsCommand {
    /// Search contacts
    Search(ContactsSearchArgs),
    /// List all contacts
    List(ContactsListArgs),
    /// Get a contact
    Get(ContactsGetArgs),
    /// Create a contact
    Create(ContactsCreateArgs),
    /// Update a contact
    Update(ContactsUpdateArgs),
    /// Delete a contact
    Delete(ContactsDeleteArgs),
    /// Directory search and listing
    Directory(ContactsDirectoryArgs),
    /// Other contacts
    Other(ContactsOtherArgs),
}

// ---------------------------------------------------------------
// Contact operations (now direct subcommands of contacts)
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ContactsSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
}

#[derive(Args, Debug)]
pub struct ContactsListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ContactsGetArgs {
    /// Resource name (e.g., people/c12345)
    pub resource_name: String,
}

#[derive(Args, Debug)]
pub struct ContactsCreateArgs {
    /// Given name
    #[arg(long)]
    pub given: Option<String>,
    /// Family name
    #[arg(long)]
    pub family: Option<String>,
    /// Email address
    #[arg(long)]
    pub email: Option<String>,
    /// Phone number
    #[arg(long)]
    pub phone: Option<String>,
}

#[derive(Args, Debug)]
pub struct ContactsUpdateArgs {
    /// Resource name
    pub resource_name: String,
    /// Given name
    #[arg(long)]
    pub given: Option<String>,
    /// Family name
    #[arg(long)]
    pub family: Option<String>,
    /// Email address
    #[arg(long)]
    pub email: Option<String>,
    /// Phone number
    #[arg(long)]
    pub phone: Option<String>,
    /// Birthday (YYYY-MM-DD)
    #[arg(long)]
    pub birthday: Option<String>,
    /// Notes
    #[arg(long)]
    pub notes: Option<String>,
    /// JSON update from file (use - for stdin)
    #[arg(long)]
    pub from_file: Option<String>,
    /// Skip etag concurrency check
    #[arg(long)]
    pub ignore_etag: bool,
}

#[derive(Args, Debug)]
pub struct ContactsDeleteArgs {
    /// Resource name
    pub resource_name: String,
}

// ---------------------------------------------------------------
// Directory
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ContactsDirectoryArgs {
    #[command(subcommand)]
    pub command: ContactsDirectoryCommand,
}

#[derive(Subcommand, Debug)]
pub enum ContactsDirectoryCommand {
    /// List directory people
    List(ContactsDirectoryListArgs),
    /// Search directory people
    Search(ContactsDirectorySearchArgs),
}

#[derive(Args, Debug)]
pub struct ContactsDirectoryListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ContactsDirectorySearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
}

// ---------------------------------------------------------------
// Other contacts
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ContactsOtherArgs {
    #[command(subcommand)]
    pub command: ContactsOtherCommand,
}

#[derive(Subcommand, Debug)]
pub enum ContactsOtherCommand {
    /// List other contacts
    List(ContactsOtherListArgs),
    /// Search other contacts
    Search(ContactsOtherSearchArgs),
}

#[derive(Args, Debug)]
pub struct ContactsOtherListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ContactsOtherSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
}
