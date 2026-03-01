//! Gmail CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Gmail service commands.
#[derive(Args, Debug)]
pub struct GmailArgs {
    #[command(subcommand)]
    pub command: GmailCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailCommand {
    /// Search threads using Gmail query syntax
    Search(GmailSearchArgs),
    /// Message operations
    Messages(GmailMessagesArgs),
    /// Thread operations (get, modify)
    Thread(GmailThreadArgs),
    /// Get a message
    Get(GmailGetArgs),
    /// Download a single attachment
    Attachment(GmailAttachmentArgs),
    /// Print Gmail web URLs for threads
    Url(GmailUrlArgs),
    /// Gmail history
    History(GmailHistoryArgs),
    /// Label operations
    Labels(GmailLabelsArgs),
    /// Batch operations
    Batch(GmailBatchArgs),
    /// Send an email
    Send(GmailSendArgs),
    /// Draft operations
    Drafts(GmailDraftsArgs),
    /// Settings and admin
    Settings(GmailSettingsArgs),
    /// Manage Gmail watch
    Watch(GmailWatchArgs),
}

#[derive(Args, Debug)]
pub struct GmailSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Max results
    #[arg(long, short = 'm', default_value = "10")]
    pub max: u32,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Fetch all pages
    #[arg(long)]
    pub all: bool,
    /// Exit with code 3 if no results
    #[arg(long)]
    pub fail_empty: bool,
    /// Show first message date instead of last
    #[arg(long)]
    pub oldest: bool,
    /// Output timezone
    #[arg(long, short = 'z')]
    pub timezone: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailMessagesArgs {
    #[command(subcommand)]
    pub command: GmailMessagesCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailMessagesCommand {
    /// Search messages
    Search(GmailMessagesSearchArgs),
}

#[derive(Args, Debug)]
pub struct GmailMessagesSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Include message body
    #[arg(long)]
    pub include_body: bool,
    /// Max results
    #[arg(long, short = 'm', default_value = "10")]
    pub max: u32,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailThreadArgs {
    #[command(subcommand)]
    pub command: GmailThreadCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailThreadCommand {
    /// Get thread details
    Get(GmailThreadGetArgs),
    /// Modify thread labels
    Modify(GmailThreadModifyArgs),
    /// Download thread attachments
    Attachments(GmailThreadAttachmentsArgs),
}

#[derive(Args, Debug)]
pub struct GmailThreadGetArgs {
    /// Thread ID
    pub thread_id: String,
    /// Download all attachments
    #[arg(long)]
    pub download: bool,
}

#[derive(Args, Debug)]
pub struct GmailThreadModifyArgs {
    /// Thread ID
    pub thread_id: String,
    /// Label IDs to add
    #[arg(long)]
    pub add: Vec<String>,
    /// Label IDs to remove
    #[arg(long)]
    pub remove: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailThreadAttachmentsArgs {
    /// Thread ID
    pub thread_id: String,
    /// Output directory
    #[arg(long)]
    pub out_dir: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailGetArgs {
    /// Message ID
    pub message_id: String,
    /// Format: full, metadata, raw
    #[arg(long, default_value = "full")]
    pub format: String,
    /// Specific headers to include
    #[arg(long)]
    pub headers: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailAttachmentArgs {
    /// Message ID
    pub message_id: String,
    /// Attachment ID
    pub attachment_id: String,
    /// Output file path
    #[arg(long)]
    pub out: Option<String>,
    /// Output filename
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailUrlArgs {
    /// Thread IDs
    pub thread_ids: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailHistoryArgs {
    /// Start history ID
    #[arg(long)]
    pub since: String,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailLabelsArgs {
    #[command(subcommand)]
    pub command: GmailLabelsCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailLabelsCommand {
    /// List all labels
    List,
    /// Get a label by ID or name
    Get(GmailLabelsGetArgs),
    /// Create a new label
    Create(GmailLabelsCreateArgs),
    /// Batch modify messages by label
    Modify(GmailLabelsModifyArgs),
    /// Delete a label
    Delete(GmailLabelsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct GmailLabelsGetArgs {
    /// Label ID or name
    pub label: String,
}

#[derive(Args, Debug)]
pub struct GmailLabelsCreateArgs {
    /// Label name
    pub name: String,
}

#[derive(Args, Debug)]
pub struct GmailLabelsModifyArgs {
    /// Thread IDs
    pub thread_ids: Vec<String>,
    /// Labels to add
    #[arg(long)]
    pub add: Vec<String>,
    /// Labels to remove
    #[arg(long)]
    pub remove: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailLabelsDeleteArgs {
    /// Label ID
    pub label_id: String,
}

#[derive(Args, Debug)]
pub struct GmailBatchArgs {
    #[command(subcommand)]
    pub command: GmailBatchCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailBatchCommand {
    /// Batch delete messages
    Delete(GmailBatchDeleteArgs),
    /// Batch modify message labels
    Modify(GmailBatchModifyArgs),
}

#[derive(Args, Debug)]
pub struct GmailBatchDeleteArgs {
    /// Message IDs
    pub message_ids: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailBatchModifyArgs {
    /// Message IDs
    pub message_ids: Vec<String>,
    /// Labels to add
    #[arg(long)]
    pub add: Vec<String>,
    /// Labels to remove
    #[arg(long)]
    pub remove: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GmailSendArgs {
    /// Recipient email(s)
    #[arg(long)]
    pub to: Vec<String>,
    /// Email subject
    #[arg(long)]
    pub subject: Option<String>,
    /// Plain text body
    #[arg(long)]
    pub body: Option<String>,
    /// HTML body
    #[arg(long)]
    pub body_html: Option<String>,
    /// CC recipients
    #[arg(long)]
    pub cc: Vec<String>,
    /// BCC recipients
    #[arg(long)]
    pub bcc: Vec<String>,
    /// Reply-To address
    #[arg(long)]
    pub reply_to: Option<String>,
    /// In-Reply-To message ID for threading
    #[arg(long)]
    pub reply_to_message_id: Option<String>,
    /// Attachments
    #[arg(long)]
    pub attach: Vec<String>,
    /// Enable open tracking
    #[arg(long)]
    pub track: bool,
}

#[derive(Args, Debug)]
pub struct GmailDraftsArgs {
    #[command(subcommand)]
    pub command: GmailDraftsCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailDraftsCommand {
    /// List drafts
    List,
    /// Get a draft
    Get(GmailDraftsGetArgs),
    /// Create a draft
    Create(GmailDraftsCreateArgs),
    /// Update a draft
    Update(GmailDraftsUpdateArgs),
    /// Send a draft
    Send(GmailDraftsSendArgs),
    /// Delete a draft
    Delete(GmailDraftsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct GmailDraftsGetArgs {
    /// Draft ID
    pub draft_id: String,
}

#[derive(Args, Debug)]
pub struct GmailDraftsCreateArgs {
    /// Recipient
    #[arg(long)]
    pub to: Vec<String>,
    /// Subject
    #[arg(long)]
    pub subject: Option<String>,
    /// Body
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailDraftsUpdateArgs {
    /// Draft ID to update
    pub draft_id: String,
    /// Recipient
    #[arg(long)]
    pub to: Vec<String>,
    /// Subject
    #[arg(long)]
    pub subject: Option<String>,
    /// Body
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailDraftsSendArgs {
    /// Draft ID to send
    pub draft_id: String,
}

#[derive(Args, Debug)]
pub struct GmailDraftsDeleteArgs {
    /// Draft ID to delete
    pub draft_id: String,
}

#[derive(Args, Debug)]
pub struct GmailSettingsArgs {
    #[command(subcommand)]
    pub command: GmailSettingsCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailSettingsCommand {
    /// Filter operations
    Filters(GmailFiltersArgs),
    /// Forwarding addresses
    Forwarding(GmailForwardingArgs),
    /// Send-as settings
    Sendas(GmailSendAsArgs),
    /// Delegate operations
    Delegates(GmailDelegatesArgs),
    /// Vacation responder
    Vacation(GmailVacationArgs),
    /// Auto-forwarding settings
    Autoforward(GmailAutoForwardArgs),
}

#[derive(Args, Debug)]
pub struct GmailFiltersArgs {
    #[command(subcommand)]
    pub command: GmailFiltersCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailFiltersCommand {
    List,
    Get(GmailFiltersGetArgs),
    Create(GmailFiltersCreateArgs),
    Delete(GmailFiltersDeleteArgs),
}

#[derive(Args, Debug)]
pub struct GmailFiltersGetArgs {
    pub filter_id: String,
}

#[derive(Args, Debug)]
pub struct GmailFiltersCreateArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailFiltersDeleteArgs {
    pub filter_id: String,
}

#[derive(Args, Debug)]
pub struct GmailForwardingArgs {
    #[command(subcommand)]
    pub command: GmailForwardingCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailForwardingCommand {
    List,
    Get(GmailForwardingGetArgs),
    Create(GmailForwardingCreateArgs),
    Delete(GmailForwardingDeleteArgs),
}

#[derive(Args, Debug)]
pub struct GmailForwardingGetArgs {
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailForwardingCreateArgs {
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailForwardingDeleteArgs {
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailSendAsArgs {
    #[command(subcommand)]
    pub command: GmailSendAsCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailSendAsCommand {
    /// List send-as aliases
    List,
    /// Get a send-as alias
    Get(GmailSendAsGetArgs),
    /// Create a send-as alias
    Create(GmailSendAsCreateArgs),
    /// Verify a send-as alias
    Verify(GmailSendAsVerifyArgs),
    /// Delete a send-as alias
    Delete(GmailSendAsDeleteArgs),
    /// Update a send-as alias
    Update(GmailSendAsUpdateArgs),
}

#[derive(Args, Debug)]
pub struct GmailSendAsGetArgs {
    /// Send-as email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailSendAsCreateArgs {
    /// Send-as email address
    pub email: String,
    /// Display name
    #[arg(long)]
    pub display_name: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailSendAsVerifyArgs {
    /// Send-as email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailSendAsDeleteArgs {
    /// Send-as email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailSendAsUpdateArgs {
    /// Send-as email address
    pub email: String,
    /// Display name
    #[arg(long)]
    pub display_name: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailDelegatesArgs {
    #[command(subcommand)]
    pub command: GmailDelegatesCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailDelegatesCommand {
    /// List delegates
    List,
    /// Get a delegate
    Get(GmailDelegatesGetArgs),
    /// Add a delegate
    Add(GmailDelegatesAddArgs),
    /// Remove a delegate
    Remove(GmailDelegatesRemoveArgs),
}

#[derive(Args, Debug)]
pub struct GmailDelegatesGetArgs {
    /// Delegate email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailDelegatesAddArgs {
    /// Delegate email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailDelegatesRemoveArgs {
    /// Delegate email address
    pub email: String,
}

#[derive(Args, Debug)]
pub struct GmailVacationArgs {
    #[command(subcommand)]
    pub command: GmailVacationCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailVacationCommand {
    Get,
    Update(GmailVacationUpdateArgs),
}

#[derive(Args, Debug)]
pub struct GmailVacationUpdateArgs {
    /// Enable/disable vacation responder
    #[arg(long)]
    pub enable: Option<bool>,
    /// Response subject
    #[arg(long)]
    pub subject: Option<String>,
    /// Response body
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailAutoForwardArgs {
    #[command(subcommand)]
    pub command: GmailAutoForwardCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailAutoForwardCommand {
    /// Get auto-forwarding settings
    Get,
    /// Update auto-forwarding settings
    Update(GmailAutoForwardUpdateArgs),
}

#[derive(Args, Debug)]
pub struct GmailAutoForwardUpdateArgs {
    /// Enable/disable auto-forwarding
    #[arg(long)]
    pub enable: Option<bool>,
    /// Forwarding email address
    #[arg(long)]
    pub email: Option<String>,
    /// Disposition (leaveInInbox, archive, trash, markRead)
    #[arg(long)]
    pub disposition: Option<String>,
}

#[derive(Args, Debug)]
pub struct GmailWatchArgs {
    #[command(subcommand)]
    pub command: GmailWatchCommand,
}

#[derive(Subcommand, Debug)]
pub enum GmailWatchCommand {
    /// Start watching
    Start(GmailWatchStartArgs),
    /// Show watch status
    Status,
    /// Renew watch
    Renew,
    /// Stop watching
    Stop,
}

#[derive(Args, Debug)]
pub struct GmailWatchStartArgs {
    /// Pub/Sub topic name
    #[arg(long)]
    pub topic: String,
    /// Label IDs to filter
    #[arg(long)]
    pub label: Vec<String>,
}
