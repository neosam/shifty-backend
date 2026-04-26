//! Atom-level UI components for the shifty redesign.
//!
//! This module groups the smallest reusable building blocks used across
//! redesigned pages. Each atom is implemented in its own file and re-exported
//! here so consumers can write `use crate::component::atoms::Btn;` (or
//! `crate::component::Btn` via the parent module re-export).
//!
//! Atoms:
//! - [`Btn`] / [`BtnVariant`] — primary, secondary, ghost and danger buttons
//!   used for forms, dialogs and inline actions.
//! - [`PersonChip`] — pastel-background name pill with the dark-text
//!   invariant (see `input.css` `.person-pill` rule).
//! - [`TupleRow`] — label/value row with an optional description, used in
//!   detail panels and summaries.
//! - [`NavBtn`] — 28×28 mono-glyph square button used for prev/next
//!   navigation in week and year contexts.
//!
//! All atoms consume the design tokens introduced in the
//! `design-tokens` capability — none of them carry hardcoded colors or
//! radii.

pub mod btn;
pub mod media_query;
pub mod nav_btn;
pub mod person_chip;
pub mod tuple_row;

pub use btn::{Btn, BtnVariant};
pub use media_query::use_media_query;
pub use nav_btn::NavBtn;
pub use person_chip::PersonChip;
pub use tuple_row::TupleRow;
