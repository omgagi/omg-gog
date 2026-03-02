//! Sheets CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Sheets service commands.
#[derive(Args, Debug)]
pub struct SheetsArgs {
    #[command(subcommand)]
    pub command: SheetsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SheetsCommand {
    /// Read cell values
    Get(SheetsGetArgs),
    /// Write cell values
    Update(SheetsUpdateArgs),
    /// Append rows
    Append(SheetsAppendArgs),
    /// Insert rows or columns
    Insert(SheetsInsertArgs),
    /// Clear cell values
    Clear(SheetsClearArgs),
    /// Apply cell formatting
    Format(SheetsFormatArgs),
    /// Read cell notes
    Notes(SheetsNotesArgs),
    /// Get spreadsheet metadata
    Metadata(SheetsMetadataArgs),
    /// Create new spreadsheet
    Create(SheetsCreateArgs),
    /// Copy spreadsheet
    Copy(SheetsCopyArgs),
    /// Export spreadsheet
    Export(SheetsExportArgs),
}

#[derive(Args, Debug)]
pub struct SheetsGetArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
    /// Major dimension (ROWS or COLUMNS)
    #[arg(long)]
    pub dimension: Option<String>,
    /// Value render option
    #[arg(long)]
    pub render: Option<String>,
}

#[derive(Args, Debug)]
pub struct SheetsUpdateArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
    /// Values to write (positional)
    pub values: Vec<String>,
    /// Values as JSON array
    #[arg(long)]
    pub values_json: Option<String>,
    /// Value input option
    #[arg(long, default_value = "USER_ENTERED")]
    pub input: String,
    /// Copy data validation from this range
    #[arg(long)]
    pub copy_validation_from: Option<String>,
}

#[derive(Args, Debug)]
pub struct SheetsAppendArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
    /// Values to append (positional)
    pub values: Vec<String>,
    /// Values as JSON array
    #[arg(long)]
    pub values_json: Option<String>,
    /// Value input option
    #[arg(long, default_value = "USER_ENTERED")]
    pub input: String,
    /// Insert data option
    #[arg(long, default_value = "INSERT_ROWS")]
    pub insert: String,
    /// Copy data validation from this range
    #[arg(long)]
    pub copy_validation_from: Option<String>,
}

#[derive(Args, Debug)]
pub struct SheetsInsertArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Sheet name or ID
    pub sheet: String,
    /// Dimension to insert (rows or cols)
    pub dimension: String,
    /// Start index (0-based)
    pub start: u32,
    /// Number of rows/columns to insert
    #[arg(long, default_value = "1")]
    pub count: u32,
    /// Insert after the index
    #[arg(long)]
    pub after: bool,
}

#[derive(Args, Debug)]
pub struct SheetsClearArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
}

#[derive(Args, Debug)]
pub struct SheetsFormatArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
    /// Format as JSON object
    #[arg(long)]
    pub format_json: String,
    /// Format fields mask
    #[arg(long, default_value = "userEnteredFormat")]
    pub format_fields: String,
}

#[derive(Args, Debug)]
pub struct SheetsNotesArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Cell range (A1 notation)
    pub range: String,
}

#[derive(Args, Debug)]
pub struct SheetsMetadataArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
}

#[derive(Args, Debug)]
pub struct SheetsCreateArgs {
    /// Spreadsheet title
    pub title: String,
    /// Comma-separated sheet names
    #[arg(long)]
    pub sheets: Option<String>,
}

#[derive(Args, Debug)]
pub struct SheetsCopyArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Title for the copy
    pub title: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct SheetsExportArgs {
    /// Spreadsheet ID
    pub spreadsheet_id: String,
    /// Export format
    #[arg(long, default_value = "xlsx")]
    pub format: String,
    /// Output file path
    #[arg(long)]
    pub out: Option<String>,
}
