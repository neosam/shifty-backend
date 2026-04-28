//! `Btn` — the primary action button used across the redesigned pages.

use dioxus::prelude::*;

use crate::base_types::ImStr;

/// Visual variant of [`Btn`].
///
/// Variants map to design tokens defined in `input.css`:
/// - [`Primary`](BtnVariant::Primary): accent on accent-ink (call-to-action)
/// - [`Secondary`](BtnVariant::Secondary): surface on ink with strong border (default)
/// - [`Ghost`](BtnVariant::Ghost): transparent with ink-soft text (low emphasis)
/// - [`Danger`](BtnVariant::Danger): bad text on surface, bad border (destructive)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BtnVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

impl Default for BtnVariant {
    fn default() -> Self {
        BtnVariant::Secondary
    }
}

const BASE_CLASSES: &str = "px-3 py-1.5 rounded-md text-body font-medium border";

/// Returns the variant-specific portion of the button class string.
pub(crate) fn variant_classes(variant: BtnVariant) -> &'static str {
    match variant {
        BtnVariant::Primary => "bg-accent text-accent-ink border-accent",
        BtnVariant::Secondary => "bg-surface text-ink border-border-strong",
        BtnVariant::Ghost => "bg-transparent text-ink-soft border-transparent",
        BtnVariant::Danger => "bg-surface text-bad border-bad",
    }
}

/// Returns the disabled-state classes (or empty when not disabled).
pub(crate) fn disabled_classes(disabled: bool) -> &'static str {
    if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    }
}

/// Builds the full class string for a `Btn` with the given variant and state.
pub(crate) fn build_class(variant: BtnVariant, disabled: bool) -> String {
    let mut out = String::with_capacity(96);
    out.push_str(BASE_CLASSES);
    out.push(' ');
    out.push_str(variant_classes(variant));
    let dc = disabled_classes(disabled);
    if !dc.is_empty() {
        out.push(' ');
        out.push_str(dc);
    }
    out
}

#[derive(Props, Clone, PartialEq)]
pub struct BtnProps {
    pub children: Element,

    #[props(default = BtnVariant::Secondary)]
    pub variant: BtnVariant,

    #[props(default = false)]
    pub disabled: bool,

    /// Optional mono-font glyph rendered before `children`.
    #[props(!optional, default = None)]
    pub icon: Option<ImStr>,

