//! `Dialog` — the redesigned modal surface.
//!
//! Coexists with the legacy [`crate::component::modal::Modal`] until call
//! sites are migrated per page (changes 05–09). New code SHOULD use
//! `Dialog` because it offers four layout variants
//! ([`DialogVariant::Center`], [`DialogVariant::Sheet`],
//! [`DialogVariant::Bottom`], [`DialogVariant::Auto`]), header/footer slots,
//! body scroll lock, ESC + backdrop dismissal and a small `use_media_query`
//! hook used internally for [`DialogVariant::Auto`] resolution.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::base_types::ImStr;
use crate::component::atoms::use_media_query;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

/// Layout variant of [`Dialog`].
///
/// `Auto` resolves at runtime via a `(max-width: 720px)` mediaquery, mapping
/// to `Bottom` on mobile and `Center` on desktop. The resolution updates
/// live: a viewport that crosses the breakpoint while the dialog is open
/// switches the layout without closing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DialogVariant {
    Center,
    Sheet,
    Bottom,
    Auto,
}

impl Default for DialogVariant {
    fn default() -> Self {
        DialogVariant::Auto
    }
}

/// Resolves `Auto` to either `Bottom` (mobile) or `Center` (desktop). Other
/// variants are returned unchanged.
pub(crate) fn resolve_variant(variant: DialogVariant, is_mobile: bool) -> DialogVariant {
    match variant {
        DialogVariant::Auto => {
            if is_mobile {
                DialogVariant::Bottom
            } else {
                DialogVariant::Center
            }
        }
        other => other,
    }
}

/// Returns `(justify, align, padding)` for the backdrop flex container per
/// resolved variant. `padding` is in pixels.
pub(crate) fn backdrop_layout(variant: DialogVariant) -> (&'static str, &'static str, u32) {
    match variant {
        DialogVariant::Center => ("center", "center", 16),
        DialogVariant::Sheet => ("flex-end", "stretch", 0),
        DialogVariant::Bottom => ("center", "flex-end", 0),
        DialogVariant::Auto => ("center", "center", 16),
    }
}

/// Returns the inline style block for the backdrop. Centralised so tests
/// can verify that variant-specific properties are produced.
pub(crate) fn backdrop_style(variant: DialogVariant) -> String {
    let (justify, align, padding) = backdrop_layout(variant);
    format!(
        "position:fixed;inset:0;background:var(--modal-veil);z-index:200;\
         display:flex;justify-content:{};align-items:{};padding:{}px;\
         animation:shifty-modal-fade 160ms ease-out;",
        justify, align, padding
    )
}

/// Returns the inline style block for the panel for a given resolved
/// variant and `width` prop. Centralised so tests can assert on
/// variant-specific properties (radius, animation, width caps).
pub(crate) fn panel_style(variant: DialogVariant, width: u32) -> String {
    let base = "background:var(--surface);color:var(--ink);\
                border:1px solid var(--border);\
                box-shadow:0 12px 40px rgba(0,0,0,0.18),0 2px 6px rgba(0,0,0,0.08);\
                display:flex;flex-direction:column;overflow:hidden;";
    match variant {
        DialogVariant::Center | DialogVariant::Auto => format!(
            "{}width:min({}px,100%);max-height:min(86vh,720px);\
             border-radius:var(--r-lg);\
             animation:shifty-modal-pop 180ms cubic-bezier(.2,.8,.2,1);",
            base, width
        ),
        DialogVariant::Sheet => format!(
            "{}width:min({}px,100%);height:100vh;max-height:100vh;\
             border-radius:0;border-right:none;border-top:none;border-bottom:none;\
             animation:shifty-modal-slide-right 220ms cubic-bezier(.2,.8,.2,1);",
            base,
            width + 60
        ),
        DialogVariant::Bottom => format!(
            "{}width:100%;max-height:92vh;\
             border-radius:var(--r-lg) var(--r-lg) 0 0;border-bottom:none;\
             animation:shifty-modal-slide-up 220ms cubic-bezier(.2,.8,.2,1);",
            base
        ),
    }
}

/// Returns `true` when the keyboard event represents an Escape press.
pub(crate) fn is_escape_key(key: &str) -> bool {
    key == "Escape"
}

