use dioxus::prelude::*;

use crate::i18n;
use crate::i18n::I18nType;

pub static I18N: GlobalSignal<I18nType> = Signal::global(|| i18n::generate(i18n::Locale::En));

pub async fn i18n_service(_rx: UnboundedReceiver<()>) {
    let set_browser_language = || async {
        let language = web_sys::window()
            .map(|w| w.navigator())
            .and_then(|n| n.language())
            .map(|locale| locale[..2].to_string())
            .unwrap_or_else(|| "en".to_string());
        let i18n = i18n::generate(i18n::Locale::from_str(&language));
        *I18N.write() = i18n;
    };

    set_browser_language().await;
}
