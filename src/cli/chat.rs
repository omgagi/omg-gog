//! Chat CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Chat service commands.
#[derive(Args, Debug)]
pub struct ChatArgs {
    #[command(subcommand)]
    pub command: ChatCommand,
}

#[derive(Subcommand, Debug)]
pub enum ChatCommand {
    /// Space operations
    Spaces(ChatSpacesArgs),
    /// Message operations
    Messages(ChatMessagesArgs),
    /// Thread operations
    Threads(ChatThreadsArgs),
    /// Direct message operations
    Dm(ChatDmArgs),
}

// ---------------------------------------------------------------
// Spaces subcommands
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ChatSpacesArgs {
    #[command(subcommand)]
    pub command: ChatSpacesCommand,
}

#[derive(Subcommand, Debug)]
pub enum ChatSpacesCommand {
    /// List all spaces
    List(ChatSpacesListArgs),
    /// Find a space by display name
    Find(ChatSpacesFindArgs),
    /// Create a new space
    Create(ChatSpacesCreateArgs),
}

#[derive(Args, Debug)]
pub struct ChatSpacesListArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ChatSpacesFindArgs {
    /// Space display name to search for
    pub name: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
}

#[derive(Args, Debug)]
pub struct ChatSpacesCreateArgs {
    /// Space display name
    pub name: String,
    /// Members to add (user resource names)
    #[arg(long)]
    pub member: Vec<String>,
}

// ---------------------------------------------------------------
// Messages subcommands
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ChatMessagesArgs {
    #[command(subcommand)]
    pub command: ChatMessagesCommand,
}

#[derive(Subcommand, Debug)]
pub enum ChatMessagesCommand {
    /// List messages in a space
    List(ChatMessagesListArgs),
    /// Send a message to a space
    Send(ChatMessagesSendArgs),
}

#[derive(Args, Debug)]
pub struct ChatMessagesListArgs {
    /// Space name (e.g., spaces/AAAA)
    pub space: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Order by (e.g., "createTime desc")
    #[arg(long)]
    pub order: Option<String>,
    /// Filter by thread name
    #[arg(long)]
    pub thread: Option<String>,
    /// Show only unread messages
    #[arg(long)]
    pub unread: bool,
}

#[derive(Args, Debug)]
pub struct ChatMessagesSendArgs {
    /// Space name (e.g., spaces/AAAA)
    pub space: String,
    /// Message text
    #[arg(long)]
    pub text: String,
    /// Thread name to reply in
    #[arg(long)]
    pub thread: Option<String>,
}

// ---------------------------------------------------------------
// Threads subcommands
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ChatThreadsArgs {
    #[command(subcommand)]
    pub command: ChatThreadsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ChatThreadsCommand {
    /// List threads in a space
    List(ChatThreadsListArgs),
}

#[derive(Args, Debug)]
pub struct ChatThreadsListArgs {
    /// Space name (e.g., spaces/AAAA)
    pub space: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

// ---------------------------------------------------------------
// DM subcommands
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ChatDmArgs {
    #[command(subcommand)]
    pub command: ChatDmCommand,
}

#[derive(Subcommand, Debug)]
pub enum ChatDmCommand {
    /// Find or create a DM space with a user
    Space(ChatDmSpaceArgs),
    /// Send a direct message
    Send(ChatDmSendArgs),
}

#[derive(Args, Debug)]
pub struct ChatDmSpaceArgs {
    /// User email or resource name
    pub user: String,
}

#[derive(Args, Debug)]
pub struct ChatDmSendArgs {
    /// Recipient email address
    pub email: String,
    /// Message text
    #[arg(long)]
    pub text: String,
    /// Thread name to reply in
    #[arg(long)]
    pub thread: Option<String>,
}
