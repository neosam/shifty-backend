//! `use_media_query` — a Dioxus hook reflecting whether a CSS media query
//! currently matches.
//!
//! Used to drive responsive component behavior (e.g. dialog variant
//! resolution, master/detail layout switching).
//!
//! Implementation notes:
//! - On non-WASM targets the hook returns a static `false` signal so unit
//!   tests of consumers remain deterministic.
//! - On WASM, the listener is registered via `add_event_listener_with_callback`
//!   and removed when the component unmounts (via the `Drop` impl on the
//!   stored guard struct held in `use_hook` storage).

use std::rc::Rc;

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

/// Returns a [`Signal<bool>`] reflecting whether the given mediaquery
/// currently matches. The signal updates live as the viewport crosses the
/// query's breakpoint.
pub fn use_media_query(query: &'static str) -> Signal<bool> {
    let initial = match_media_initial(query);
    let mut signal = use_signal(|| initial);

    use_hook(|| install_media_query_listener(query, signal.clone()));

    let _ = &mut signal;
    signal
}

#[cfg(target_arch = "wasm32")]
fn match_media_initial(query: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.match_media(query).ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn match_media_initial(_query: &str) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
struct MediaQueryGuard {
    mql: web_sys::MediaQueryList,
    closure: Closure<dyn FnMut(web_sys::MediaQueryListEvent)>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for MediaQueryGuard {
    fn drop(&mut self) {
        let _ = self
            .mql
            .remove_event_listener_with_callback("change", self.closure.as_ref().unchecked_ref());
    }
}

#[cfg(target_arch = "wasm32")]
fn install_media_query_listener(
    query: &'static str,
    mut signal: Signal<bool>,
) -> Option<Rc<MediaQueryGuard>> {
    let window = web_sys::window()?;
    let mql = window.match_media(query).ok().flatten()?;
    let closure: Closure<dyn FnMut(web_sys::MediaQueryListEvent)> =
        Closure::wrap(Box::new(move |event: web_sys::MediaQueryListEvent| {
            signal.set(event.matches());
        }));
    let _ = mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
    Some(Rc::new(MediaQueryGuard { mql, closure }))
}

#[cfg(not(target_arch = "wasm32"))]
fn install_media_query_listener(_query: &'static str, _signal: Signal<bool>) -> Option<Rc<()>> {
    None
}
