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

    // Top bar
    i18n.add_text(Locale::En, Key::Shiftplan, "Schichtplan");
    i18n.add_text(Locale::En, Key::Employees, "Angestellte");
    i18n.add_text(Locale::En, Key::MyTime, "Zeitkonto");
    i18n.add_text(Locale::En, Key::YearOverview, "Jahresübersicht");
    i18n.add_text(Locale::En, Key::Logout, "Logout");

    // Shiftplan
    i18n.add_text(
        Locale::En,
        Key::ShiftplanCalendarWeek,
        "{week}/{year} - vom {date}",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanTakeLastWeek,
        "Letzte Woche hinzufügen",
    );
    i18n.add_text(Locale::En, Key::ShiftplanEditAs, "Du bearbeitest:");
    i18n.add_text(Locale::En, Key::ShiftplanYouAre, "Du bist ");
    i18n.add_text(
        Locale::En,
        Key::ConflictBookingsHeader,
        "Fehlerhafte Zuweisungen",
    );

    // Employee report
    i18n.add_text(Locale::En, Key::OverallHeading, "Gesamtansicht");
    i18n.add_text(
        Locale::En,
        Key::WorkingHoursPerWeekHeading,
        "Stunden pro Woche",
    );
    i18n.add_text(
        Locale::En,
        Key::WorkingHoursPerDayHeading,
        "Stunden pro Tag",
    );
    i18n.add_text(Locale::En, Key::ExtraHoursHeading, "Zusatzstunden");

    i18n.add_text(Locale::En, Key::Balance, "Stundenkonto");
    i18n.add_text(Locale::En, Key::Required, "Soll");
    i18n.add_text(Locale::En, Key::Overall, "Gesamt");
    i18n.add_text(Locale::En, Key::CategoryShiftplan, "Schichtplan");
    i18n.add_text(Locale::En, Key::CategoryExtraWork, "Zusatzarbeit");
    i18n.add_text(Locale::En, Key::CategoryVacation, "Urlaub");
    i18n.add_text(Locale::En, Key::CategorySickLeave, "Krank");
    i18n.add_text(Locale::En, Key::CategoryHolidays, "Feiertage");

    i18n.add_text(Locale::En, Key::ShowDetails, "Mehr");
    i18n.add_text(Locale::En, Key::HideDetails, "Weniger");

    i18n.add_text(Locale::En, Key::Hours, "Std.");

    i18n.add_text(Locale::En, Key::AddEntry, "Hinzufügen");
    i18n.add_text(
        Locale::En,
        Key::WorkHoursDescription,
        "(Arbeitsstunden, die nicht im Schichtplan enthalten sind)",
    );

    // Add extra hours form
    i18n.add_text(
        Locale::En,
        Key::AddExtraHoursFormTitle,
        "Extra Stunden hinzufügen",
    );
    i18n.add_text(Locale::En, Key::AmountOfHours, "Anzahl der Stunden");
    i18n.add_text(Locale::En, Key::Category, "Kategorie");
    i18n.add_text(Locale::En, Key::Description, "Beschreibung");
    i18n.add_text(Locale::En, Key::When, "Wann");
    i18n.add_text(Locale::En, Key::Submit, "Hinzufügen");
    i18n.add_text(Locale::En, Key::Cancel, "Abbrechen");

    i18n.add_text(
        Locale::En,
        Key::AddExtraHoursChoiceTitle,
        "Bitte Kategorie auswählen",
    );
    i18n.add_text(Locale::En, Key::AddVacationTitle, "Urlaub hinzufügen");
    i18n.add_text(Locale::En, Key::AddHolidaysTitle, "Feiertag hinzufügen");
    i18n.add_text(
        Locale::En,
        Key::AddSickLeaveTitle,
        "Krankheitstage hinzufügen",
    );
    i18n.add_text(Locale::En, Key::WeekLabel, "Woche");
    i18n.add_text(Locale::En, Key::FullWeekLabel, "Ganze Woche");

    // Non-prod warnings
    i18n.add_text(
        Locale::En,
        Key::NonProdWarning,
        "Das ist nur eine Seite zum Testen❗",
    );
    i18n.add_text(Locale::En, Key::NonProdWarningDetails,
        "Diese Seite darf nicht produktiv genutzt werden! Sie kann Bugs enthalten und Daten können jederzeit zurückgesetzt werden und verloren gehen!");

    // Not authenticated page
    i18n.add_text(Locale::En, Key::WelcomeTitle, "Willkommen zu Shifty!");
    i18n.add_text(Locale::En, Key::PleaseLogin, "Bitte log dich ein.");
    i18n.add_text(
        Locale::En,
        Key::PleaseChoose,
        "Bitte wähle eine Ansicht von der oberen Leiste.",
    );
}
