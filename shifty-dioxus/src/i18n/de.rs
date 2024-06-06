use super::{I18n, Key, Locale};

pub fn add_i18n_de(i18n: &mut I18n<Key, Locale>) {
    i18n.add_locale(Locale::En);
    i18n.add_text(Locale::En, Key::Home, "Start");
    i18n.add_text(Locale::En, Key::About, "Über");

    // Add weekdays
    i18n.add_text(Locale::En, Key::Monday, "Montag");
    i18n.add_text(Locale::En, Key::Tuesday, "Dienstag");
    i18n.add_text(Locale::En, Key::Wednesday, "Mittwoch");
    i18n.add_text(Locale::En, Key::Thursday, "Donnerstag");
    i18n.add_text(Locale::En, Key::Friday, "Freitag");
    i18n.add_text(Locale::En, Key::Saturday, "Samstag");
    i18n.add_text(Locale::En, Key::Sunday, "Sonntag");
}