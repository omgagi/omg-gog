// Argument rewriting, alias dispatch
//
// The primary desire path rewriting logic lives in cli/mod.rs:
//   - rewrite_desire_path_args()
//   - rewrite_command_aliases()
//   - is_calendar_events_command()
//   - global_flag_takes_value()
//
// This module re-exports those functions for architectural clarity.

pub use super::global_flag_takes_value;
pub use super::is_calendar_events_command;
pub use super::rewrite_command_aliases;
pub use super::rewrite_desire_path_args;
