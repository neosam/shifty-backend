//! `PersonChip` — name pill used wherever an employee is referenced.
//!
//! ## Color invariant
//!
//! When a pastel `color` is supplied, the chip carries the global
//! `.person-pill` CSS class which forces `color: var(--chip-ink)` via an
//! intentional `!important` rule (see `input.css`). The dark-text color is
//! identical in light and dark themes because the background is always a
//! light pastel — flipping the text color in dark mode would destroy
//! contrast.
//!
//! When `color` is `None`, the chip falls back to a transparent body with
//! a dashed `--border-strong` border and `--ink-soft` text. This path
//! deliberately does **not** carry the `.person-pill` class so the dashed
//! variant can adopt the active theme's ink-soft color.
//!
//! ## No initials
//!
//! The chip renders only the name text. There is no avatar circle, no
//! abbreviation, no two-letter initials. This is enforced at the
//! component level (the only text node is `{props.name}`).

use dioxus::prelude::*;

use crate::base_types::ImStr;

const SHAPE_CLASSES: &str =
    "inline-flex px-[4px] pl-[7px] py-px rounded-sm text-body font-medium whitespace-nowrap";

/// Builds the class string for a `PersonChip`.
///
/// Splits across the two visual paths (with/without color) and adds the
/// optional `font-semibold` for the bold highlight case. Returned as a
/// space-separated string, ready to drop into `class:`.
pub(crate) fn build_class(has_color: bool, bold: bool, clickable: bool) -> String {
    let mut out = String::with_capacity(160);
    out.push_str(SHAPE_CLASSES);
    if has_color {
        out.push(' ');
        out.push_str("person-pill");
    } else {
        out.push(' ');
        out.push_str("border border-dashed border-border-strong text-ink-soft bg-transparent");
    }
    if bold {
        out.push(' ');
        out.push_str("font-semibold");
    }
    if clickable {
        out.push(' ');
        out.push_str("cursor-pointer");
    }
    out
}

#[derive(Props, Clone, PartialEq)]
pub struct PersonChipProps {
    pub name: ImStr,

    /// Pastel hex color (e.g. `"#dbe0ff"`). When `None`, the chip renders
    /// in the dashed-border, no-color fallback variant.
    #[props(!optional, default = None)]
    pub color: Option<ImStr>,

    /// Highlight the chip with `font-semibold` (used to mark the
    /// currently-edited person in the week view).
    #[props(default = false)]
    pub bold: bool,

