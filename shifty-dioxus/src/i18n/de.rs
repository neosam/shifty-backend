use super::{I18n, Key, Locale};

pub fn add_i18n_de(i18n: &mut I18n<Key, Locale>) {
    i18n.add_locale(Locale::De);
    i18n.add_text(Locale::De, Key::Home, "Start");
    i18n.add_text(Locale::De, Key::About, "Über");

    // Add weekdays
    i18n.add_text(Locale::De, Key::Monday, "Montag");
    i18n.add_text(Locale::De, Key::Tuesday, "Dienstag");
    i18n.add_text(Locale::De, Key::Wednesday, "Mittwoch");
    i18n.add_text(Locale::De, Key::Thursday, "Donnerstag");
    i18n.add_text(Locale::De, Key::Friday, "Freitag");
    i18n.add_text(Locale::De, Key::Saturday, "Samstag");
    i18n.add_text(Locale::De, Key::Sunday, "Sonntag");

    // Top bar
    i18n.add_text(Locale::De, Key::Shiftplan, "Schichtplan");
    i18n.add_text(Locale::De, Key::Employees, "Mitarbeiter");
    i18n.add_text(Locale::De, Key::MyTime, "Zeitkonto");
    i18n.add_text(Locale::De, Key::YearOverview, "Jahresübersicht");
    i18n.add_text(Locale::De, Key::Logout, "Logout");

    // Shiftplan
    i18n.add_text(
        Locale::De,
        Key::ShiftplanCalendarWeek,
        "{week}/{year} - vom {date}",
    );
    i18n.add_text(
        Locale::De,
        Key::ShiftplanTakeLastWeek,
        "Letzte Woche hinzufügen",
    );
    i18n.add_text(Locale::De, Key::ShiftplanEditAs, "Du bearbeitest:");
    i18n.add_text(Locale::De, Key::ShiftplanYouAre, "Du bist ");
    i18n.add_text(
        Locale::De,
        Key::ConflictBookingsHeader,
        "Fehlerhafte Zuweisungen",
    );
    i18n.add_text(
        Locale::De,
        Key::PersonalCalendarExport,
        "Persönlichen Kalender exportieren (iCal)",
    );
    i18n.add_text(
        Locale::De,
        Key::UnsufficientlyBookedCalendarExport,
        "Unterbesetzte Schichten Kalender exportieren (iCal)",
    );
    i18n.add_text(Locale::De, Key::WeekMessage, "Wochennachricht");

    // Weekly overview page
    i18n.add_text(Locale::De, Key::WeeklyOverviewTitle, "Jahresübersicht");
    i18n.add_text(
        Locale::De,
        Key::AvailableRequiredHours,
        "Verfügbare / Benötigte Stunden",
    );
    i18n.add_text(Locale::De, Key::MissingHours, "Differenz");
    i18n.add_text(Locale::De, Key::UnsavedChanges, "Ungespeicherte Änderungen");

    // Employee report
    i18n.add_text(Locale::De, Key::OverallHeading, "Gesamtansicht");
    i18n.add_text(
        Locale::De,
        Key::WorkingHoursPerWeekHeading,
        "Stunden pro Woche",
    );
    i18n.add_text(
        Locale::De,
        Key::WorkingHoursPerDayHeading,
        "Stunden pro Tag",
    );
    i18n.add_text(Locale::De, Key::ExtraHoursHeading, "Zusatzstunden");
    i18n.add_text(Locale::De, Key::WorkDetailsHeading, "Arbeitsverträge");

    i18n.add_text(Locale::De, Key::Balance, "Stundenkonto");
    i18n.add_text(Locale::De, Key::Required, "Soll");
    i18n.add_text(Locale::De, Key::Overall, "Gesamt");
    i18n.add_text(Locale::De, Key::CarryoverBalance, "Übertrag Vorjahr");
    i18n.add_text(Locale::De, Key::CategoryShiftplan, "Schichtplan");
    i18n.add_text(Locale::De, Key::CategoryExtraWork, "Zusatzarbeit");
    i18n.add_text(Locale::De, Key::CategoryVacation, "Urlaub");
    i18n.add_text(Locale::De, Key::CategoryVacationHours, "Urlaub (Stunden)");
    i18n.add_text(Locale::De, Key::CategoryVacationDays, "Urlaub");
    i18n.add_text(Locale::De, Key::CategorySickLeave, "Krank");
    i18n.add_text(Locale::De, Key::CategoryHolidays, "Feiertage");
    i18n.add_text(Locale::De, Key::CategoryUnavailable, "Nicht verfügbar");
    i18n.add_text(Locale::De, Key::CategoryCustom, "Sonstige");

    i18n.add_text(Locale::De, Key::VacationDaysLabel, "Urlaubstage");
    i18n.add_text(Locale::De, Key::VacationCarryoverLabel, "Vorjahresurlaub");

    i18n.add_text(Locale::De, Key::ShowDetails, "Mehr");
    i18n.add_text(Locale::De, Key::HideDetails, "Weniger");

    i18n.add_text(Locale::De, Key::Hours, "Std.");
    i18n.add_text(Locale::De, Key::Days, "Tage");

    i18n.add_text(Locale::De, Key::AddEntry, "Sonstige Stunden hinzufügen");
    i18n.add_text(
        Locale::De,
        Key::WorkHoursDescription,
        "(Arbeitsstunden, die nicht im Schichtplan enthalten sind)",
    );
    i18n.add_text(
        Locale::De,
        Key::UnavailableDescription,
        "(Stunden, die nicht das Zeitkonto beeinflussen aber dem Schichtplanner zeigen, dass du nicht verfügbar bist)",
    );
    i18n.add_text(Locale::De, Key::ActionsLabel, "Mehr");
    i18n.add_text(
        Locale::De,
        Key::ShowFullYearLabel,
        "Bericht des vollständigen Jahres",
    );
    i18n.add_text(
        Locale::De,
        Key::ShowUntilNowLabel,
        "Bericht bis zur heutigen Woche",
    );
    i18n.add_text(
        Locale::De,
        Key::AddWorkDetailsLabel,
        "Arbeitvertrag hinzufügen",
    );
    i18n.add_text(
        Locale::De,
        Key::CurrentWeekNote,
        "Nur Daten bis zur aktuellen Woche werden angezeigt.",
    );

    // Add extra hours form
    i18n.add_text(
        Locale::De,
        Key::AddExtraHoursFormTitle,
        "Extra Stunden hinzufügen",
    );
    i18n.add_text(Locale::De, Key::AmountOfHours, "Anzahl der Stunden");
    i18n.add_text(Locale::De, Key::AmountOfDays, "Anzahl der Tage");
    i18n.add_text(Locale::De, Key::Category, "Kategorie");
    i18n.add_text(Locale::De, Key::Description, "Beschreibung");
    i18n.add_text(Locale::De, Key::When, "Wann");
    i18n.add_text(Locale::De, Key::Submit, "Hinzufügen");
    i18n.add_text(Locale::De, Key::Cancel, "Abbrechen");

    i18n.add_text(
        Locale::De,
        Key::AddExtraHoursChoiceTitle,
        "Bitte Kategorie auswählen",
    );
    i18n.add_text(Locale::De, Key::AddVacationTitle, "Urlaub hinzufügen");
    i18n.add_text(Locale::De, Key::AddHolidaysTitle, "Feiertag hinzufügen");
    i18n.add_text(
        Locale::De,
        Key::AddSickLeaveTitle,
        "Krankheitstage hinzufügen",
    );
    i18n.add_text(Locale::De, Key::WeekLabel, "Woche");
    i18n.add_text(Locale::De, Key::FullWeekLabel, "Ganze Woche");

    // Non-prod warnings
    i18n.add_text(
        Locale::De,
        Key::NonProdWarning,
        "Das ist nur eine Seite zum Testen❗",
    );
    i18n.add_text(Locale::De, Key::NonProdWarningDetails,
        "Diese Seite darf nicht produktiv genutzt werden! Sie kann Bugs enthalten und Daten können jederzeit zurückgesetzt werden und verloren gehen!");

    // Not authenticated page
    i18n.add_text(Locale::De, Key::WelcomeTitle, "Willkommen zu Shifty!");
    i18n.add_text(Locale::De, Key::PleaseLogin, "Hier klicken zum Einloggen.");
    i18n.add_text(
        Locale::De,
        Key::PleaseChoose,
        "Bitte wähle eine Ansicht von der oberen Leiste.",
    );

    // Employee work details form
    i18n.add_text(
        Locale::De,
        Key::AddWorkDetailsFormTitle,
        "Arbeitsvertrag von {name}",
    );
    i18n.add_text(Locale::De, Key::FromLabel, "Von");
    i18n.add_text(Locale::De, Key::ToLabel, "Bis");
    i18n.add_text(Locale::De, Key::WorkdaysLabel, "Wochentage");
    i18n.add_text(
        Locale::De,
        Key::ExpectedHoursPerWeekLabel,
        "Wochenarbeitsstunden",
    );
    i18n.add_text(Locale::De, Key::ExpectedHours, "Sollstunden");
    i18n.add_text(Locale::De, Key::DaysPerWeekLabel, "Arbeitstage pro Woche");
    i18n.add_text(
        Locale::De,
        Key::VacationEntitlementsPerYearLabel,
        "Urlaubsanspruch im Jahr",
    );
    i18n.add_text(Locale::De, Key::HolidaysInHoursLabel, "Feiertag in Stunden");
    i18n.add_text(
        Locale::De,
        Key::WorkdaysInHoursLabel,
        "Arbeitstag in Stunden",
    );

    // Slot edit
    i18n.add_text(Locale::De, Key::SlotEditTitle, "Slot bearbeiten");
    i18n.add_text(Locale::De, Key::SlotNewTitle, "Neuen Slot erstellen");
    i18n.add_text(
        Locale::De,
        Key::SlotEditExplanation,
        "Diese Änderungen werden ab der Kalenderwoche {week}/{year} angewendet.  Vorherige Woche bleiben unverändert.",
    );
    i18n.add_text(
        Locale::De,
        Key::SlotEditValidUntilExplanation,
        "Die Änderungen werden nur bis zum {date} angewendet.  Nachfolgende Wochen bleiben unverändert.",
    );
    i18n.add_text(Locale::De, Key::MinPersonsLabel, "Benötigte Personen");
    i18n.add_text(Locale::De, Key::WeekdayLabel, "Wochentag");
    i18n.add_text(Locale::De, Key::SaveLabel, "Speichern");
    i18n.add_text(Locale::De, Key::CancelLabel, "Abbrechen");
    i18n.add_text(
        Locale::De,
        Key::SlotEditSaveError,
        "Fehler beim Speichern",
    );

    // Custom extra hours management
    i18n.add_text(Locale::De, Key::CustomExtraHoursManagement, "Verwaltung benutzerdefinierter Zusatzstunden");
    i18n.add_text(Locale::De, Key::Name, "Name");
    i18n.add_text(Locale::De, Key::ModifiesBalance, "Ändert Saldo");
    i18n.add_text(Locale::De, Key::Actions, "Aktionen");
    i18n.add_text(Locale::De, Key::AddNew, "Neu hinzufügen");
    i18n.add_text(Locale::De, Key::Save, "Speichern");
    i18n.add_text(Locale::De, Key::Edit, "Bearbeiten");
    i18n.add_text(Locale::De, Key::Delete, "Löschen");
    i18n.add_text(Locale::De, Key::ConfirmDelete, "Löschen bestätigen");

    // Billing period management
    i18n.add_text(Locale::De, Key::BillingPeriods, "Abrechnungszeiträume");
    i18n.add_text(Locale::De, Key::BillingPeriodDetails, "Abrechnungszeitraum Details");
    i18n.add_text(Locale::De, Key::CreateNewBillingPeriod, "➕ Neuen Abrechnungszeitraum erstellen");
    i18n.add_text(Locale::De, Key::BillingPeriod, "Abrechnungszeitraum");
    i18n.add_text(Locale::De, Key::StartDate, "Startdatum");
    i18n.add_text(Locale::De, Key::EndDate, "Enddatum");
    i18n.add_text(Locale::De, Key::CreatedAt, "Erstellt am");
    i18n.add_text(Locale::De, Key::CreatedBy, "Erstellt von");
    i18n.add_text(Locale::De, Key::DeletedAt, "Gelöscht am");
    i18n.add_text(Locale::De, Key::DeletedBy, "Gelöscht von");
    i18n.add_text(Locale::De, Key::Active, "Aktiv");
    i18n.add_text(Locale::De, Key::Deleted, "Gelöscht");
    i18n.add_text(Locale::De, Key::SalesPersons, "Verkäufer");
    i18n.add_text(Locale::De, Key::BasicInformation, "Grundinformationen");
    i18n.add_text(Locale::De, Key::LoadingBillingPeriods, "Lade Abrechnungszeiträume...");
    i18n.add_text(Locale::De, Key::LoadingBillingPeriodDetails, "Lade Abrechnungszeitraum Details...");
    i18n.add_text(Locale::De, Key::CreateBillingPeriod, "Abrechnungszeitraum erstellen");
    i18n.add_text(Locale::De, Key::Period, "Zeitraum");
    i18n.add_text(Locale::De, Key::NoSalesPersonsInBillingPeriod, "Keine Verkäufer in diesem Abrechnungszeitraum.");
    i18n.add_text(Locale::De, Key::SalesPersonsIncluded, "{count} Verkäufer enthalten");
    i18n.add_text(Locale::De, Key::FilterSalesPersonsByName, "Verkäufer nach Name filtern...");
    i18n.add_text(Locale::De, Key::NoSalesPersonsMatchFilter, "Keine Verkäufer entsprechen dem Filter '{filter}'.");
    i18n.add_text(Locale::De, Key::ShowActive, "Aktiv");
    i18n.add_text(Locale::De, Key::ShowInactive, "Inaktive anzeigen");
    i18n.add_text(Locale::De, Key::ShowPaid, "Bezahlt");
    i18n.add_text(Locale::De, Key::ShowUnpaid, "Unbezahlte anzeigen");
    i18n.add_text(Locale::De, Key::Values, "Werte");
    i18n.add_text(Locale::De, Key::Delta, "Unterschied");
    i18n.add_text(Locale::De, Key::YtdFrom, "YTD Von");
    i18n.add_text(Locale::De, Key::YtdTo, "YTD Bis");
    i18n.add_text(Locale::De, Key::FullYear, "Ganzes Jahr");
    i18n.add_text(Locale::De, Key::InvalidBillingPeriodId, "Ungültige Abrechnungszeitraum ID");
    i18n.add_text(Locale::De, Key::SelectEndDateForNewBillingPeriod, "Wählen Sie das Enddatum für den neuen Abrechnungszeitraum. Das Startdatum wird automatisch berechnet.");

    // Text templates
    i18n.add_text(Locale::De, Key::TextTemplateManagement, "Textvorlagen");
    i18n.add_text(Locale::De, Key::TemplateType, "Vorlagentyp");
    i18n.add_text(Locale::De, Key::TemplateText, "Vorlagentext");
    i18n.add_text(Locale::De, Key::AddNewTemplate, "Neue Vorlage hinzufügen");
    i18n.add_text(Locale::De, Key::EditTemplate, "Vorlage bearbeiten");
    i18n.add_text(Locale::De, Key::CustomReports, "Benutzerdefinierte Berichte");
    i18n.add_text(Locale::De, Key::GenerateReport, "Bericht generieren");
    i18n.add_text(Locale::De, Key::SelectTemplate, "Vorlage auswählen");
    i18n.add_text(Locale::De, Key::GeneratingReport, "Generiere...");
    i18n.add_text(Locale::De, Key::GeneratedReport, "Generierter Bericht");
    i18n.add_text(Locale::De, Key::CreateNewTemplate, "Neue Vorlage erstellen");
    i18n.add_text(Locale::De, Key::Saving, "Speichere...");
    i18n.add_text(Locale::De, Key::TemplateName, "Vorlagenname");

    // User management
    i18n.add_text(Locale::De, Key::UserManagement, "Benutzerverwaltung");
    i18n.add_text(Locale::De, Key::UserDetails, "Benutzer Details");
    i18n.add_text(Locale::De, Key::SalesPersonDetails, "Verkäufer Details");
    i18n.add_text(Locale::De, Key::Users, "Benutzer");
    i18n.add_text(Locale::De, Key::UsersCount, "{count} Benutzer");
    i18n.add_text(Locale::De, Key::SalesPersonsCount, "{count} Personen");
    i18n.add_text(Locale::De, Key::NoUsersFound, "Keine Benutzer gefunden");
    i18n.add_text(Locale::De, Key::AddFirstUserBelow, "Fügen Sie Ihren ersten Benutzer unten hinzu");
    i18n.add_text(Locale::De, Key::NoSalesPersonsFound, "Keine Verkäufer gefunden");
    i18n.add_text(Locale::De, Key::CreateFirstSalesPersonBelow, "Erstellen Sie Ihren ersten Verkäufer unten");
    i18n.add_text(Locale::De, Key::DeleteUser, "Benutzer löschen");
    i18n.add_text(Locale::De, Key::AddNewUser, "Neuen Benutzer hinzufügen");
    i18n.add_text(Locale::De, Key::CreateUser, "Benutzer erstellen");
    i18n.add_text(Locale::De, Key::CreateNewSalesPerson, "Neuen Verkäufer erstellen");
    i18n.add_text(Locale::De, Key::ManageRolesAndPermissions, "Rollen und Berechtigungen für diesen Benutzer verwalten.");
    i18n.add_text(Locale::De, Key::RoleAssignments, "Rollenzuweisungen");
    i18n.add_text(Locale::De, Key::RolesCount, "{assigned} von {total} Rollen");
    i18n.add_text(Locale::De, Key::NoRolesAvailable, "Keine Rollen verfügbar");
    i18n.add_text(Locale::De, Key::ContactAdministratorForRoles, "Wenden Sie sich an Ihren Administrator, um Rollen einzurichten");
    i18n.add_text(Locale::De, Key::BackToUserManagement, "Zurück zur Benutzerverwaltung");
    i18n.add_text(Locale::De, Key::ShiftplanColor, "Schichtplan Farbe");
    i18n.add_text(Locale::De, Key::ColorPreview, "Farbvorschau");
    i18n.add_text(Locale::De, Key::Settings, "Einstellungen");
    i18n.add_text(Locale::De, Key::ThisPersonReceivesPayment, "Diese Person erhält Bezahlung");
    i18n.add_text(Locale::De, Key::ThisPersonIsInactive, "Diese Person ist inaktiv");
    i18n.add_text(Locale::De, Key::UserAccount, "Benutzerkonto");
    i18n.add_text(Locale::De, Key::ConnectUserAccount, "Benutzerkonto verbinden");
    i18n.add_text(Locale::De, Key::SaveChanges, "Änderungen speichern");
    i18n.add_text(Locale::De, Key::LoadingSalesPersonDetails, "Lade Verkäufer Details...");
    i18n.add_text(Locale::De, Key::SalesPersonSavedSuccessfully, "Verkäufer erfolgreich gespeichert!");
    i18n.add_text(Locale::De, Key::EditSalesPersonInformation, "Verkäufer Informationen bearbeiten");
    i18n.add_text(Locale::De, Key::CreateNewSalesPersonTitle, "Neuen Verkäufer erstellen");
    i18n.add_text(Locale::De, Key::Paid, "Bezahlt");
    i18n.add_text(Locale::De, Key::Inactive, "Inaktiv");
    i18n.add_text(Locale::De, Key::Login, "Anmelden");
    i18n.add_text(Locale::De, Key::LogoutUser, "Abmelden {user}");
    i18n.add_text(Locale::De, Key::ShiftplanReport, "Schichtplan Bericht");
    i18n.add_text(Locale::De, Key::GenerateShiftplanReport, "Schichtplan Bericht generieren");
    i18n.add_text(Locale::De, Key::ShiftplanReportGenerated, "Schichtplan Bericht generiert");
    i18n.add_text(Locale::De, Key::CopyToClipboard, "In Zwischenablage kopieren");
    i18n.add_text(Locale::De, Key::CopiedToClipboard, "In Zwischenablage kopiert!");
    i18n.add_text(Locale::De, Key::CopyFailed, "Fehler beim Kopieren in die Zwischenablage");
}
