//! Slides CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Slides service commands.
#[derive(Args, Debug)]
pub struct SlidesArgs {
    #[command(subcommand)]
    pub command: SlidesCommand,
}

#[derive(Subcommand, Debug)]
pub enum SlidesCommand {
    /// Export presentation
    Export(SlidesExportArgs),
    /// Get presentation metadata
    Info(SlidesInfoArgs),
    /// Create new presentation
    Create(SlidesCreateArgs),
    /// Create presentation from markdown
    CreateFromMarkdown(SlidesCreateFromMarkdownArgs),
    /// Copy presentation
    Copy(SlidesCopyArgs),
    /// List all slides
    ListSlides(SlidesListSlidesArgs),
    /// Add a slide
    AddSlide(SlidesAddSlideArgs),
    /// Delete a slide
    DeleteSlide(SlidesDeleteSlideArgs),
    /// Read slide content
    ReadSlide(SlidesReadSlideArgs),
    /// Update speaker notes
    UpdateNotes(SlidesUpdateNotesArgs),
    /// Replace slide image
    ReplaceSlide(SlidesReplaceSlideArgs),
}

#[derive(Args, Debug)]
pub struct SlidesExportArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Export format
    #[arg(long, default_value = "pptx")]
    pub format: String,
    /// Output file path
    #[arg(long)]
    pub out: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesInfoArgs {
    /// Presentation ID
    pub presentation_id: String,
}

#[derive(Args, Debug)]
pub struct SlidesCreateArgs {
    /// Presentation title
    pub title: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
    /// Template presentation ID
    #[arg(long)]
    pub template: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesCreateFromMarkdownArgs {
    /// Markdown content
    #[arg(long)]
    pub content: Option<String>,
    /// File containing markdown content
    #[arg(long)]
    pub content_file: Option<String>,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
    /// Presentation title
    #[arg(long)]
    pub title: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesCopyArgs {
    /// Source presentation ID
    pub presentation_id: String,
    /// Title for the copy
    pub title: String,
    /// Parent folder ID
    #[arg(long)]
    pub parent: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesListSlidesArgs {
    /// Presentation ID
    pub presentation_id: String,
}

#[derive(Args, Debug)]
pub struct SlidesAddSlideArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Layout ID
    #[arg(long)]
    pub layout_id: Option<String>,
    /// Insertion index
    #[arg(long)]
    pub index: Option<i32>,
    /// Image URL for the slide
    #[arg(long)]
    pub image_url: Option<String>,
    /// Speaker notes
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesDeleteSlideArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Slide ID
    pub slide_id: String,
}

#[derive(Args, Debug)]
pub struct SlidesReadSlideArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Slide ID
    pub slide_id: String,
}

#[derive(Args, Debug)]
pub struct SlidesUpdateNotesArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Slide ID
    pub slide_id: String,
    /// Notes text (positional, multiple words joined)
    pub text: Vec<String>,
    /// File containing notes text
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Args, Debug)]
pub struct SlidesReplaceSlideArgs {
    /// Presentation ID
    pub presentation_id: String,
    /// Slide ID
    pub slide_id: String,
    /// Image URL to replace with
    #[arg(long)]
    pub image_url: String,
}
