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
    i18n.add_text(
        Locale::En,
        Key::PersonalCalendarExport,
        "Persönlichen Kalender exportieren (iCal)",
    );
    i18n.add_text(
        Locale::En,
        Key::UnsufficientlyBookedCalendarExport,
        "Unterbesetzte Schichten Kalender exportieren (iCal)",
    );

    // Weekly overview page
    i18n.add_text(Locale::En, Key::WeeklyOverviewTitle, "Jahresübersicht");
    i18n.add_text(
        Locale::En,
        Key::AvailableRequiredHours,
        "Verfügbare / Benötigte Stunden",
    );
    i18n.add_text(Locale::En, Key::MissingHours, "Differenz");

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
    i18n.add_text(Locale::En, Key::WorkDetailsHeading, "Arbeitsverträge");

    i18n.add_text(Locale::En, Key::Balance, "Stundenkonto");
    i18n.add_text(Locale::En, Key::Required, "Soll");
    i18n.add_text(Locale::En, Key::Overall, "Gesamt");
    i18n.add_text(Locale::En, Key::CarryoverBalance, "Übertrag Vorjahr");
    i18n.add_text(Locale::En, Key::CategoryShiftplan, "Schichtplan");
    i18n.add_text(Locale::En, Key::CategoryExtraWork, "Zusatzarbeit");
    i18n.add_text(Locale::En, Key::CategoryVacation, "Urlaub");
    i18n.add_text(Locale::En, Key::CategoryVacationHours, "Urlaub (Stunden)");
    i18n.add_text(Locale::En, Key::CategoryVacationDays, "Urlaub");
    i18n.add_text(Locale::En, Key::CategorySickLeave, "Krank");
    i18n.add_text(Locale::En, Key::CategoryHolidays, "Feiertage");
    i18n.add_text(Locale::En, Key::CategoryUnavailable, "Nicht verfügbar");
    i18n.add_text(Locale::En, Key::CategoryCustom, "Sonstige");

    i18n.add_text(Locale::En, Key::VacationDaysLabel, "Urlaubstage");
    i18n.add_text(Locale::En, Key::VacationCarryoverLabel, "Vorjahresurlaub");

    i18n.add_text(Locale::En, Key::ShowDetails, "Mehr");
    i18n.add_text(Locale::En, Key::HideDetails, "Weniger");

    i18n.add_text(Locale::En, Key::Hours, "Std.");
    i18n.add_text(Locale::En, Key::Days, "Tage");

    i18n.add_text(Locale::En, Key::AddEntry, "Sonstige Stunden hinzufügen");
    i18n.add_text(
        Locale::En,
        Key::WorkHoursDescription,
        "(Arbeitsstunden, die nicht im Schichtplan enthalten sind)",
    );
    i18n.add_text(
        Locale::En,
        Key::UnavailableDescription,
        "(Stunden, die nicht das Zeitkonto beeinflussen aber dem Schichtplanner zeigen, dass du nicht verfügbar bist)",
    );
    i18n.add_text(Locale::En, Key::ActionsLabel, "Mehr");
    i18n.add_text(
        Locale::En,
        Key::ShowFullYearLabel,
        "Bericht des vollständigen Jahres",
    );
    i18n.add_text(
        Locale::En,
        Key::ShowUntilNowLabel,
        "Bericht bis zur heutigen Woche",
    );
    i18n.add_text(
        Locale::En,
        Key::AddWorkDetailsLabel,
        "Arbeitvertrag hinzufügen",
    );
    i18n.add_text(
        Locale::En,
        Key::CurrentWeekNote,
        "Nur Daten bis zur aktuellen Woche werden angezeigt.",
    );

    // Add extra hours form
    i18n.add_text(
        Locale::En,
        Key::AddExtraHoursFormTitle,
        "Extra Stunden hinzufügen",
    );
    i18n.add_text(Locale::En, Key::AmountOfHours, "Anzahl der Stunden");
    i18n.add_text(Locale::En, Key::AmountOfDays, "Anzahl der Tage");
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
    i18n.add_text(Locale::En, Key::PleaseLogin, "Hier klicken zum Einloggen.");
    i18n.add_text(
        Locale::En,
        Key::PleaseChoose,
        "Bitte wähle eine Ansicht von der oberen Leiste.",
    );

    // Employee work details form
    i18n.add_text(
        Locale::En,
        Key::AddWorkDetailsFormTitle,
        "Arbeitsvertrag von {name}",
    );
    i18n.add_text(Locale::En, Key::FromLabel, "Von");
    i18n.add_text(Locale::En, Key::ToLabel, "Bis");
    i18n.add_text(Locale::En, Key::WorkdaysLabel, "Wochentage");
    i18n.add_text(
        Locale::En,
        Key::ExpectedHoursPerWeekLabel,
        "Wochenarbeitsstunden",
    );
    i18n.add_text(Locale::En, Key::DaysPerWeekLabel, "Arbeitstage pro Woche");
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementsPerYearLabel,
        "Urlaubsanspruch im Jahr",
    );
    i18n.add_text(Locale::En, Key::HolidaysInHoursLabel, "Feiertag in Stunden");
    i18n.add_text(
        Locale::En,
        Key::WorkdaysInHoursLabel,
        "Arbeitstag in Stunden",
    );

    // Slot edit
    i18n.add_text(Locale::En, Key::SlotEditTitle, "Slot bearbeiten");
    i18n.add_text(Locale::En, Key::SlotNewTitle, "Neuen Slot erstellen");
    i18n.add_text(
        Locale::En,
        Key::SlotEditExplanation,
        "Diese Änderungen werden ab der Kalenderwoche {week}/{year} angewendet.  Vorherige Woche bleiben unverändert.",
    );
    i18n.add_text(
        Locale::En,
        Key::SlotEditValidUntilExplanation,
        "Die Änderungen werden nur bis zum {date} angewendet.  Nachfolgende Wochen bleiben unverändert.",
    );
    i18n.add_text(Locale::En, Key::MinPersonsLabel, "Benötigte Personen");
    i18n.add_text(Locale::En, Key::WeekdayLabel, "Wochentag");
    i18n.add_text(Locale::En, Key::SaveLabel, "Speichern");
    i18n.add_text(Locale::En, Key::CancelLabel, "Abbrechen");
    i18n.add_text(
        Locale::En,
        Key::SlotEditSaveError,
        "Konnte nicht speichern.  Bitte prüfen, dass der Slot nicht mit anderen Slots überlappt.",
    );
}
