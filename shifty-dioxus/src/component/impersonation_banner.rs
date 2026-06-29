//! Persistent non-closable amber impersonation banner (Phase 32, D-32-04).
//!
//! While the admin is impersonating another user, `ImpersonationBanner` renders
//! an amber bar (using static `bg-warn-soft`/`border-warn`/`text-warn` tokens —
//! Pitfall 5: no interpolated class strings) above the router outlet on every
//! page.  The banner is intentionally non-closable (D-32-04); the Stop button
//! dispatches `ImpersonateAction::Stop`, which calls DELETE /admin/impersonate
//! and reloads the page (D-32-06 / IMP-04).
//!
//! ## Structure
//!
//! The public `ImpersonationBanner` component reads the global `IMPERSONATE_STORE`
//! and holds the `use_coroutine_handle`.  The private `ImpersonationBannerView`
//! is a prop-driven delegate that can be rendered in SSR unit tests without the
//! impersonate service being registered in the VirtualDom context.

use dioxus::prelude::*;

use crate::{
    component::atoms::{Btn, BtnVariant},
    i18n::Key,
    service::{
        i18n::I18N,
        impersonate::{ImpersonateAction, ImpersonateStore, IMPERSONATE_STORE},
    },
};

// ─── Private prop-driven view component (testable without coroutine) ─────────

#[derive(Props, Clone, PartialEq)]
struct ImpersonationBannerViewProps {
    store: ImpersonateStore,
    on_stop: EventHandler<()>,
}

/// Prop-driven visual implementation.
///
/// Does not call `use_coroutine_handle` so it can be rendered in SSR unit tests
/// without the impersonate service registered.  All behaviour is driven by
/// `store` and `on_stop`.
#[component]
fn ImpersonationBannerView(props: ImpersonationBannerViewProps) -> Element {
    if !props.store.impersonating {
        return rsx! {};
    }

    let i18n = I18N.read().clone();
    let user_display = props
        .store
        .user_id
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("?");
    let banner_text = i18n
        .t(Key::ImpersonateBanner)
        .as_ref()
        .replace("{user}", user_display);
    let stop_label = i18n.t(Key::ImpersonateStop).to_string();
    let p10_hint = i18n.t(Key::ImpersonateP10Hint).to_string();
    let on_stop = props.on_stop;

    rsx! {
        // D-32-04: non-closable amber bar; static Tailwind classes (Pitfall 5).
        div {
            class: "bg-warn-soft border-l-4 border-warn text-warn px-4 py-2 flex items-center justify-between gap-4 print:hidden",
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-body font-semibold text-warn", "{banner_text}" }
                span { class: "text-micro text-warn", "{p10_hint}" }
            }
            // One-click Stop — no close/dismiss (D-32-04).
            Btn {
                variant: BtnVariant::Secondary,
                on_click: move |_| on_stop.call(()),
                "{stop_label}"
            }
        }
    }
}

// ─── Public component ─────────────────────────────────────────────────────────

/// Persistent amber impersonation banner.
///
/// Renders nothing when not impersonating.  Mount in `app.rs` above the router
/// outlet so the banner appears on every route (D-32-04 / SC1).
///
/// The Stop button dispatches `ImpersonateAction::Stop` via the impersonate
/// service coroutine, which calls DELETE /admin/impersonate then reloads the
/// page (D-32-06 / IMP-04).
#[component]
pub fn ImpersonationBanner() -> Element {
    let store = IMPERSONATE_STORE.read().clone();
    let impersonate_service = use_coroutine_handle::<ImpersonateAction>();
    rsx! {
        ImpersonationBannerView {
            store,
            on_stop: move |_| impersonate_service.send(ImpersonateAction::Stop),
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_types::ImStr;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    /// Behaviour: when not impersonating, the banner renders nothing — no amber
    /// bar and no Stop button (so the UI is clean for normal sessions).
    #[test]
    fn banner_hidden_when_not_impersonating() {
        fn app() -> Element {
            rsx! {
                ImpersonationBannerView {
                    store: ImpersonateStore {
                        impersonating: false,
                        user_id: None,
                        loaded: true,
                    },
                    on_stop: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("bg-warn-soft"),
            "non-impersonating banner must not render amber bar, got: {html}"
        );
        assert!(
            !html.contains("Stop"),
            "non-impersonating banner must not show Stop button, got: {html}"
        );
    }

    /// Behaviour: when impersonating with user_id Some("alex"), the banner renders
    /// the amber bar, interpolates "alex" into the banner text, shows a Stop
    /// button, shows the P10 hint, and has NO close/dismiss control (D-32-04).
    #[test]
    fn banner_shown_when_impersonating() {
        fn app() -> Element {
            rsx! {
                ImpersonationBannerView {
                    store: ImpersonateStore {
                        impersonating: true,
                        user_id: Some(ImStr::from("alex")),
                        loaded: true,
                    },
                    on_stop: |_| {},
                }
            }
        }
        let html = render(app);

        // Amber styling (static Tailwind classes, Pitfall 5).
        assert!(
            html.contains("bg-warn-soft"),
            "impersonating banner must have amber background, got: {html}"
        );
        assert!(
            html.contains("border-warn"),
            "impersonating banner must have warn border, got: {html}"
        );

        // D-32-03: raw user_id (no name lookup) interpolated in banner text.
        assert!(
            html.contains("alex"),
            "impersonating banner must contain the impersonated username, got: {html}"
        );

        // Stop button present (Key::ImpersonateStop).
        assert!(
            html.contains("Stop"),
            "impersonating banner must have a Stop button, got: {html}"
        );

        // P10 hint present (D-32-02a).
        assert!(
            html.contains("Admin-only"),
            "impersonating banner must contain the P10 hint, got: {html}"
        );

        // Non-closable (D-32-04): no close or dismiss control.
        let lower = html.to_lowercase();
        assert!(
            !lower.contains("close") && !lower.contains("dismiss") && !html.contains('×'),
            "impersonating banner must not have a close/dismiss control, got: {html}"
        );
    }
}
