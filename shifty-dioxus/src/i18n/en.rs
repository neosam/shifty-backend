use super::{I18n, Key, Locale};

pub fn add_i18n_en(i18n: &mut I18n<Key, Locale>) {
    i18n.add_locale(Locale::En);
    i18n.add_text(Locale::En, Key::Home, "Home");
    i18n.add_text(Locale::En, Key::About, "About");

    // Add weekdays
    i18n.add_text(Locale::En, Key::Monday, "Monday");
    i18n.add_text(Locale::En, Key::Tuesday, "Tuesday");
    i18n.add_text(Locale::En, Key::Wednesday, "Wednesday");
    i18n.add_text(Locale::En, Key::Thursday, "Thursday");
    i18n.add_text(Locale::En, Key::Friday, "Friday");
    i18n.add_text(Locale::En, Key::Saturday, "Saturday");
    i18n.add_text(Locale::En, Key::Sunday, "Sunday");

    // Top bar
    i18n.add_text(Locale::En, Key::Shiftplan, "Shiftplan");
    i18n.add_text(Locale::En, Key::Employees, "Employees");
    i18n.add_text(Locale::En, Key::MyTime, "My Time");
    i18n.add_text(Locale::En, Key::Logout, "Logout");

    // Shiftplan
    i18n.add_text(
        Locale::En,
        Key::ShiftplanCalendarWeek,
        "{week}/{year} - from {date}",
    );
    i18n.add_text(Locale::En, Key::ShiftplanTakeLastWeek, "Add last week");
    i18n.add_text(Locale::En, Key::ShiftplanEditAs, "You edit:");
    i18n.add_text(Locale::En, Key::ShiftplanYouAre, "You are ");

    // Employee report
    i18n.add_text(Locale::En, Key::OverallHeading, "Overall");
    i18n.add_text(
        Locale::En,
        Key::WorkingHoursPerWeekHeading,
        "Working hours per week",
    );
    i18n.add_text(
        Locale::En,
        Key::WorkingHoursPerDayHeading,
        "Working hours per day",
    );
    i18n.add_text(Locale::En, Key::ExtraHoursHeading, "Extra hours");

    i18n.add_text(Locale::En, Key::Balance, "Balance");
    i18n.add_text(Locale::En, Key::Required, "Planned");
    i18n.add_text(Locale::En, Key::Overall, "Actual");
    i18n.add_text(Locale::En, Key::CategoryShiftplan, "Shiftplan");
    i18n.add_text(Locale::En, Key::CategoryExtraWork, "Extra work");
    i18n.add_text(Locale::En, Key::CategoryVacation, "Vacation");
    i18n.add_text(Locale::En, Key::CategorySickLeave, "Sick leave");
    i18n.add_text(Locale::En, Key::CategoryHolidays, "Holiday");

    i18n.add_text(Locale::En, Key::ShowDetails, "More");
    i18n.add_text(Locale::En, Key::HideDetails, "Less");

    i18n.add_text(Locale::En, Key::Hours, "hours");

    i18n.add_text(Locale::En, Key::AddEntry, "Add entry");
    i18n.add_text(
        Locale::En,
        Key::WorkHoursDescription,
        "(work hours which are not covered by the shiftplan)",
    );

    // Add extra hours form
    i18n.add_text(Locale::En, Key::AddExtraHoursFormTitle, "Add extra hours");
    i18n.add_text(Locale::En, Key::Category, "Category");
    i18n.add_text(Locale::En, Key::AmountOfHours, "Amount of hours");
    i18n.add_text(Locale::En, Key::Description, "Description");
    i18n.add_text(Locale::En, Key::When, "When");
    i18n.add_text(Locale::En, Key::Submit, "Submit");
    i18n.add_text(Locale::En, Key::Cancel, "Cancel");

    i18n.add_text(
        Locale::En,
        Key::AddExtraHoursChoiceTitle,
        "Choose category to add",
    );
    i18n.add_text(Locale::En, Key::AddVacationTitle, "Add vacation");
    i18n.add_text(Locale::En, Key::AddHolidaysTitle, "Add holidays");
    i18n.add_text(Locale::En, Key::AddSickLeaveTitle, "Add sick leave");

    i18n.add_text(Locale::En, Key::WeekLabel, "Week");
    i18n.add_text(Locale::En, Key::FullWeekLabel, "Full week");

    // Non-prod warnings
    i18n.add_text(
        Locale::En,
        Key::NonProdWarning,
        "This is a non-production environment",
    );
    i18n.add_text(Locale::En, Key::NonProdWarningDetails,
        "This page is not inended for production use. It could contain bugs and data can be reverted and lost anytime without warning.");
}
