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
    i18n.add_text(Locale::En, Key::YearOverview, "Year Overview");
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
    i18n.add_text(
        Locale::En,
        Key::ConflictBookingsHeader,
        "Invalid booked slots",
    );

    // Weekly overview page
    i18n.add_text(Locale::En, Key::WeeklyOverviewTitle, "Weekly Overview");
    i18n.add_text(
        Locale::En,
        Key::AvailableRequiredHours,
        "Available / Requested Hours",
    );
    i18n.add_text(Locale::En, Key::MissingHours, "Difference");

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
    i18n.add_text(Locale::En, Key::WorkDetailsHeading, "Work contracts");
    i18n.add_text(Locale::En, Key::ExtraHoursHeading, "Extra hours");

    i18n.add_text(Locale::En, Key::Balance, "Balance");
    i18n.add_text(Locale::En, Key::Required, "Planned");
    i18n.add_text(Locale::En, Key::Overall, "Actual");
    i18n.add_text(Locale::En, Key::CarryoverBalance, "Carryover balance");
    i18n.add_text(Locale::En, Key::CategoryShiftplan, "Shiftplan");
    i18n.add_text(Locale::En, Key::CategoryExtraWork, "Extra work");
    i18n.add_text(Locale::En, Key::CategoryVacation, "Vacation");
    i18n.add_text(Locale::En, Key::CategorySickLeave, "Sick leave");
    i18n.add_text(Locale::En, Key::CategoryHolidays, "Holiday");
    i18n.add_text(Locale::En, Key::CategoryUnavailable, "Unavailable");

    i18n.add_text(Locale::En, Key::VacationDaysLabel, "Vacation days");
    i18n.add_text(
        Locale::En,
        Key::VacationCarryoverLabel,
        "Open vacation days from last year",
    );

    i18n.add_text(Locale::En, Key::ShowDetails, "More");
    i18n.add_text(Locale::En, Key::HideDetails, "Less");

    i18n.add_text(Locale::En, Key::Hours, "hours");
    i18n.add_text(Locale::En, Key::Days, "days");

    i18n.add_text(Locale::En, Key::AddEntry, "Add additional hours");
    i18n.add_text(
        Locale::En,
        Key::WorkHoursDescription,
        "(work hours which are not covered by the shiftplan)",
    );
    i18n.add_text(
        Locale::En,
        Key::UnavailableDescription,
        "(Hours which do not affect the hour balance but marks shows the shiftplanner that you are not available)",
    );
    i18n.add_text(Locale::En, Key::ActionsLabel, "Actions");
    i18n.add_text(Locale::En, Key::ShowFullYearLabel, "Show full year");
    i18n.add_text(Locale::En, Key::ShowUntilNowLabel, "Show until now");
    i18n.add_text(Locale::En, Key::AddWorkDetailsLabel, "Add work contract");

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
        "This is a test environment only❗",
    );
    i18n.add_text(Locale::En, Key::NonProdWarningDetails,
        "This page is not intended for production use. It could contain bugs and data can be reverted and lost anytime without warning.");

    // Not authenticated page
    i18n.add_text(Locale::En, Key::WelcomeTitle, "Welcome to Shifty!");
    i18n.add_text(Locale::En, Key::PleaseLogin, "Click here to log in.");
    i18n.add_text(
        Locale::En,
        Key::PleaseChoose,
        "Choose your view from the menu on top of the page.",
    );

    // Employee work details form
    i18n.add_text(
        Locale::En,
        Key::AddWorkDetailsFormTitle,
        "Work contract for {name}",
    );
    i18n.add_text(Locale::En, Key::FromLabel, "From");
    i18n.add_text(Locale::En, Key::ToLabel, "To");
    i18n.add_text(Locale::En, Key::WorkdaysLabel, "Workdays");
    i18n.add_text(
        Locale::En,
        Key::ExpectedHoursPerWeekLabel,
        "Expected hours per week",
    );
    i18n.add_text(Locale::En, Key::DaysPerWeekLabel, "Days per week");
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementsPerYearLabel,
        "Vacation days",
    );
    i18n.add_text(Locale::En, Key::HolidaysInHoursLabel, "Holidays in hours");
    i18n.add_text(Locale::En, Key::WorkdaysInHoursLabel, "Workdays in hours");

    // Slot edit
    i18n.add_text(Locale::En, Key::SlotEditTitle, "Slot Edit");
    i18n.add_text(Locale::En, Key::SlotNewTitle, "Create new slot");
    i18n.add_text(
        Locale::En,
        Key::SlotEditExplanation,
        "These changes will be valid starting from week {week}/{year}.  Previous weeks will not be affected.",
    );
    i18n.add_text(
        Locale::En,
        Key::SlotEditValidUntilExplanation,
        "The changes will be applied until {date}.  Slots in future weeks will not be affected.",
    );
    i18n.add_text(Locale::En, Key::MinPersonsLabel, "Required persons");
    i18n.add_text(Locale::En, Key::WeekdayLabel, "Weekday");
    i18n.add_text(Locale::En, Key::SaveLabel, "Save");
    i18n.add_text(Locale::En, Key::CancelLabel, "Cancel");
    i18n.add_text(
        Locale::En,
        Key::SlotEditSaveError,
        "Could not save.  Please verify that the slot does not overlap with other slots.",
    );
}
