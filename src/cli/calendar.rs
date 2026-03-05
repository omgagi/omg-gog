//! Calendar CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Calendar service commands.
#[derive(Args, Debug)]
pub struct CalendarArgs {
    #[command(subcommand)]
    pub command: CalendarCommand,
}

#[derive(Subcommand, Debug)]
pub enum CalendarCommand {
    /// List calendars
    Calendars(CalendarCalendarsArgs),
    /// List calendar ACL
    Acl(CalendarAclArgs),
    /// List events
    #[command(alias = "ls")]
    Events(CalendarEventsArgs),
    /// Get event details
    Event(CalendarEventArgs),
    /// Create an event
    Create(CalendarCreateArgs),
    /// Update an event
    Update(CalendarUpdateArgs),
    /// Delete an event
    Delete(CalendarDeleteArgs),
    /// Get free/busy information
    Freebusy(CalendarFreeBusyArgs),
    /// Respond to an event invitation
    #[command(alias = "rsvp")]
    Respond(CalendarRespondArgs),
    /// Search events across calendars
    Search(CalendarSearchArgs),
    /// Show server time
    Time,
    /// List workspace users
    Users,
    /// Show events for a Google Group
    Team(CalendarTeamArgs),
    /// Show calendar colors
    Colors,
    /// Find scheduling conflicts
    Conflicts(CalendarConflictsArgs),
    /// Create Focus Time block
    #[command(name = "focus-time")]
    FocusTime(CalendarFocusTimeArgs),
    /// Create Out of Office event
    #[command(name = "out-of-office", alias = "ooo")]
    OutOfOffice(CalendarOooArgs),
    /// Set working location
    #[command(name = "working-location", alias = "wl")]
    WorkingLocation(CalendarWorkingLocationArgs),
    /// Generate propose-time URL
    #[command(name = "propose-time")]
    ProposeTime(CalendarProposeTimeArgs),
    /// Manage push notification watches
    Watch(CalendarWatchArgs),
}

#[derive(Args, Debug)]
pub struct CalendarCalendarsArgs {
    #[arg(long, short = 'm', default_value = "100")]
    pub max: u32,
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub fail_empty: bool,
}

#[derive(Args, Debug)]
pub struct CalendarAclArgs {
    /// Calendar ID
    pub calendar_id: String,
}

#[derive(Args, Debug)]
pub struct CalendarEventsArgs {
    /// Calendar ID or name
    #[arg(long)]
    pub cal: Option<String>,
    /// Multiple calendar IDs (CSV)
    #[arg(long)]
    pub calendars: Option<String>,
    /// Events from all calendars
    #[arg(long)]
    pub all: bool,
    /// Start time/date
    #[arg(long)]
    pub from: Option<String>,
    /// End time/date
    #[arg(long)]
    pub to: Option<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
    /// Free text query
    #[arg(long, short = 'q')]
    pub query: Option<String>,
    /// Show day of week
    #[arg(long)]
    pub weekday: bool,
    /// Calendar API fields parameter
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalendarEventArgs {
    /// Calendar ID
    pub calendar_id: String,
    /// Event ID
    pub event_id: String,
}

#[derive(Args, Debug)]
pub struct CalendarCreateArgs {
    /// Event summary
    #[arg(long)]
    pub summary: String,
    /// Start time
    #[arg(long)]
    pub from: String,
    /// End time
    #[arg(long)]
    pub to: String,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Location
    #[arg(long)]
    pub location: Option<String>,
    /// Attendees (comma-separated)
    #[arg(long)]
    pub attendees: Option<String>,
    /// All-day event
    #[arg(long)]
    pub all_day: bool,
    /// Calendar ID
    #[arg(long, default_value = "primary")]
    pub cal: String,
    /// Event type
    #[arg(long)]
    pub event_type: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalendarUpdateArgs {
    /// Calendar ID
    pub calendar_id: String,
    /// Event ID
    pub event_id: String,
    /// Summary
    #[arg(long)]
    pub summary: Option<String>,
    /// Start time
    #[arg(long)]
    pub from: Option<String>,
    /// End time
    #[arg(long)]
    pub to: Option<String>,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Location
    #[arg(long)]
    pub location: Option<String>,
    /// Add attendee
    #[arg(long)]
    pub add_attendee: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CalendarDeleteArgs {
    /// Calendar ID
    pub calendar_id: String,
    /// Event ID
    pub event_id: String,
}

#[derive(Args, Debug)]
pub struct CalendarFreeBusyArgs {
    /// Calendar IDs (comma-separated)
    pub calendar_ids: String,
    /// Start time
    #[arg(long)]
    pub from: String,
    /// End time
    #[arg(long)]
    pub to: String,
}

#[derive(Args, Debug)]
pub struct CalendarRespondArgs {
    /// Calendar ID
    pub calendar_id: String,
    /// Event ID
    pub event_id: String,
    /// Response status: accepted, declined, tentative
    #[arg(long)]
    pub status: String,
    /// Send updates: all, none, externalOnly
    #[arg(long)]
    pub send_updates: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalendarSearchArgs {
    /// Search query
    pub query: Vec<String>,
    /// Calendar ID
    #[arg(long)]
    pub cal: Option<String>,
    /// Multiple calendars
    #[arg(long)]
    pub calendars: Option<String>,
    /// Search all calendars
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct CalendarTeamArgs {
    /// Group email
    pub group_email: String,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalendarConflictsArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub cal: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalendarFocusTimeArgs {
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long, default_value = "primary")]
    pub cal: String,
}

#[derive(Args, Debug)]
pub struct CalendarOooArgs {
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long, default_value = "primary")]
    pub cal: String,
}

#[derive(Args, Debug)]
pub struct CalendarWorkingLocationArgs {
    /// Location type: home, office, custom
    pub location_type: String,
    /// Custom label
    #[arg(long)]
    pub label: Option<String>,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long, default_value = "primary")]
    pub cal: String,
}

#[derive(Args, Debug)]
pub struct CalendarProposeTimeArgs {
    /// Calendar ID
    pub calendar_id: String,
    /// Event ID
    pub event_id: String,
}

#[derive(Args, Debug)]
pub struct CalendarWatchArgs {
    #[command(subcommand)]
    pub command: CalendarWatchCommand,
}

#[derive(Subcommand, Debug)]
pub enum CalendarWatchCommand {
    /// Start watching calendar events
    Start(CalendarWatchStartArgs),
    /// Stop watching
    Stop(CalendarWatchStopArgs),
    /// Show watch status
    Status,
}

#[derive(Args, Debug)]
pub struct CalendarWatchStartArgs {
    /// HTTPS callback URL for push notifications
    #[arg(long)]
    pub callback_url: String,
    /// Calendar ID (default: primary)
    #[arg(long, default_value = "primary")]
    pub calendar: String,
}

#[derive(Args, Debug)]
pub struct CalendarWatchStopArgs {
    /// Channel ID (from watch start)
    #[arg(long)]
    pub channel_id: String,
    /// Resource ID (from watch start)
    #[arg(long)]
    pub resource_id: String,
}
