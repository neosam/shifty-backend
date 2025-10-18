use super::{I18n, Key, Locale};

pub fn add_i18n_cs(i18n: &mut I18n<Key, Locale>) {
    i18n.add_locale(Locale::Cs);
    i18n.add_text(Locale::Cs, Key::Home, "Domů");
    i18n.add_text(Locale::Cs, Key::About, "O aplikaci");

    // Add weekdays
    i18n.add_text(Locale::Cs, Key::Monday, "Pondělí");
    i18n.add_text(Locale::Cs, Key::Tuesday, "Úterý");
    i18n.add_text(Locale::Cs, Key::Wednesday, "Středa");
    i18n.add_text(Locale::Cs, Key::Thursday, "Čtvrtek");
    i18n.add_text(Locale::Cs, Key::Friday, "Pátek");
    i18n.add_text(Locale::Cs, Key::Saturday, "Sobota");
    i18n.add_text(Locale::Cs, Key::Sunday, "Neděle");

    // Top bar
    i18n.add_text(Locale::Cs, Key::Shiftplan, "Plán směn");
    i18n.add_text(Locale::Cs, Key::Employees, "Zaměstnanci");
    i18n.add_text(Locale::Cs, Key::MyTime, "Moje hodiny");
    i18n.add_text(Locale::Cs, Key::YearOverview, "Roční přehled");
    i18n.add_text(Locale::Cs, Key::Logout, "Odhlásit");

    // Shiftplan
    i18n.add_text(
        Locale::Cs,
        Key::ShiftplanCalendarWeek,
        "{week}/{year} - od {date}",
    );
    i18n.add_text(Locale::Cs, Key::ShiftplanTakeLastWeek, "Přidat minulý týden");
    i18n.add_text(Locale::Cs, Key::ShiftplanEditAs, "Upravujete:");
    i18n.add_text(Locale::Cs, Key::ShiftplanYouAre, "Jste ");
    i18n.add_text(
        Locale::Cs,
        Key::ConflictBookingsHeader,
        "Neplatné rezervace",
    );
    i18n.add_text(
        Locale::Cs,
        Key::PersonalCalendarExport,
        "Export osobního kalendáře (iCal)",
    );
    i18n.add_text(
        Locale::Cs,
        Key::UnsufficientlyBookedCalendarExport,
        "Export kalendáře nedostatečně obsazených směn (iCal)",
    );
    i18n.add_text(Locale::Cs, Key::WeekMessage, "Týdenní zpráva");

    // Weekly overview page
    i18n.add_text(Locale::Cs, Key::WeeklyOverviewTitle, "Týdenní přehled");
    i18n.add_text(
        Locale::Cs,
        Key::AvailableRequiredHours,
        "Dostupné / Požadované hodiny",
    );
    i18n.add_text(Locale::Cs, Key::MissingHours, "Rozdíl");
    i18n.add_text(Locale::Cs, Key::UnsavedChanges, "Neuložené změny");

    // Employee report
    i18n.add_text(Locale::Cs, Key::OverallHeading, "Celkový přehled");
    i18n.add_text(
        Locale::Cs,
        Key::WorkingHoursPerWeekHeading,
        "Pracovní hodiny za týden",
    );
    i18n.add_text(
        Locale::Cs,
        Key::WorkingHoursPerDayHeading,
        "Pracovní hodiny za den",
    );
    i18n.add_text(Locale::Cs, Key::WorkDetailsHeading, "Pracovní smlouvy");
    i18n.add_text(Locale::Cs, Key::ExtraHoursHeading, "Přesčasy");

    i18n.add_text(Locale::Cs, Key::Balance, "Zůstatek");
    i18n.add_text(Locale::Cs, Key::Required, "Plánováno");
    i18n.add_text(Locale::Cs, Key::Overall, "Skutečné");
    i18n.add_text(Locale::Cs, Key::CarryoverBalance, "Převod ze zůstatku");
    i18n.add_text(Locale::Cs, Key::CategoryShiftplan, "Plán směn");
    i18n.add_text(Locale::Cs, Key::CategoryExtraWork, "Přesčasy");
    i18n.add_text(Locale::Cs, Key::CategoryVacation, "Dovolená");
    i18n.add_text(Locale::Cs, Key::CategoryVacationHours, "Dovolená (hodiny)");
    i18n.add_text(Locale::Cs, Key::CategoryVacationDays, "Dovolená");
    i18n.add_text(Locale::Cs, Key::CategorySickLeave, "Nemocenská");
    i18n.add_text(Locale::Cs, Key::CategoryHolidays, "Svátky");
    i18n.add_text(Locale::Cs, Key::CategoryUnavailable, "Nedostupný");
    i18n.add_text(Locale::Cs, Key::CategoryCustom, "Vlastní");

    i18n.add_text(Locale::Cs, Key::VacationDaysLabel, "Dny dovolené");
    i18n.add_text(
        Locale::Cs,
        Key::VacationCarryoverLabel,
        "Nevyčerpaná dovolená z minulého roku",
    );

    i18n.add_text(Locale::Cs, Key::ShowDetails, "Více");
    i18n.add_text(Locale::Cs, Key::HideDetails, "Méně");

    i18n.add_text(Locale::Cs, Key::Hours, "hodiny");
    i18n.add_text(Locale::Cs, Key::Days, "dny");

    i18n.add_text(Locale::Cs, Key::AddEntry, "Přidat další hodiny");
    i18n.add_text(
        Locale::Cs,
        Key::WorkHoursDescription,
        "(pracovní hodiny, které nejsou pokryty plánem směn)",
    );
    i18n.add_text(
        Locale::Cs,
        Key::UnavailableDescription,
        "(Hodiny, které neovlivňují zůstatek hodin, ale ukážou plánovači směn, že nejste k dispozici)",
    );
    i18n.add_text(Locale::Cs, Key::ActionsLabel, "Akce");
    i18n.add_text(Locale::Cs, Key::ShowFullYearLabel, "Zobrazit celý rok");
    i18n.add_text(Locale::Cs, Key::ShowUntilNowLabel, "Zobrazit do teď");
    i18n.add_text(Locale::Cs, Key::AddWorkDetailsLabel, "Přidat pracovní smlouvu");
    i18n.add_text(
        Locale::Cs,
        Key::CurrentWeekNote,
        "Zobrazit pouze data do aktuálního týdne.",
    );

    // Add extra hours form
    i18n.add_text(Locale::Cs, Key::AddExtraHoursFormTitle, "Přidat přesčasy");
    i18n.add_text(Locale::Cs, Key::Category, "Kategorie");
    i18n.add_text(Locale::Cs, Key::AmountOfHours, "Počet hodin");
    i18n.add_text(Locale::Cs, Key::AmountOfDays, "Počet dní");
    i18n.add_text(Locale::Cs, Key::Description, "Popis");
    i18n.add_text(Locale::Cs, Key::When, "Kdy");
    i18n.add_text(Locale::Cs, Key::Submit, "Odeslat");
    i18n.add_text(Locale::Cs, Key::Cancel, "Zrušit");

    i18n.add_text(
        Locale::Cs,
        Key::AddExtraHoursChoiceTitle,
        "Vyberte kategorii k přidání",
    );
    i18n.add_text(Locale::Cs, Key::AddVacationTitle, "Přidat dovolenou");
    i18n.add_text(Locale::Cs, Key::AddHolidaysTitle, "Přidat svátky");
    i18n.add_text(Locale::Cs, Key::AddSickLeaveTitle, "Přidat nemocenskou");

    i18n.add_text(Locale::Cs, Key::WeekLabel, "Týden");
    i18n.add_text(Locale::Cs, Key::FullWeekLabel, "Celý týden");

    // Non-prod warnings
    i18n.add_text(
        Locale::Cs,
        Key::NonProdWarning,
        "Toto je pouze testovací prostředí❗",
    );
    i18n.add_text(Locale::Cs, Key::NonProdWarningDetails,
        "Tato stránka není určena pro produkční použití. Může obsahovat chyby a data mohou být kdykoliv obnovena a ztracena bez varování.");

    // Not authenticated page
    i18n.add_text(Locale::Cs, Key::WelcomeTitle, "Vítejte v Shifty!");
    i18n.add_text(Locale::Cs, Key::PleaseLogin, "Klikněte zde pro přihlášení.");
    i18n.add_text(
        Locale::Cs,
        Key::PleaseChoose,
        "Vyberte pohled z menu v horní části stránky.",
    );

    // Employee work details form
    i18n.add_text(
        Locale::Cs,
        Key::AddWorkDetailsFormTitle,
        "Pracovní smlouva pro {name}",
    );
    i18n.add_text(Locale::Cs, Key::FromLabel, "Od");
    i18n.add_text(Locale::Cs, Key::ToLabel, "Do");
    i18n.add_text(Locale::Cs, Key::WorkdaysLabel, "Pracovní dny");
    i18n.add_text(
        Locale::Cs,
        Key::ExpectedHoursPerWeekLabel,
        "Očekávané hodiny týdně",
    );
    i18n.add_text(Locale::Cs, Key::ExpectedHours, "Očekávané hodiny");
    i18n.add_text(Locale::Cs, Key::DaysPerWeekLabel, "Dny týdně");
    i18n.add_text(
        Locale::Cs,
        Key::VacationEntitlementsPerYearLabel,
        "Dny dovolené",
    );
    i18n.add_text(Locale::Cs, Key::HolidaysInHoursLabel, "Svátky v hodinách");
    i18n.add_text(Locale::Cs, Key::WorkdaysInHoursLabel, "Pracovní dny v hodinách");

    // Slot edit
    i18n.add_text(Locale::Cs, Key::SlotEditTitle, "Upravit směnu");
    i18n.add_text(Locale::Cs, Key::SlotNewTitle, "Vytvořit novou směnu");
    i18n.add_text(
        Locale::Cs,
        Key::SlotEditExplanation,
        "Tyto změny budou platné od týdne {week}/{year}. Předchozí týdny nebudou ovlivněny.",
    );
    i18n.add_text(
        Locale::Cs,
        Key::SlotEditValidUntilExplanation,
        "Změny budou použity do {date}. Směny v budoucích týdnech nebudou ovlivněny.",
    );
    i18n.add_text(Locale::Cs, Key::MinPersonsLabel, "Požadované osoby");
    i18n.add_text(Locale::Cs, Key::WeekdayLabel, "Den v týdnu");
    i18n.add_text(Locale::Cs, Key::SaveLabel, "Uložit");
    i18n.add_text(Locale::Cs, Key::CancelLabel, "Zrušit");
    i18n.add_text(
        Locale::Cs,
        Key::SlotEditSaveError,
        "Chyba při ukládání směny",
    );

    // Custom extra hours management
    i18n.add_text(Locale::Cs, Key::CustomExtraHoursManagement, "Správa vlastních přesčasů");
    i18n.add_text(Locale::Cs, Key::Name, "Název");
    i18n.add_text(Locale::Cs, Key::ModifiesBalance, "Upravuje zůstatek");
    i18n.add_text(Locale::Cs, Key::Actions, "Akce");
    i18n.add_text(Locale::Cs, Key::AddNew, "Přidat nový");
    i18n.add_text(Locale::Cs, Key::Save, "Uložit");
    i18n.add_text(Locale::Cs, Key::Edit, "Upravit");
    i18n.add_text(Locale::Cs, Key::Delete, "Smazat");
    i18n.add_text(Locale::Cs, Key::ConfirmDelete, "Potvrdit smazání");

    // Billing period management
    i18n.add_text(Locale::Cs, Key::BillingPeriods, "Fakturační období");
    i18n.add_text(Locale::Cs, Key::BillingPeriodDetails, "Detaily fakturačního období");
    i18n.add_text(Locale::Cs, Key::CreateNewBillingPeriod, "➕ Vytvořit nové fakturační období");
    i18n.add_text(Locale::Cs, Key::BillingPeriod, "Fakturační období");
    i18n.add_text(Locale::Cs, Key::StartDate, "Datum zahájení");
    i18n.add_text(Locale::Cs, Key::EndDate, "Datum ukončení");
    i18n.add_text(Locale::Cs, Key::CreatedAt, "Vytvořeno");
    i18n.add_text(Locale::Cs, Key::CreatedBy, "Vytvořil");
    i18n.add_text(Locale::Cs, Key::DeletedAt, "Smazáno");
    i18n.add_text(Locale::Cs, Key::DeletedBy, "Smazal");
    i18n.add_text(Locale::Cs, Key::Active, "Aktivní");
    i18n.add_text(Locale::Cs, Key::Deleted, "Smazáno");
    i18n.add_text(Locale::Cs, Key::SalesPersons, "Prodejci");
    i18n.add_text(Locale::Cs, Key::BasicInformation, "Základní informace");
    i18n.add_text(Locale::Cs, Key::LoadingBillingPeriods, "Načítám fakturační období...");
    i18n.add_text(Locale::Cs, Key::LoadingBillingPeriodDetails, "Načítám detaily fakturačního období...");
    i18n.add_text(Locale::Cs, Key::CreateBillingPeriod, "Vytvořit fakturační období");
    i18n.add_text(Locale::Cs, Key::Period, "Období");
    i18n.add_text(Locale::Cs, Key::NoSalesPersonsInBillingPeriod, "V tomto fakturačním období nejsou žádní prodejci.");
    i18n.add_text(Locale::Cs, Key::SalesPersonsIncluded, "{count} prodejců zahrnuto");
    i18n.add_text(Locale::Cs, Key::FilterSalesPersonsByName, "Filtrovat prodejce podle jména...");
    i18n.add_text(Locale::Cs, Key::NoSalesPersonsMatchFilter, "Žádní prodejci neodpovídají filtru '{filter}'.");
    i18n.add_text(Locale::Cs, Key::ShowActive, "Aktivní");
    i18n.add_text(Locale::Cs, Key::ShowInactive, "Zobrazit neaktivní");
    i18n.add_text(Locale::Cs, Key::ShowPaid, "Placené");
    i18n.add_text(Locale::Cs, Key::ShowUnpaid, "Zobrazit neplacené");
    i18n.add_text(Locale::Cs, Key::Values, "Hodnoty");
    i18n.add_text(Locale::Cs, Key::Delta, "Delta");
    i18n.add_text(Locale::Cs, Key::YtdFrom, "YTD Od");
    i18n.add_text(Locale::Cs, Key::YtdTo, "YTD Do");
    i18n.add_text(Locale::Cs, Key::FullYear, "Celý rok");
    i18n.add_text(Locale::Cs, Key::InvalidBillingPeriodId, "Neplatné ID fakturačního období");
    i18n.add_text(Locale::Cs, Key::SelectEndDateForNewBillingPeriod, "Vyberte datum ukončení pro nové fakturační období. Datum zahájení bude vypočítáno automaticky.");

    // User management
    i18n.add_text(Locale::Cs, Key::UserManagement, "Správa uživatelů");
    i18n.add_text(Locale::Cs, Key::UserDetails, "Detaily uživatele");
    i18n.add_text(Locale::Cs, Key::SalesPersonDetails, "Detaily prodejce");
    i18n.add_text(Locale::Cs, Key::Users, "Uživatelé");
    i18n.add_text(Locale::Cs, Key::UsersCount, "{count} uživatelů");
    i18n.add_text(Locale::Cs, Key::SalesPersonsCount, "{count} osob");
    i18n.add_text(Locale::Cs, Key::NoUsersFound, "Nebyli nalezeni žádní uživatelé");
    i18n.add_text(Locale::Cs, Key::AddFirstUserBelow, "Přidejte svého prvního uživatele níže");
    i18n.add_text(Locale::Cs, Key::NoSalesPersonsFound, "Nebyli nalezeni žádní prodejci");
    i18n.add_text(Locale::Cs, Key::CreateFirstSalesPersonBelow, "Vytvořte svého prvního prodejce níže");
    i18n.add_text(Locale::Cs, Key::DeleteUser, "Smazat uživatele");
    i18n.add_text(Locale::Cs, Key::AddNewUser, "Přidat nového uživatele");
    i18n.add_text(Locale::Cs, Key::CreateUser, "Vytvořit uživatele");
    i18n.add_text(Locale::Cs, Key::CreateNewSalesPerson, "Vytvořit nového prodejce");
    i18n.add_text(Locale::Cs, Key::ManageRolesAndPermissions, "Spravovat role a oprávnění pro tohoto uživatele.");
    i18n.add_text(Locale::Cs, Key::RoleAssignments, "Přiřazení rolí");
    i18n.add_text(Locale::Cs, Key::RolesCount, "{assigned} z {total} rolí");
    i18n.add_text(Locale::Cs, Key::NoRolesAvailable, "Nejsou k dispozici žádné role");
    i18n.add_text(Locale::Cs, Key::ContactAdministratorForRoles, "Kontaktujte svého správce pro nastavení rolí");
    i18n.add_text(Locale::Cs, Key::BackToUserManagement, "Zpět na správu uživatelů");
    i18n.add_text(Locale::Cs, Key::ShiftplanColor, "Barva plánu směn");
    i18n.add_text(Locale::Cs, Key::ColorPreview, "Náhled barvy");
    i18n.add_text(Locale::Cs, Key::Settings, "Nastavení");
    i18n.add_text(Locale::Cs, Key::ThisPersonReceivesPayment, "Tato osoba dostává mzdu");
    i18n.add_text(Locale::Cs, Key::ThisPersonIsInactive, "Tato osoba je neaktivní");
    i18n.add_text(Locale::Cs, Key::UserAccount, "Uživatelský účet");
    i18n.add_text(Locale::Cs, Key::ConnectUserAccount, "Připojit uživatelský účet");
    // User invitations
    i18n.add_text(Locale::Cs, Key::UserInvitations, "Uživatelské pozvánky");
    i18n.add_text(Locale::Cs, Key::GenerateInvitation, "Vytvořit pozvánku");
    i18n.add_text(Locale::Cs, Key::InvitationLink, "Odkaz na pozvánku");
    i18n.add_text(Locale::Cs, Key::RevokeInvitation, "Zrušit");
    i18n.add_text(Locale::Cs, Key::RevokeSession, "Zrušit relaci");
    i18n.add_text(Locale::Cs, Key::InvitationStatus, "Stav");
    i18n.add_text(Locale::Cs, Key::ExpirationHours, "Platnost (hodiny)");
    i18n.add_text(Locale::Cs, Key::InvitationCopied, "Zkopírováno!");
    i18n.add_text(Locale::Cs, Key::Valid, "Platná");
    i18n.add_text(Locale::Cs, Key::Expired, "Vypršela");
    i18n.add_text(Locale::Cs, Key::Redeemed, "Uplatněna");
    i18n.add_text(Locale::Cs, Key::SessionRevoked, "Relace zrušena");
    i18n.add_text(Locale::Cs, Key::NoInvitationsFound, "Žádné pozvánky nenalezeny");
    i18n.add_text(Locale::Cs, Key::GenerateFirstInvitation, "Vytvořte první pozvánku níže");
    i18n.add_text(Locale::Cs, Key::InvitationsCount, "{count} pozvánky");
    i18n.add_text(Locale::Cs, Key::GenerateNewInvitation, "Vytvořit novou pozvánku");
    i18n.add_text(Locale::Cs, Key::OptionalExpirationHours, "Platnost (hodiny)");
    i18n.add_text(Locale::Cs, Key::SaveChanges, "Uložit změny");
    i18n.add_text(Locale::Cs, Key::LoadingSalesPersonDetails, "Načítám detaily prodejce...");
    i18n.add_text(Locale::Cs, Key::SalesPersonSavedSuccessfully, "Prodejce byl úspěšně uložen!");
    i18n.add_text(Locale::Cs, Key::EditSalesPersonInformation, "Upravit informace o prodejci");
    i18n.add_text(Locale::Cs, Key::CreateNewSalesPersonTitle, "Vytvořit nového prodejce");
    i18n.add_text(Locale::Cs, Key::Paid, "Placený");
    i18n.add_text(Locale::Cs, Key::Inactive, "Neaktivní");
    i18n.add_text(Locale::Cs, Key::Login, "Přihlásit se");
    i18n.add_text(Locale::Cs, Key::LogoutUser, "Odhlásit {user}");
    i18n.add_text(Locale::Cs, Key::ShiftplanReport, "Zpráva plánu směn");
    i18n.add_text(Locale::Cs, Key::GenerateShiftplanReport, "Generovat zprávu plánu směn");
    i18n.add_text(Locale::Cs, Key::ShiftplanReportGenerated, "Zpráva plánu směn vygenerována");
    i18n.add_text(Locale::Cs, Key::CopyToClipboard, "Kopírovat do schránky");
    i18n.add_text(Locale::Cs, Key::CopiedToClipboard, "Zkopírováno do schránky!");
    i18n.add_text(Locale::Cs, Key::CopyFailed, "Chyba při kopírování do schránky");
}