#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    pub on_close: EventHandler<()>,
    pub title: ImStr,

    #[props(!optional, default = None)]
    pub subtitle: Option<ImStr>,

    pub children: Element,

    #[props(!optional, default = None)]
    pub footer: Option<Element>,

    #[props(default = DialogVariant::Auto)]
    pub variant: DialogVariant,

    #[props(default = 460)]
    pub width: u32,
}

/// Modal dialog with four layout variants, header/footer slots, body
/// scroll lock and standard dismissal paths (backdrop click, ESC, X
/// button). See module docs for the relationship to legacy `Modal`.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    rsx! {
        DialogContent {
            on_close: props.on_close,
            title: props.title.clone(),
            subtitle: props.subtitle.clone(),
            footer: props.footer.clone(),
            variant: props.variant,
            width: props.width,
            { props.children }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DialogContentProps {
    on_close: EventHandler<()>,
    title: ImStr,
    subtitle: Option<ImStr>,
    children: Element,
    footer: Option<Element>,
    variant: DialogVariant,
    width: u32,
}

/// Inner component that holds all hooks. Keeping the hooks on this
/// always-mounted-when-open component (instead of on [`Dialog`]) avoids
/// the rules-of-hooks violation that would otherwise occur when the parent
/// early-returns based on `open`.
#[component]
fn DialogContent(props: DialogContentProps) -> Element {
    let is_mobile = use_media_query("(max-width: 720px)");
    let resolved = resolve_variant(props.variant, *is_mobile.read());

    use_body_scroll_lock();
    use_escape_dismiss(props.on_close);

    let backdrop = backdrop_style(resolved);
    let panel = panel_style(resolved, props.width);
    let header_padding = if matches!(resolved, DialogVariant::Bottom) {
        "8px 18px 0"
    } else {
        "16px 18px 0"
    };

    let on_close = props.on_close;
    let on_close_for_x = props.on_close;

    rsx! {
        div {
            role: "presentation",
            style: "{backdrop}",
            onclick: move |_| {
                on_close.call(());
            },

            div {
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "shifty-dialog-title",
                style: "{panel}",
                onclick: move |evt| {
                    evt.stop_propagation();
                },

                if matches!(resolved, DialogVariant::Bottom) {
                    div { class: "flex justify-center pt-2",
                        div { class: "w-9 h-1 rounded-full bg-border-strong" }
                    }
                }

                div {
                    class: "flex items-start justify-between gap-3",
                    style: "padding:{header_padding};",
                    div { class: "min-w-0",
                        h3 {
                            id: "shifty-dialog-title",
                            class: "m-0 text-base font-bold tracking-tight",
                            "{props.title}"
                        }
                        if let Some(subtitle) = props.subtitle.as_ref() {
                            div { class: "text-xs text-ink-muted mt-0.5",
                                "{subtitle}"
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        "aria-label": "Close",
                        class: "w-7 h-7 inline-flex items-center justify-center rounded-md border border-transparent bg-transparent text-ink-muted text-lg leading-none flex-shrink-0 hover:bg-surface-alt hover:text-ink",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_close_for_x.call(());
                        },
                        "×"
                    }
                }

                div {
                    class: "flex-1 overflow-y-auto",
                    style: "padding:14px 18px 16px;",
                    { props.children }
                }

                if let Some(footer) = props.footer.as_ref() {
                    div {
                        class: "flex justify-end gap-2 px-[18px] py-3 border-t border-border bg-surface-alt",
                        { footer.clone() }
                    }
                }
            }
        }
    }
}

// ─── Body scroll lock ───────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct BodyScrollLockGuard {
    prev_overflow: String,
}

#[cfg(target_arch = "wasm32")]
impl Drop for BodyScrollLockGuard {
    fn drop(&mut self) {
        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            let _ = body.style().set_property("overflow", &self.prev_overflow);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn install_body_scroll_lock() -> Option<Rc<BodyScrollLockGuard>> {
    let body = web_sys::window()?.document()?.body()?;
    let style = body.style();
    let prev = style.get_property_value("overflow").unwrap_or_default();
    let _ = style.set_property("overflow", "hidden");
    Some(Rc::new(BodyScrollLockGuard {
        prev_overflow: prev,
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn install_body_scroll_lock() -> Option<Rc<()>> {
    None
}

fn use_body_scroll_lock() {
    use_hook(|| install_body_scroll_lock());
}

// ─── ESC dismiss ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
struct EscapeGuard {
    closure: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for EscapeGuard {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                "keydown",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn install_escape_listener(on_close: EventHandler<()>) -> Option<Rc<EscapeGuard>> {
    let window = web_sys::window()?;
    let closure: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if is_escape_key(&event.key()) {
                on_close.call(());
            }
        }));
    let _ = window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
    Some(Rc::new(EscapeGuard { closure }))
}

#[cfg(not(target_arch = "wasm32"))]
fn install_escape_listener(_on_close: EventHandler<()>) -> Option<Rc<()>> {
    None
}

fn use_escape_dismiss(on_close: EventHandler<()>) {
    use_hook(|| install_escape_listener(on_close));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_variant_auto_picks_bottom_on_mobile() {
        assert_eq!(
            resolve_variant(DialogVariant::Auto, true),
            DialogVariant::Bottom
        );
    }

    #[test]
    fn resolve_variant_auto_picks_center_on_desktop() {
        assert_eq!(
            resolve_variant(DialogVariant::Auto, false),
            DialogVariant::Center
        );
    }

    #[test]
    fn resolve_variant_passes_through_explicit_variants() {
        assert_eq!(
            resolve_variant(DialogVariant::Sheet, true),
            DialogVariant::Sheet
        );
        assert_eq!(
            resolve_variant(DialogVariant::Center, true),
            DialogVariant::Center
        );
        assert_eq!(
            resolve_variant(DialogVariant::Bottom, false),
            DialogVariant::Bottom
        );
    }

    #[test]
    fn backdrop_layout_center_centers() {
        assert_eq!(
            backdrop_layout(DialogVariant::Center),
            ("center", "center", 16)
        );
    }

    #[test]
    fn backdrop_layout_sheet_aligns_right_full_height() {
        assert_eq!(
            backdrop_layout(DialogVariant::Sheet),
            ("flex-end", "stretch", 0)
        );
    }

    #[test]
    fn backdrop_layout_bottom_aligns_to_bottom_full_width() {
        assert_eq!(
            backdrop_layout(DialogVariant::Bottom),
            ("center", "flex-end", 0)
        );
    }

    #[test]
    fn backdrop_style_uses_modal_veil_token() {
        let s = backdrop_style(DialogVariant::Center);
        assert!(
            s.contains("background:var(--modal-veil)"),
            "missing veil: {s}"
        );
        assert!(s.contains("z-index:200"), "z-index not set: {s}");
        assert!(
            s.contains("animation:shifty-modal-fade"),
            "missing fade animation: {s}"
        );
    }

    #[test]
    fn panel_style_center_uses_pop_animation_and_width_cap() {
        let s = panel_style(DialogVariant::Center, 460);
        assert!(s.contains("width:min(460px,100%)"), "width cap wrong: {s}");
        assert!(
            s.contains("animation:shifty-modal-pop"),
            "missing pop animation: {s}"
        );
        assert!(
            s.contains("border-radius:var(--r-lg)"),
            "missing rounded corners: {s}"
        );
    }

    #[test]
    fn panel_style_sheet_adds_60_to_width_and_full_height() {
        let s = panel_style(DialogVariant::Sheet, 460);
        assert!(
            s.contains("width:min(520px,100%)"),
            "sheet width wrong: {s}"
        );
        assert!(s.contains("height:100vh"), "sheet not full height: {s}");
        assert!(
            s.contains("animation:shifty-modal-slide-right"),
            "missing slide-right animation: {s}"
        );
    }

    #[test]
    fn panel_style_bottom_full_width_and_top_radius_only() {
        let s = panel_style(DialogVariant::Bottom, 460);
        assert!(s.contains("width:100%"), "bottom not full width: {s}");
        assert!(
            s.contains("border-radius:var(--r-lg) var(--r-lg) 0 0"),
            "wrong radius: {s}"
        );
        assert!(
            s.contains("animation:shifty-modal-slide-up"),
            "missing slide-up animation: {s}"
        );
    }

    #[test]
    fn is_escape_key_recognises_escape() {
        assert!(is_escape_key("Escape"));
        assert!(!is_escape_key("Esc"));
        assert!(!is_escape_key(""));
        assert!(!is_escape_key("Enter"));
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn closed_dialog_renders_nothing() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: false,
                    on_close: |_| {},
                    title: ImStr::from("Hidden"),
                    div { "should not appear" }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("should not appear"),
            "closed dialog should not render children: {html}"
        );
        assert!(
            !html.contains("shifty-dialog-title"),
            "closed dialog should not render header: {html}"
        );
    }

    #[test]
    fn open_dialog_renders_title_and_close_button() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("Settings"),
                    div { "body" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Settings"), "missing title: {html}");
        assert!(
            html.contains(r#"id="shifty-dialog-title""#),
            "missing title id: {html}"
        );
        assert!(html.contains("body"), "missing body: {html}");
        assert!(
            html.contains(r#"aria-label="Close""#),
            "missing close button: {html}"
        );
        assert!(html.contains("×"), "missing × glyph: {html}");
    }

    #[test]
    fn subtitle_renders_when_provided() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("Settings"),
                    subtitle: Some(ImStr::from("Configure")),
                    div { "x" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("Configure"), "missing subtitle: {html}");
        assert!(
            html.contains("text-ink-muted"),
            "subtitle styling missing: {html}"
        );
    }

    #[test]
    fn no_subtitle_renders_no_subtitle_div() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("Settings"),
                    div { "x" }
                }
            }
        }
        let html = render(app);
        // The subtitle slot is the only place that combines `text-xs` with
        // `text-ink-muted` — the close-X button uses `text-lg text-ink-muted`,
        // so this assertion is specific to the subtitle div.
        assert!(
            !html.contains("text-xs text-ink-muted"),
            "unexpected subtitle markup: {html}"
        );
    }

    #[test]
    fn footer_renders_when_provided() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("T"),
                    footer: Some(rsx! { button { "OK" } }),
                    div { "x" }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("OK"), "footer button missing: {html}");
        assert!(
            html.contains("border-t border-border bg-surface-alt"),
            "footer container styling missing: {html}"
        );
    }

    #[test]
    fn auto_variant_resolves_to_center_outside_wasm() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("T"),
                    variant: DialogVariant::Auto,
                    div { "x" }
                }
            }
        }
        let html = render(app);
        // Center variant inline style includes pop animation
        assert!(
            html.contains("shifty-modal-pop"),
            "auto did not resolve to center on non-wasm: {html}"
        );
        // No drag handle on center
        assert!(
            !html.contains("w-9 h-1 rounded-full"),
            "drag handle should not appear on center: {html}"
        );
    }

    #[test]
    fn bottom_variant_renders_drag_handle() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("T"),
                    variant: DialogVariant::Bottom,
                    div { "x" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("w-9 h-1 rounded-full"),
            "drag handle missing on bottom variant: {html}"
        );
        assert!(
            html.contains("shifty-modal-slide-up"),
            "missing slide-up animation: {html}"
        );
    }

    #[test]
    fn sheet_variant_uses_slide_right_animation() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("T"),
                    variant: DialogVariant::Sheet,
                    div { "x" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("shifty-modal-slide-right"),
            "missing slide-right animation: {html}"
        );
        // Sheet has no drag handle
        assert!(
            !html.contains("w-9 h-1 rounded-full"),
            "sheet should not have a drag handle: {html}"
        );
    }

    #[test]
    fn dialog_root_has_presentation_role_and_modal_panel_has_dialog_role() {
        fn app() -> Element {
            rsx! {
                Dialog {
                    open: true,
                    on_close: |_| {},
                    title: ImStr::from("T"),
                    div { "x" }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"role="presentation""#),
            "backdrop role missing: {html}"
        );
        assert!(
            html.contains(r#"role="dialog""#),
            "panel role missing: {html}"
        );
        assert!(
            html.contains(r#"aria-modal="true""#),
            "aria-modal missing: {html}"
        );
    }

    #[test]
    fn default_variant_is_auto() {
        assert_eq!(DialogVariant::default(), DialogVariant::Auto);
    }
}
