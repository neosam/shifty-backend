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
    i18n.add_text(
        Locale::En,
        Key::PersonalCalendarExport,
        "Personal calendar export (iCal)",
    );
    i18n.add_text(
        Locale::En,
        Key::UnsufficientlyBookedCalendarExport,
        "Unsufficiently booked slots calendar export (iCal)",
    );
    i18n.add_text(Locale::En, Key::WeekMessage, "Week Message");

    // Weekly overview page
    i18n.add_text(Locale::En, Key::WeeklyOverviewTitle, "Weekly Overview");
    i18n.add_text(Locale::En, Key::PaidVolunteer, "Paid / Volunteer");
    i18n.add_text(
        Locale::En,
        Key::AvailableRequiredHours,
        "Available / Required",
    );
    i18n.add_text(Locale::En, Key::MissingHours, "Difference");
    i18n.add_text(Locale::En, Key::UnsavedChanges, "Unsaved changes");

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
    i18n.add_text(Locale::En, Key::CategoryVacationHours, "Vacation (hours)");
    i18n.add_text(Locale::En, Key::CategoryVacationDays, "Vacation");
    i18n.add_text(Locale::En, Key::CategorySickLeave, "Sick leave");
    i18n.add_text(Locale::En, Key::CategoryHolidays, "Holiday");
    i18n.add_text(Locale::En, Key::CategoryUnavailable, "Unavailable");
    i18n.add_text(Locale::En, Key::CategoryCustom, "Custom");

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
    i18n.add_text(
        Locale::En,
        Key::CurrentWeekNote,
        "Only show data until the current week.",
    );

    // Add extra hours form
    i18n.add_text(Locale::En, Key::AddExtraHoursFormTitle, "Add extra hours");
    i18n.add_text(Locale::En, Key::Category, "Category");
    i18n.add_text(Locale::En, Key::AmountOfHours, "Amount of hours");
    i18n.add_text(Locale::En, Key::AmountOfDays, "Amount of days");
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
    i18n.add_text(Locale::En, Key::ExpectedHours, "Expected Hours");
    i18n.add_text(Locale::En, Key::DaysPerWeekLabel, "Days per week");
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementsPerYearLabel,
        "Vacation days",
    );
    i18n.add_text(Locale::En, Key::DynamicHourLabel, "Dynamic hours");
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
    i18n.add_text(Locale::En, Key::SlotEditSaveError, "Error saving slot");

    // Custom extra hours management
    i18n.add_text(
        Locale::En,
        Key::CustomExtraHoursManagement,
        "Custom Extra Hours Management",
    );
    i18n.add_text(Locale::En, Key::Name, "Name");
    i18n.add_text(Locale::En, Key::ModifiesBalance, "Modifies Balance");
    i18n.add_text(Locale::En, Key::Actions, "Actions");
    i18n.add_text(Locale::En, Key::AddNew, "Add New");
    i18n.add_text(Locale::En, Key::Save, "Save");
    i18n.add_text(Locale::En, Key::Edit, "Edit");
    i18n.add_text(Locale::En, Key::Delete, "Delete");
    i18n.add_text(Locale::En, Key::ConfirmDelete, "Confirm Delete");

    // Billing period management
    i18n.add_text(Locale::En, Key::BillingPeriods, "Billing Periods");
    i18n.add_text(
        Locale::En,
        Key::BillingPeriodDetails,
        "Billing Period Details",
    );
    i18n.add_text(
        Locale::En,
        Key::CreateNewBillingPeriod,
        "➕ Create New Billing Period",
    );
    i18n.add_text(Locale::En, Key::BillingPeriod, "Billing Period");
    i18n.add_text(Locale::En, Key::StartDate, "Start Date");
    i18n.add_text(Locale::En, Key::EndDate, "End Date");
    i18n.add_text(Locale::En, Key::CreatedAt, "Created At");
    i18n.add_text(Locale::En, Key::CreatedBy, "Created By");
    i18n.add_text(Locale::En, Key::DeletedAt, "Deleted At");
    i18n.add_text(Locale::En, Key::DeletedBy, "Deleted By");
    i18n.add_text(Locale::En, Key::Active, "Active");
    i18n.add_text(Locale::En, Key::Deleted, "Deleted");
    i18n.add_text(Locale::En, Key::SalesPersons, "Sales Persons");
    i18n.add_text(Locale::En, Key::BasicInformation, "Basic Information");
    i18n.add_text(
        Locale::En,
        Key::LoadingBillingPeriods,
        "Loading billing periods...",
    );
    i18n.add_text(
        Locale::En,
        Key::LoadingBillingPeriodDetails,
        "Loading billing period details...",
    );
    i18n.add_text(
        Locale::En,
        Key::CreateBillingPeriod,
        "Create Billing Period",
    );
    i18n.add_text(Locale::En, Key::Period, "Period");
    i18n.add_text(
        Locale::En,
        Key::NoSalesPersonsInBillingPeriod,
        "No sales persons in this billing period.",
    );
    i18n.add_text(
        Locale::En,
        Key::SalesPersonsIncluded,
        "{count} sales persons included",
    );
    i18n.add_text(
        Locale::En,
        Key::FilterSalesPersonsByName,
        "Filter sales persons by name...",
    );
    i18n.add_text(
        Locale::En,
        Key::NoSalesPersonsMatchFilter,
        "No sales persons match the filter '{filter}'.",
    );
    i18n.add_text(Locale::En, Key::ShowActive, "Active");
    i18n.add_text(Locale::En, Key::ShowInactive, "Show Inactive");
    i18n.add_text(Locale::En, Key::ShowPaid, "Paid");
    i18n.add_text(Locale::En, Key::ShowUnpaid, "Show Unpaid");
    i18n.add_text(Locale::En, Key::Values, "Values");
    i18n.add_text(Locale::En, Key::Delta, "Delta");
    i18n.add_text(Locale::En, Key::YtdFrom, "YTD From");
    i18n.add_text(Locale::En, Key::YtdTo, "YTD To");
    i18n.add_text(Locale::En, Key::FullYear, "Full Year");
    i18n.add_text(
        Locale::En,
        Key::InvalidBillingPeriodId,
        "Invalid billing period id",
    );
    i18n.add_text(Locale::En, Key::SelectEndDateForNewBillingPeriod, "Select the end date for the new billing period. The start date will be calculated automatically.");

    // Text templates
    i18n.add_text(
        Locale::En,
        Key::TextTemplateManagement,
        "Text Template Management",
    );
    i18n.add_text(Locale::En, Key::TemplateType, "Template Type");
    i18n.add_text(Locale::En, Key::TemplateText, "Template Text");
    i18n.add_text(Locale::En, Key::AddNewTemplate, "Add New Template");
    i18n.add_text(Locale::En, Key::EditTemplate, "Edit Template");
    i18n.add_text(Locale::En, Key::CustomReports, "Custom Reports");
    i18n.add_text(Locale::En, Key::GenerateReport, "Generate Report");
    i18n.add_text(Locale::En, Key::SelectTemplate, "Select Template");
    i18n.add_text(Locale::En, Key::GeneratingReport, "Generating...");
    i18n.add_text(Locale::En, Key::GeneratedReport, "Generated Report");
    i18n.add_text(Locale::En, Key::CreateNewTemplate, "Create New Template");
    i18n.add_text(Locale::En, Key::Saving, "Saving...");
    i18n.add_text(Locale::En, Key::TemplateName, "Template Name");

    // User management
    i18n.add_text(Locale::En, Key::UserManagement, "User Management");
    i18n.add_text(Locale::En, Key::UserDetails, "User Details");
    i18n.add_text(Locale::En, Key::SalesPersonDetails, "Sales Person Details");
    i18n.add_text(Locale::En, Key::Users, "Users");
    i18n.add_text(Locale::En, Key::UsersCount, "{count} users");
    i18n.add_text(Locale::En, Key::SalesPersonsCount, "{count} persons");
    i18n.add_text(Locale::En, Key::NoUsersFound, "No users found");
    i18n.add_text(
        Locale::En,
        Key::AddFirstUserBelow,
        "Add your first user below",
    );
    i18n.add_text(
        Locale::En,
        Key::NoSalesPersonsFound,
        "No sales persons found",
    );
    i18n.add_text(
        Locale::En,
        Key::CreateFirstSalesPersonBelow,
        "Create your first sales person below",
    );
    i18n.add_text(Locale::En, Key::DeleteUser, "Delete user");
    i18n.add_text(Locale::En, Key::AddNewUser, "Add New User");
    i18n.add_text(Locale::En, Key::CreateUser, "Create User");
    i18n.add_text(
        Locale::En,
        Key::CreateNewSalesPerson,
        "Create New Sales Person",
    );
    i18n.add_text(
        Locale::En,
        Key::ManageRolesAndPermissions,
        "Manage roles and permissions for this user.",
    );
    i18n.add_text(Locale::En, Key::RoleAssignments, "Role Assignments");
    i18n.add_text(Locale::En, Key::RolesCount, "{assigned} of {total} roles");
    i18n.add_text(Locale::En, Key::NoRolesAvailable, "No roles available");
    i18n.add_text(
        Locale::En,
        Key::ContactAdministratorForRoles,
        "Contact your administrator to set up roles",
    );
    i18n.add_text(
        Locale::En,
        Key::BackToUserManagement,
        "Back to User Management",
    );
    i18n.add_text(Locale::En, Key::ShiftplanColor, "Shiftplan Color");
    i18n.add_text(Locale::En, Key::ColorPreview, "Color preview");
    i18n.add_text(Locale::En, Key::Settings, "Settings");
    i18n.add_text(
        Locale::En,
        Key::ThisPersonReceivesPayment,
        "This person receives payment",
    );
    i18n.add_text(
        Locale::En,
        Key::ThisPersonIsInactive,
        "This person is inactive",
    );
    i18n.add_text(Locale::En, Key::UserAccount, "User Account");
    i18n.add_text(Locale::En, Key::ConnectUserAccount, "Connect User Account");
    // User invitations
    i18n.add_text(Locale::En, Key::UserInvitations, "User Invitations");
    i18n.add_text(Locale::En, Key::GenerateInvitation, "Generate Invitation");
    i18n.add_text(Locale::En, Key::InvitationLink, "Invitation Link");
    i18n.add_text(Locale::En, Key::RevokeInvitation, "Revoke");
    i18n.add_text(Locale::En, Key::RevokeSession, "Revoke Session");
    i18n.add_text(Locale::En, Key::InvitationStatus, "Status");
    i18n.add_text(Locale::En, Key::ExpirationHours, "Expiration (hours)");
    i18n.add_text(Locale::En, Key::CopyToClipboard, "Copy");
    i18n.add_text(Locale::En, Key::InvitationCopied, "Copied!");
    i18n.add_text(Locale::En, Key::Valid, "Valid");
    i18n.add_text(Locale::En, Key::Expired, "Expired");
    i18n.add_text(Locale::En, Key::Redeemed, "Redeemed");
    i18n.add_text(Locale::En, Key::SessionRevoked, "Session Revoked");
    i18n.add_text(Locale::En, Key::NoInvitationsFound, "No invitations found");
    i18n.add_text(
        Locale::En,
        Key::GenerateFirstInvitation,
        "Generate the first invitation below",
    );
    i18n.add_text(Locale::En, Key::InvitationsCount, "{count} invitations");
    i18n.add_text(
        Locale::En,
        Key::GenerateNewInvitation,
        "Generate New Invitation",
    );
    i18n.add_text(
        Locale::En,
        Key::OptionalExpirationHours,
        "Expiration (hours)",
    );
    i18n.add_text(Locale::En, Key::SaveChanges, "Save Changes");
    i18n.add_text(
        Locale::En,
        Key::LoadingSalesPersonDetails,
        "Loading sales person details...",
    );
    i18n.add_text(
        Locale::En,
        Key::SalesPersonSavedSuccessfully,
        "Sales person saved successfully!",
    );
    i18n.add_text(
        Locale::En,
        Key::EditSalesPersonInformation,
        "Edit sales person information",
    );
    i18n.add_text(
        Locale::En,
        Key::CreateNewSalesPersonTitle,
        "Create new sales person",
    );
    i18n.add_text(Locale::En, Key::Paid, "Paid");
    i18n.add_text(Locale::En, Key::Volunteer, "Volunteer");
    i18n.add_text(Locale::En, Key::Inactive, "Inactive");
    i18n.add_text(Locale::En, Key::Login, "Login");
    i18n.add_text(Locale::En, Key::LogoutUser, "Logout {user}");
    i18n.add_text(Locale::En, Key::ShiftplanReport, "Shiftplan Report");
    i18n.add_text(
        Locale::En,
        Key::GenerateShiftplanReport,
        "Generate Shiftplan Report",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanReportGenerated,
        "Shiftplan Report Generated",
    );
    i18n.add_text(Locale::En, Key::CopyToClipboard, "Copy to Clipboard");
    i18n.add_text(Locale::En, Key::CopiedToClipboard, "Copied to clipboard!");
    i18n.add_text(Locale::En, Key::CopyFailed, "Failed to copy to clipboard");
}
