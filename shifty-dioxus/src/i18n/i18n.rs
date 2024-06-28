use std::{collections::HashMap, hash::Hash, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18n<Key: Copy + PartialEq + Eq + Hash, Locale: Copy + PartialEq + Eq + Hash> {
    pub(crate) locales: HashMap<Locale, HashMap<Key, Rc<str>>>,
    pub(crate) fallback_locale: Locale,
    pub(crate) current_locale: Locale,
    pub(crate) fallback_string: Rc<str>,
}

impl<Key: Copy + PartialEq + Eq + Hash, Locale: Copy + PartialEq + Eq + Hash> I18n<Key, Locale> {
    pub fn new(current_locale: Locale, fallback_locale: Locale) -> Self {
        Self {
            locales: HashMap::new(),
            fallback_locale,
            current_locale,
            fallback_string: "??".into(),
        }
    }

    /*pub fn set_fallback_locale(&mut self, locale: Locale) {
        self.fallback_locale = locale;
    }

    pub fn set_current_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    pub fn set_fallback_string(&mut self, string: &str) {
        self.fallback_string = string.into();
    }*/

    pub fn add_locale(&mut self, locale: Locale) {
        self.locales.insert(locale, HashMap::new());
    }

    pub fn add_text(&mut self, locale: Locale, key: Key, text: &str) {
        if let Some(locale_map) = self.locales.get_mut(&locale) {
            locale_map.insert(key, text.into());
        }
    }

    pub fn get_text(&self, key: Key) -> Rc<str> {
        if let Some(locale_map) = self.locales.get(&self.current_locale) {
            if let Some(text) = locale_map.get(&key) {
                return text.clone();
            }
        }

        if let Some(locale_map) = self.locales.get(&self.fallback_locale) {
            if let Some(text) = locale_map.get(&key) {
                return text.clone();
            }
        }

        self.fallback_string.clone()
    }

    pub fn t(&self, key: Key) -> Rc<str> {
        self.get_text(key)
    }

    pub fn t_m(&self, key: Key, values_map: HashMap<&str, &str>) -> Rc<str> {
        let mut text = self.get_text(key).to_string();
        for (key, value) in values_map {
            text = text.replace(&format!("{{{}}}", key), value);
        }
        text.into()
    }
}