    #[props(!optional, default = None)]
    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn PersonChip(props: PersonChipProps) -> Element {
    let has_color = props.color.is_some();
    let clickable = props.on_click.is_some();
    let class = build_class(has_color, props.bold, clickable);
    let inline_style = props
        .color
        .as_ref()
        .map(|c| format!("background-color: {}", c))
        .unwrap_or_default();
    let on_click = props.on_click.clone();

    rsx! {
        span {
            class: "{class}",
            style: "{inline_style}",
            onclick: move |evt| {
                if let Some(handler) = &on_click {
                    evt.prevent_default();
                    handler.call(());
                }
            },
            "{props.name}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_color_carries_person_pill_class() {
        let c = build_class(true, false, false);
        assert!(c.contains("person-pill"), "missing person-pill: {c}");
    }

    #[test]
    fn with_color_does_not_carry_dashed_border() {
        let c = build_class(true, false, false);
        assert!(
            !c.contains("border-dashed"),
            "with-color path should not have dashed border: {c}"
        );
    }

    #[test]
    fn without_color_uses_dashed_border_and_inksoft() {
        let c = build_class(false, false, false);
        assert!(c.contains("border-dashed"), "missing border-dashed: {c}");
        assert!(
            c.contains("border-border-strong"),
            "missing border-border-strong: {c}"
        );
        assert!(c.contains("text-ink-soft"), "missing text-ink-soft: {c}");
        assert!(c.contains("bg-transparent"), "missing bg-transparent: {c}");
    }

    #[test]
    fn without_color_does_not_carry_person_pill_class() {
        let c = build_class(false, false, false);
        assert!(
            !c.contains("person-pill"),
            "no-color path must NOT carry person-pill (would force dark ink): {c}"
        );
    }

    #[test]
    fn bold_adds_font_semibold() {
        let c = build_class(true, true, false);
        assert!(c.contains("font-semibold"), "missing font-semibold: {c}");
    }

    #[test]
    fn not_bold_omits_font_semibold() {
        let c = build_class(true, false, false);
        assert!(!c.contains("font-semibold"));
    }

    #[test]
    fn clickable_adds_cursor_pointer() {
        let c = build_class(true, false, true);
        assert!(c.contains("cursor-pointer"), "missing cursor-pointer: {c}");
    }

    #[test]
    fn non_clickable_omits_cursor_pointer() {
        let c = build_class(true, false, false);
        assert!(!c.contains("cursor-pointer"));
    }

    #[test]
    fn shape_classes_present_in_both_paths() {
        for has_color in [true, false] {
            let c = build_class(has_color, false, false);
            assert!(c.contains("inline-flex"));
            assert!(c.contains("rounded-sm"));
            assert!(c.contains("text-body"));
            assert!(c.contains("font-medium"));
        }
    }

    #[test]
    fn with_color_carries_whitespace_nowrap() {
        let c = build_class(true, false, false);
        assert!(
            c.contains("whitespace-nowrap"),
            "missing whitespace-nowrap (color path): {c}"
        );
    }

    #[test]
    fn without_color_carries_whitespace_nowrap() {
        let c = build_class(false, false, false);
        assert!(
            c.contains("whitespace-nowrap"),
            "missing whitespace-nowrap (no-color path): {c}"
        );
    }

    #[test]
    fn bold_variant_carries_whitespace_nowrap_and_font_semibold() {
        let c = build_class(true, true, false);
        assert!(
            c.contains("whitespace-nowrap"),
            "missing whitespace-nowrap (bold variant): {c}"
        );
        assert!(
            c.contains("font-semibold"),
            "missing font-semibold (bold variant): {c}"
        );
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn with_color_renders_inline_background() {
        fn app() -> Element {
            rsx! {
                PersonChip {
                    name: ImStr::from("Alex"),
                    color: Some(ImStr::from("#dbe0ff")),
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("background-color: #dbe0ff"),
            "expected inline color: {html}"
        );
        assert!(
            html.contains("person-pill"),
            "missing person-pill class: {html}"
        );
        assert!(html.contains("Alex"));
    }

    #[test]
    fn without_color_does_not_emit_person_pill_in_html() {
        fn app() -> Element {
            rsx! { PersonChip { name: ImStr::from("Bo") } }
        }
        let html = render(app);
        assert!(
            !html.contains("person-pill"),
            "no-color path leaked person-pill class: {html}"
        );
        assert!(
            html.contains("border-dashed"),
            "missing dashed border: {html}"
        );
        assert!(html.contains("Bo"));
    }

    #[test]
    fn renders_only_name_no_initials() {
        fn app() -> Element {
            rsx! {
                PersonChip {
                    name: ImStr::from("Charlie Brown"),
                    color: Some(ImStr::from("#ffeedd")),
                }
            }
        }
        let html = render(app);
        // Only the full name appears; no two-letter initials and no avatar circle.
        assert!(html.contains("Charlie Brown"));
        // The HTML must not contain a stray "CB" or other initials abbreviation.
        // (We allow "CB" inside larger words; check for a standalone initials pattern.)
        assert!(
            !html.contains(">CB<"),
            "initials should not be rendered: {html}"
        );
    }

    #[test]
    fn bold_variant_emits_font_semibold_in_html() {
        fn app() -> Element {
            rsx! {
                PersonChip {
                    name: ImStr::from("D"),
                    color: Some(ImStr::from("#abcdef")),
                    bold: true,
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("font-semibold"),
            "missing font-semibold: {html}"
        );
    }
}
