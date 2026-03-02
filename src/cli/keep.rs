//! Keep CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Keep service commands.
#[derive(Args, Debug)]
pub struct KeepArgs {
    /// Service account key file
    #[arg(long)]
    pub service_account: Option<String>,

    /// Email to impersonate
    #[arg(long)]
    pub impersonate: Option<String>,

    #[command(subcommand)]
    pub command: KeepCommand,
}

#[derive(Subcommand, Debug)]
pub enum KeepCommand {
    /// List notes
    List(KeepListArgs),
    /// Get a note
    Get(KeepGetArgs),
    /// Search notes
    Search(KeepSearchArgs),
    /// Download an attachment
    Attachment(KeepAttachmentArgs),
}

#[derive(Args, Debug)]
pub struct KeepListArgs {
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
    /// Filter expression
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Args, Debug)]
pub struct KeepGetArgs {
    /// Note ID or resource name
    pub note_id: String,
}

#[derive(Args, Debug)]
pub struct KeepSearchArgs {
    /// Search query
    pub query: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
}

#[derive(Args, Debug)]
pub struct KeepAttachmentArgs {
    /// Attachment resource name
    pub attachment_name: String,
    /// MIME type filter
    #[arg(long)]
    pub mime_type: Option<String>,
    /// Output file path
    #[arg(long)]
    pub out: Option<String>,
}
