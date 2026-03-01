//! Docs CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Docs service commands.
#[derive(Args, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: DocsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    /// Export document as PDF/DOCX/TXT
    Export(DocsExportArgs),
    /// Get document metadata
    Info(DocsInfoArgs),
    /// Create a new Google Doc
    Create(DocsCreateArgs),
    /// Copy a document
    Copy(DocsCopyArgs),
    /// Extract plain text from document
    Cat(DocsCatArgs),
    /// List document tabs
    ListTabs(DocsListTabsArgs),
    /// Manage document comments
    Comments(DocsCommentsArgs),
    /// Write content to document
    Write(DocsWriteArgs),
    /// Insert text at position
    Insert(DocsInsertArgs),
    /// Delete text range
    Delete(DocsDeleteArgs),
    /// Find and replace text
    FindReplace(DocsFindReplaceArgs),
    /// Update document content
    Update(DocsUpdateArgs),
    /// Edit with find/replace flags
    Edit(DocsEditArgs),
    /// Sed-like regex find/replace
    Sed(DocsSedArgs),
    /// Clear all document content
    Clear(DocsClearArgs),
}

#[derive(Args, Debug)]
pub struct DocsExportArgs {
    /// Document ID
    pub doc_id: String,
    /// Export format
    #[arg(long, default_value = "pdf")]
    pub format: String,
    /// Output file path
    #[arg(long)]
    pub out: Option<String>,
}

#[derive(Args, Debug)]
pub struct DocsInfoArgs {
    /// Document ID
    pub doc_id: String,
}

#[derive(Args, Debug)]
pub struct DocsCreateArgs {
    /// Document title
    pub title: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
    /// File to import as initial content
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DocsCopyArgs {
    /// Source document ID
    pub doc_id: String,
    /// Title for the copy
    pub title: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct DocsCatArgs {
    /// Document ID
    pub doc_id: String,
    /// Maximum bytes to read
    #[arg(long, default_value = "2097152")]
    pub max_bytes: String,
    /// Specific tab to read
    #[arg(long)]
    pub tab: Option<String>,
    /// Read all tabs
    #[arg(long)]
    pub all_tabs: bool,
    /// Output raw JSON structure
    #[arg(long)]
    pub raw: bool,
}

#[derive(Args, Debug)]
pub struct DocsListTabsArgs {
    /// Document ID
    pub doc_id: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsArgs {
    #[command(subcommand)]
    pub command: DocsCommentsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommentsCommand {
    /// List comments on a document
    List(DocsCommentsListArgs),
    /// Get a specific comment
    Get(DocsCommentsGetArgs),
    /// Add a comment
    Add(DocsCommentsAddArgs),
    /// Reply to a comment
    Reply(DocsCommentsReplyArgs),
    /// Resolve a comment
    Resolve(DocsCommentsResolveArgs),
    /// Delete a comment
    Delete(DocsCommentsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct DocsCommentsListArgs {
    /// File ID
    pub file_id: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsGetArgs {
    /// File ID
    pub file_id: String,
    /// Comment ID
    pub comment_id: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsAddArgs {
    /// File ID
    pub file_id: String,
    /// Comment content
    #[arg(long)]
    pub content: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsReplyArgs {
    /// File ID
    pub file_id: String,
    /// Comment ID
    pub comment_id: String,
    /// Reply content
    #[arg(long)]
    pub content: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsResolveArgs {
    /// File ID
    pub file_id: String,
    /// Comment ID
    pub comment_id: String,
}

#[derive(Args, Debug)]
pub struct DocsCommentsDeleteArgs {
    /// File ID
    pub file_id: String,
    /// Comment ID
    pub comment_id: String,
}

#[derive(Args, Debug)]
pub struct DocsWriteArgs {
    /// Document ID
    pub doc_id: String,
    /// Content to write (positional, multiple words joined)
    pub content: Vec<String>,
    /// File to read content from
    #[arg(long)]
    pub file: Option<String>,
    /// Replace all existing content
    #[arg(long)]
    pub replace: bool,
    /// Treat input as Markdown
    #[arg(long)]
    pub markdown: bool,
}

#[derive(Args, Debug)]
pub struct DocsInsertArgs {
    /// Document ID
    pub doc_id: String,
    /// Content to insert (positional, multiple words joined)
    pub content: Vec<String>,
    /// Insertion index (1-based)
    #[arg(long, default_value = "1")]
    pub index: String,
    /// File to read content from
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DocsDeleteArgs {
    /// Document ID
    pub doc_id: String,
    /// Start index
    #[arg(long)]
    pub start: i64,
    /// End index
    #[arg(long)]
    pub end: i64,
}

#[derive(Args, Debug)]
pub struct DocsFindReplaceArgs {
    /// Document ID
    pub doc_id: String,
    /// Text to find
    pub find: String,
    /// Replacement text
    pub replace: String,
    /// Case-sensitive matching
    #[arg(long, default_value = "true")]
    pub match_case: bool,
}

#[derive(Args, Debug)]
pub struct DocsUpdateArgs {
    /// Document ID
    pub doc_id: String,
    /// Content string
    #[arg(long)]
    pub content: Option<String>,
    /// File to read content from
    #[arg(long)]
    pub content_file: Option<String>,
    /// Content format
    #[arg(long, default_value = "plain")]
    pub format: String,
    /// Append instead of replace
    #[arg(long)]
    pub append: bool,
}

#[derive(Args, Debug)]
pub struct DocsEditArgs {
    /// Document ID
    pub doc_id: String,
    /// Text to find
    #[arg(long)]
    pub find: String,
    /// Replacement text
    #[arg(long)]
    pub replace: String,
    /// Case-sensitive matching
    #[arg(long, default_value = "true")]
    pub match_case: bool,
}

#[derive(Args, Debug)]
pub struct DocsSedArgs {
    /// Document ID
    pub doc_id: String,
    /// Sed expressions (positional)
    pub expression: Vec<String>,
    /// Sed expression (can be repeated)
    #[arg(short = 'e', long = "expression")]
    pub expr_flag: Vec<String>,
    /// File containing sed expressions
    #[arg(short = 'f', long = "file")]
    pub file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DocsClearArgs {
    /// Document ID
    pub doc_id: String,
}
