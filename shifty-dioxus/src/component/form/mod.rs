//! Form atoms for the redesigned dialogs.
//!
//! Each atom uses the design tokens defined in `input.css` and the
//! `Form*` prefix to coexist with the legacy form components in
//! [`crate::component::base_components`] (`TextInput`, `Select`, etc.) until
//! the per-page migrations in changes 05–09 are complete. After full
//! migration, a cleanup change drops the `Form*` prefix and removes the
//! legacy components — see `openspec/changes/REDESIGN_PLAN.md`.

pub mod checkbox;
pub mod field;
pub mod inputs;

pub use checkbox::FormCheckbox;
pub use field::Field;
pub use inputs::{FormSelectInput, FormTextInput, FormTextareaInput};