    #[props(!optional, default = None)]
    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn Btn(props: BtnProps) -> Element {
    let class = build_class(props.variant, props.disabled);
    let disabled = props.disabled;
    let on_click = props.on_click.clone();

    rsx! {
        button {
            class: "{class}",
            disabled,
            onclick: move |evt| {
                evt.prevent_default();
                if disabled {
                    return;
                }
                if let Some(handler) = &on_click {
                    handler.call(());
                }
            },
            if let Some(icon) = props.icon.as_ref() {
                span { class: "font-mono mr-1", "{icon}" }
            }
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_classes_primary_has_accent_tokens() {
        let s = variant_classes(BtnVariant::Primary);
        assert!(s.contains("bg-accent"), "primary missing bg-accent: {s}");
        assert!(
            s.contains("text-accent-ink"),
            "primary missing text-accent-ink: {s}"
        );
    }

    #[test]
    fn variant_classes_secondary_has_surface_ink_strong_border() {
        let s = variant_classes(BtnVariant::Secondary);
        assert!(
            s.contains("bg-surface"),
            "secondary missing bg-surface: {s}"
        );
        assert!(s.contains("text-ink"), "secondary missing text-ink: {s}");
        assert!(
            s.contains("border-border-strong"),
            "secondary missing border-border-strong: {s}"
        );
    }

    #[test]
    fn variant_classes_ghost_is_transparent_inksoft() {
        let s = variant_classes(BtnVariant::Ghost);
        assert!(s.contains("bg-transparent"), "ghost not transparent: {s}");
        assert!(
            s.contains("text-ink-soft"),
            "ghost missing text-ink-soft: {s}"
        );
    }

    #[test]
    fn variant_classes_danger_uses_bad_tokens() {
        let s = variant_classes(BtnVariant::Danger);
        assert!(s.contains("text-bad"), "danger missing text-bad: {s}");
        assert!(s.contains("border-bad"), "danger missing border-bad: {s}");
        assert!(s.contains("bg-surface"), "danger missing bg-surface: {s}");
    }

    #[test]
    fn build_class_includes_base_classes() {
        let c = build_class(BtnVariant::Secondary, false);
        assert!(c.contains("px-3"));
        assert!(c.contains("py-1.5"));
        assert!(c.contains("rounded-md"));
        assert!(c.contains("text-body"));
        assert!(c.contains("font-medium"));
        assert!(c.contains("border"));
    }

    #[test]
    fn build_class_disabled_adds_opacity_and_cursor() {
        let c = build_class(BtnVariant::Primary, true);
        assert!(c.contains("opacity-50"), "missing opacity-50: {c}");
        assert!(
            c.contains("cursor-not-allowed"),
            "missing cursor-not-allowed: {c}"
        );
    }

    #[test]
    fn build_class_enabled_omits_disabled_classes() {
        let c = build_class(BtnVariant::Primary, false);
        assert!(!c.contains("opacity-50"));
        assert!(!c.contains("cursor-not-allowed"));
    }

    #[test]
    fn default_variant_is_secondary() {
        assert_eq!(BtnVariant::default(), BtnVariant::Secondary);
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn renders_button_with_secondary_classes_by_default() {
        fn app() -> Element {
            rsx! { Btn { "Save" } }
        }
        let html = render(app);
        assert!(
            html.starts_with("<button"),
            "expected <button> root: {html}"
        );
        assert!(
            html.contains("bg-surface"),
            "missing secondary class: {html}"
        );
        assert!(html.contains("text-ink"), "missing text-ink: {html}");
        assert!(html.contains("Save"), "missing children: {html}");
    }

    #[test]
    fn renders_primary_variant() {
        fn app() -> Element {
            rsx! { Btn { variant: BtnVariant::Primary, "Go" } }
        }
        let html = render(app);
        assert!(html.contains("bg-accent"));
        assert!(html.contains("text-accent-ink"));
    }

    #[test]
    fn renders_ghost_variant() {
        fn app() -> Element {
            rsx! { Btn { variant: BtnVariant::Ghost, "x" } }
        }
        let html = render(app);
        assert!(html.contains("bg-transparent"));
        assert!(html.contains("text-ink-soft"));
    }

    #[test]
    fn renders_danger_variant() {
        fn app() -> Element {
            rsx! { Btn { variant: BtnVariant::Danger, "Delete" } }
        }
        let html = render(app);
        assert!(html.contains("text-bad"));
        assert!(html.contains("border-bad"));
    }

    #[test]
    fn disabled_render_includes_disabled_attr_and_classes() {
        fn app() -> Element {
            rsx! { Btn { variant: BtnVariant::Primary, disabled: true, "no" } }
        }
        let html = render(app);
        assert!(
            html.contains("disabled"),
            "missing disabled attribute: {html}"
        );
        assert!(html.contains("opacity-50"));
        assert!(html.contains("cursor-not-allowed"));
    }

    #[test]
    fn icon_renders_in_mono_span_before_children() {
        fn app() -> Element {
            rsx! {
                Btn { icon: Some(ImStr::from("+")), "Add" }
            }
        }
        let html = render(app);
        let icon_pos = html
            .find("font-mono")
            .expect(&format!("missing font-mono span: {html}"));
        let mr_pos = html.find("mr-1").expect("missing mr-1 class");
        let children_pos = html.find("Add").expect("missing button children");
        assert!(icon_pos < children_pos, "icon must precede children");
        assert!(mr_pos < children_pos, "mr-1 must precede children");
        assert!(html.contains(">+<"), "icon glyph not rendered: {html}");
    }

    #[test]
    fn no_icon_omits_mono_span() {
        fn app() -> Element {
            rsx! { Btn { "Hello" } }
        }
        let html = render(app);
        assert!(
            !html.contains("font-mono"),
            "unexpected font-mono span: {html}"
        );
    }
}
