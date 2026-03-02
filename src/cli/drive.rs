//! Drive CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Drive service commands.
#[derive(Args, Debug)]
pub struct DriveArgs {
    #[command(subcommand)]
    pub command: DriveCommand,
}

#[derive(Subcommand, Debug)]
pub enum DriveCommand {
    /// List files in a folder
    Ls(DriveLsArgs),
    /// Full-text search across Drive
    Search(DriveSearchArgs),
    /// Get file metadata
    Get(DriveGetArgs),
    /// Download a file
    Download(DriveDownloadArgs),
    /// Copy a file
    Copy(DriveCopyArgs),
    /// Upload a file
    Upload(DriveUploadArgs),
    /// Create a folder
    Mkdir(DriveMkdirArgs),
    /// Delete a file
    #[command(alias = "rm")]
    Delete(DriveDeleteArgs),
    /// Move a file
    Move(DriveMoveArgs),
    /// Rename a file
    Rename(DriveRenameArgs),
    /// Share a file
    Share(DriveShareArgs),
    /// Remove a permission
    Unshare(DriveUnshareArgs),
    /// List permissions
    Permissions(DrivePermissionsArgs),
    /// Print web URLs
    Url(DriveUrlArgs),
    /// Manage file comments
    Comments(DriveCommentsArgs),
    /// List shared drives
    Drives(DriveDrivesArgs),
}

#[derive(Args, Debug)]
pub struct DriveLsArgs {
    /// Max results
    #[arg(long, short = 'm', default_value = "20")]
    pub max: u32,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Drive query filter
    #[arg(long, short = 'q')]
    pub query: Option<String>,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
    /// Include shared drives (default: true)
    #[arg(long, default_value = "true")]
    pub all_drives: bool,
}

#[derive(Args, Debug)]
pub struct DriveSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Treat as raw Drive query language
    #[arg(long)]
    pub raw_query: bool,
    /// Max results
    #[arg(long, short = 'm', default_value = "20")]
    pub max: u32,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Include shared drives
    #[arg(long, default_value = "true")]
    pub all_drives: bool,
}

#[derive(Args, Debug)]
pub struct DriveGetArgs {
    /// File ID
    pub file_id: String,
}

#[derive(Args, Debug)]
pub struct DriveDownloadArgs {
    /// File ID
    pub file_id: String,
    /// Output path
    #[arg(long)]
    pub out: Option<String>,
    /// Export format for Google Docs files
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Args, Debug)]
pub struct DriveCopyArgs {
    /// File ID
    pub file_id: String,
    /// Name for the copy
    #[arg(long)]
    pub name: Option<String>,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct DriveUploadArgs {
    /// Local file path
    pub path: String,
    /// File name on Drive
    #[arg(long)]
    pub name: Option<String>,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
    /// Auto-convert to Google format
    #[arg(long)]
    pub convert: bool,
    /// Specific Google format to convert to
    #[arg(long)]
    pub convert_to: Option<String>,
}

#[derive(Args, Debug)]
pub struct DriveMkdirArgs {
    /// Folder name
    pub name: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct DriveDeleteArgs {
    /// File ID
    pub file_id: String,
    /// Permanently delete (bypass trash)
    #[arg(long)]
    pub permanent: bool,
}

#[derive(Args, Debug)]
pub struct DriveMoveArgs {
    /// File ID
    pub file_id: String,
    /// Destination parent folder ID
    #[arg(long)]
    pub parent: String,
}

#[derive(Args, Debug)]
pub struct DriveRenameArgs {
    /// File ID
    pub file_id: String,
    /// New name
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct DriveShareArgs {
    /// File ID
    pub file_id: String,
    /// Share target: anyone, user, domain
    #[arg(long, default_value = "anyone")]
    pub to: String,
    /// Email for user share
    #[arg(long)]
    pub email: Option<String>,
    /// Domain for domain share
    #[arg(long)]
    pub domain: Option<String>,
    /// Role: reader, writer
    #[arg(long, default_value = "reader")]
    pub role: String,
    /// Discoverable link
    #[arg(long)]
    pub discoverable: bool,
}

#[derive(Args, Debug)]
pub struct DriveUnshareArgs {
    /// File ID
    pub file_id: String,
    /// Permission ID
    pub permission_id: String,
}

#[derive(Args, Debug)]
pub struct DrivePermissionsArgs {
    /// File ID
    pub file_id: String,
    /// Max results
    #[arg(long)]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct DriveUrlArgs {
    /// File IDs
    pub file_ids: Vec<String>,
}

#[derive(Args, Debug)]
pub struct DriveCommentsArgs {
    #[command(subcommand)]
    pub command: DriveCommentsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DriveCommentsCommand {
    /// List comments
    List(DriveCommentsListArgs),
    /// Create a comment
    Create(DriveCommentsCreateArgs),
    /// Reply to a comment
    Reply(DriveCommentsReplyArgs),
}

#[derive(Args, Debug)]
pub struct DriveCommentsListArgs {
    /// File ID
    pub file_id: String,
}

#[derive(Args, Debug)]
pub struct DriveCommentsCreateArgs {
    /// File ID
    pub file_id: String,
    /// Comment content
    #[arg(long)]
    pub content: String,
}

#[derive(Args, Debug)]
pub struct DriveCommentsReplyArgs {
    /// File ID
    pub file_id: String,
    /// Comment ID
    pub comment_id: String,
    /// Reply content
    #[arg(long)]
    pub content: String,
}

#[derive(Args, Debug)]
pub struct DriveDrivesArgs {
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Query filter
    #[arg(long, short = 'q')]
    pub query: Option<String>,
}
