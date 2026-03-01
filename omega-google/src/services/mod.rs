pub mod common;
pub mod gmail;
pub mod calendar;
pub mod drive;
pub mod docs;
pub mod slides;
pub mod sheets;
pub mod forms;
pub mod chat;
pub mod tasks;
pub mod classroom;
pub mod contacts;
pub mod people;

use crate::output::{OutputMode, JsonTransform};
use crate::ui::Ui;
use crate::cli::root::RootFlags;

/// Shared context passed to all service handlers.
pub struct ServiceContext {
    pub client: reqwest::Client,
    pub output_mode: OutputMode,
    pub json_transform: JsonTransform,
    pub ui: Ui,
    pub flags: RootFlags,
}

impl ServiceContext {
    /// Write output in the appropriate format.
    pub fn write_output<T: serde::Serialize>(
        &self,
        value: &T,
    ) -> anyhow::Result<()> {
        match self.output_mode {
            OutputMode::Json => {
                crate::output::write_json(&mut std::io::stdout(), value, &self.json_transform)
            }
            OutputMode::Plain => {
                // For plain output, serialize to JSON and write
                let json_str = serde_json::to_string(value)?;
                println!("{}", json_str);
                Ok(())
            }
            OutputMode::Text => {
                // For text output, serialize to pretty JSON as a default
                let json_str = serde_json::to_string_pretty(value)?;
                println!("{}", json_str);
                Ok(())
            }
        }
    }

    /// Write paginated output with nextPageToken hint on stderr.
    pub fn write_paginated<T: serde::Serialize>(
        &self,
        value: &T,
        next_page_token: Option<&str>,
    ) -> anyhow::Result<()> {
        self.write_output(value)?;
        if let Some(token) = next_page_token {
            self.ui.hint(&format!("Next page: --page {}", token));
        }
        Ok(())
    }

    /// Check if this is a dry-run.
    pub fn is_dry_run(&self) -> bool {
        self.flags.dry_run
    }

    /// Check if force mode is enabled.
    pub fn is_force(&self) -> bool {
        self.flags.force
    }

    /// Get the account identifier.
    pub fn account(&self) -> Option<&str> {
        self.flags.account.as_deref()
    }
}
