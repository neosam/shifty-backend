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
    i18n.add_text(Locale::En, Key::TopBarYouAreLabel, "You are");
    i18n.add_text(Locale::En, Key::TopBarAdminGroupLabel, "Administration");

    // Shiftplan
    i18n.add_text(
        Locale::En,
        Key::ShiftplanCalendarWeek,
        "{week}/{year} - from {date}",
    );
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
    i18n.add_text(Locale::En, Key::ShiftplanFilledOfNeed, "{filled}/{need}");
    i18n.add_text(Locale::En, Key::ShiftplanLastWeek, "Last week");
    i18n.add_text(
        Locale::En,
        Key::ShiftplanCellAddTitle,
        "Add me to this slot",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanCellRemoveTitle,
        "Remove me from this slot",
    );
    i18n.add_text(Locale::En, Key::ShiftplanCreateTitle, "Create shiftplan");
    i18n.add_text(Locale::En, Key::ShiftplanEditTitle, "Edit shiftplan");
    i18n.add_text(
        Locale::En,
        Key::ShiftplanDeleteConfirmTitle,
        "Delete shiftplan",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanDeleteConfirmBody,
        "Are you sure you want to delete shiftplan {name}? This cannot be undone.",
    );
    i18n.add_text(Locale::En, Key::ShiftplanIsPlanningLabel, "Planning only");

    // Booking log
    i18n.add_text(Locale::En, Key::BookingLogTitle, "Booking Log");
    i18n.add_text(Locale::En, Key::BookingLogShow, "Show Booking Log");
    i18n.add_text(Locale::En, Key::BookingLogHide, "Hide Booking Log");
    i18n.add_text(Locale::En, Key::BookingLogLoading, "Loading...");
    i18n.add_text(Locale::En, Key::BookingLogDay, "Day");
    i18n.add_text(Locale::En, Key::BookingLogName, "Name");
    i18n.add_text(Locale::En, Key::BookingLogTime, "Time");
    i18n.add_text(Locale::En, Key::BookingLogCreated, "Created");
    i18n.add_text(Locale::En, Key::BookingLogCreatedBy, "Created By");
    i18n.add_text(Locale::En, Key::BookingLogDeleted, "Deleted");
    i18n.add_text(Locale::En, Key::BookingLogDeletedBy, "Deleted By");
    i18n.add_text(Locale::En, Key::BookingLogFilterName, "Filter by Name");
    i18n.add_text(Locale::En, Key::BookingLogFilterDay, "Filter by Day");
    i18n.add_text(Locale::En, Key::BookingLogFilterStatus, "Filter by Status");
    i18n.add_text(
        Locale::En,
        Key::BookingLogFilterCreatedBy,
        "Filter by Creator",
    );
    i18n.add_text(Locale::En, Key::BookingLogFilterClear, "Clear Filters");
    i18n.add_text(Locale::En, Key::BookingLogFilterAll, "All");
    i18n.add_text(Locale::En, Key::BookingLogFilterActiveOnly, "Active Only");
    i18n.add_text(Locale::En, Key::BookingLogFilterDeletedOnly, "Deleted Only");
    i18n.add_text(Locale::En, Key::BookingLogDeletedTag, "Deleted");
    i18n.add_text(Locale::En, Key::BookingNoInfo, "No information available");

    // Weekly overview page
    i18n.add_text(Locale::En, Key::WeeklyOverviewTitle, "Weekly Overview");
    i18n.add_text(Locale::En, Key::PaidVolunteer, "Paid / Volunteer");
    i18n.add_text(Locale::En, Key::Committed, "Voluntary committed");
    i18n.add_text(
        Locale::En,
        Key::PaidCommittedVolunteer,
        "Paid / Voluntary committed / Volunteer",
    );
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
    i18n.add_text(Locale::En, Key::CategoryUnpaidLeave, "Unpaid Leave");
    i18n.add_text(Locale::En, Key::CategoryVolunteerWork, "Volunteer Work");
    i18n.add_text(Locale::En, Key::CategoryCustom, "Custom");

    i18n.add_text(
        Locale::En,
        Key::CapPlannedHoursLabel,
        "Cap planned hours at expected",
    );
    i18n.add_text(
        Locale::En,
        Key::CapPlannedHoursHelp,
        "Hours beyond the expected weekly amount are recorded as volunteer work and do not affect the balance.",
    );

    i18n.add_text(Locale::En, Key::VacationDaysLabel, "Vacation days");
    i18n.add_text(
        Locale::En,
        Key::VacationCarryoverLabel,
        "Open vacation days from last year",
    );

    i18n.add_text(Locale::En, Key::ShowDetails, "More");
    i18n.add_text(Locale::En, Key::HideDetails, "Less");

    i18n.add_text(Locale::En, Key::Hours, "hours");
    i18n.add_text(Locale::En, Key::HoursShort, "h");
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
    i18n.add_text(
        Locale::En,
        Key::EditExtraHoursFormTitle,
        "Edit extra hours entry",
    );
    i18n.add_text(Locale::En, Key::EditExtraHourLabel, "Edit");
    i18n.add_text(
        Locale::En,
        Key::ExtraHoursConflictNotice,
        "Entry was modified elsewhere; the data has been refreshed. Please retry your edit.",
    );
    i18n.add_text(Locale::En, Key::Category, "Category");
    i18n.add_text(Locale::En, Key::AmountOfHours, "Amount of hours");
    i18n.add_text(Locale::En, Key::AmountOfDays, "Amount of days");
    i18n.add_text(Locale::En, Key::Description, "Description");
    i18n.add_text(Locale::En, Key::When, "When");
    i18n.add_text(Locale::En, Key::Submit, "Submit");
    i18n.add_text(Locale::En, Key::Cancel, "Cancel");
    i18n.add_text(Locale::En, Key::ErrorBannerDismiss, "Dismiss error");

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
        Key::WorkdaysHelp,
        "The days the person usually works.",
    );
    i18n.add_text(
        Locale::En,
        Key::ExpectedHoursPerWeekLabel,
        "Expected hours per week",
    );
    i18n.add_text(
        Locale::En,
        Key::ExpectedHoursPerWeekHelp,
        "How many target hours per week.",
    );
    i18n.add_text(Locale::En, Key::ExpectedHours, "Expected Hours");
    i18n.add_text(Locale::En, Key::DaysPerWeekLabel, "Days per week");
    i18n.add_text(
        Locale::En,
        Key::DaysPerWeekHelp,
        "How many days per week the person usually comes in.",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementsPerYearLabel,
        "Vacation days",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementsPerYearHelp,
        "The total annual leave per year as per contract.",
    );
    i18n.add_text(Locale::En, Key::DynamicHourLabel, "Dynamic hours");
    i18n.add_text(
        Locale::En,
        Key::DynamicHourHelp,
        "The target always matches the hours worked \u{2014} ideal when the person is paid by the hour.",
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
    i18n.add_text(Locale::En, Key::SlotEditSaveError, "Error saving slot");
    i18n.add_text(Locale::En, Key::SlotEditModeScopeLabel, "Scope");
    i18n.add_text(
        Locale::En,
        Key::SlotEditModeFromThisWeek,
        "From this week on (default)",
    );
    i18n.add_text(
        Locale::En,
        Key::SlotEditModeThisWeekOnly,
        "This week only",
    );
    i18n.add_text(
        Locale::En,
        Key::SlotEditModeThisWeekOnlyHint,
        "Changes apply exclusively to calendar week {week}/{year}. From the following week, the original slot values are automatically restored.",
    );

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
    i18n.add_text(Locale::En, Key::Create, "Create");
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
    i18n.add_text(Locale::En, Key::CommittedVoluntaryLabel, "Voluntary Commitment (h)");
    i18n.add_text(
        Locale::En,
        Key::CommittedVoluntaryHelp,
        "Committed voluntary hours.",
    );
    i18n.add_text(Locale::En, Key::EmployeesShowAll, "all");
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
    i18n.add_text(Locale::En, Key::TemplateEngine, "Template Engine");
    i18n.add_text(Locale::En, Key::TemplateEngineTera, "Tera");
    i18n.add_text(Locale::En, Key::TemplateEngineMiniJinja, "MiniJinja");
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
    i18n.add_text(
        Locale::En,
        Key::UserInvitationsLoadError,
        "Failed to load invitations. See error banner for details.",
    );
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

    // Delete billing period
    i18n.add_text(Locale::En, Key::DeleteBillingPeriod, "Delete");
    i18n.add_text(Locale::En, Key::ConfirmDeleteBillingPeriod, "Are you sure you want to delete the billing period {period}? This action cannot be undone.");
    i18n.add_text(
        Locale::En,
        Key::DeleteBillingPeriodError,
        "Failed to delete billing period: {error}",
    );

    // My Shifts page
    i18n.add_text(Locale::En, Key::MyShifts, "My Shifts");
    i18n.add_text(
        Locale::En,
        Key::NoShiftsFound,
        "No shifts found for this period.",
    );

    // Day view
    i18n.add_text(Locale::En, Key::ViewModeWeek, "Week");
    i18n.add_text(Locale::En, Key::ViewModeDay, "Day");

    // Weekly overview chart
    i18n.add_text(Locale::En, Key::ChartRequiredHours, "Required Hours");
    i18n.add_text(Locale::En, Key::PreviousYear, "Previous year");
    i18n.add_text(Locale::En, Key::NextYear, "Next year");
    i18n.add_text(Locale::En, Key::WeekShort, "W");

    // Shiftplan assignments
    i18n.add_text(
        Locale::En,
        Key::ShiftplanAssignments,
        "Shiftplan Assignments",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanAssignmentsInfo,
        "No selection means this person is eligible for all shiftplans.",
    );
    i18n.add_text(Locale::En, Key::PermissionLevelAvailable, "Available");
    i18n.add_text(Locale::En, Key::PermissionLevelPlannerOnly, "Planner Only");
    i18n.add_text(
        Locale::En,
        Key::BookingForbidden,
        "This person is not eligible for this shiftplan.",
    );

    // Employees page
    i18n.add_text(Locale::En, Key::SearchPlaceholder, "Search…");
    i18n.add_text(Locale::En, Key::OtherHours, "Other hours");
    i18n.add_text(Locale::En, Key::More, "More");
    i18n.add_text(Locale::En, Key::BackToList, "Back");
    i18n.add_text(Locale::En, Key::HoursUnderTarget, "below target");
    i18n.add_text(Locale::En, Key::HoursOverTarget, "above target");
    i18n.add_text(Locale::En, Key::TargetReached, "Target reached");

    // User management page
    i18n.add_text(Locale::En, Key::ColumnLinkedUser, "Linked user");
    i18n.add_text(
        Locale::En,
        Key::ColumnLinkedSalesPerson,
        "Linked sales person",
    );
    i18n.add_text(Locale::En, Key::ColumnRoles, "Roles");
    i18n.add_text(Locale::En, Key::ColumnType, "Type");
    i18n.add_text(Locale::En, Key::Unlinked, "—");
    i18n.add_text(Locale::En, Key::DeleteUserConfirmTitle, "Delete user");
    i18n.add_text(
        Locale::En,
        Key::DeleteUserConfirmBody,
        "Are you sure you want to delete user {username}? This cannot be undone.",
    );

    // Working-hours mini overview (cards / table layout toggle)
    i18n.add_text(Locale::En, Key::WorkingHoursLayoutCards, "Cards");
    i18n.add_text(Locale::En, Key::WorkingHoursLayoutTable, "Table");
    i18n.add_text(Locale::En, Key::WorkingHoursTableEmployee, "Employee");
    i18n.add_text(Locale::En, Key::WorkingHoursTableActual, "Actual");
    i18n.add_text(Locale::En, Key::WorkingHoursTableTarget, "Target");
    i18n.add_text(Locale::En, Key::WorkingHoursTableUtilization, "Utilization");
    i18n.add_text(Locale::En, Key::WorkingHoursTableTotal, "Total");

    // Absence management (Phase 8)
    i18n.add_text(Locale::En, Key::AbsencePageTitle, "Absences");
    i18n.add_text(
        Locale::En,
        Key::AbsencePageSubtitle,
        "Vacation, sick leave and unpaid leave as date ranges. Hours per day are derived from the active employment contract.",
    );
    i18n.add_text(Locale::En, Key::AbsenceMenuLabel, "Absences");
    i18n.add_text(Locale::En, Key::AbsenceNewBtn, "New absence");
    i18n.add_text(Locale::En, Key::AbsenceModalCreateBtn, "Create");
    i18n.add_text(Locale::En, Key::AbsenceModalSaveBtn, "Save");
    i18n.add_text(Locale::En, Key::AbsenceModalCancelBtn, "Cancel");
    i18n.add_text(Locale::En, Key::AbsenceModalDeleteBtn, "Delete");
    i18n.add_text(Locale::En, Key::AbsenceEmptyFilterHeading, "No results");
    i18n.add_text(
        Locale::En,
        Key::AbsenceEmptyFilterBody,
        "No absences match the current filter. Reset filters or create a new absence.",
    );
    i18n.add_text(Locale::En, Key::AbsenceEmptySelfHeading, "No absences yet");
    i18n.add_text(
        Locale::En,
        Key::AbsenceEmptySelfBody,
        "Create your first absence (vacation, sick leave or unpaid leave).",
    );
    i18n.add_text(Locale::En, Key::AbsenceFieldEmployee, "Employee");
    i18n.add_text(Locale::En, Key::AbsenceFieldCategory, "Category");
    i18n.add_text(Locale::En, Key::AbsenceFieldFrom, "From");
    i18n.add_text(Locale::En, Key::AbsenceFieldTo, "To (inclusive)");
    i18n.add_text(Locale::En, Key::AbsenceFieldDescription, "Description");
    i18n.add_text(
        Locale::En,
        Key::AbsenceFieldDescriptionHint,
        "Optional — e.g. travel destination or note.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceModalCreateSubtitle,
        "Full-day range. Hours are derived from the contract.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceModalEditSubtitle,
        "Changes are saved with optimistic locking.",
    );
    i18n.add_text(Locale::En, Key::AbsencePreviewHeader, "Preview");
    i18n.add_text(
        Locale::En,
        Key::AbsencePreviewFooter,
        "Holidays in the range count as 0 h. Hours per day come from the contract active on that day.",
    );
    i18n.add_text(Locale::En, Key::AbsenceCategoryVacation, "Vacation");
    i18n.add_text(Locale::En, Key::AbsenceCategorySickLeave, "Sick leave");
    i18n.add_text(Locale::En, Key::AbsenceCategoryUnpaidLeave, "Unpaid leave");
    i18n.add_text(Locale::En, Key::AbsenceFilterCategoryLabel, "Category");
    i18n.add_text(Locale::En, Key::AbsenceFilterCategoryAll, "All");
    i18n.add_text(Locale::En, Key::AbsenceFilterPersonLabel, "Person");
    i18n.add_text(Locale::En, Key::AbsenceFilterPersonAll, "All people");
    i18n.add_text(Locale::En, Key::AbsenceGroupEmployees, "Employees");
    i18n.add_text(Locale::En, Key::AbsenceGroupVolunteers, "Volunteers");
    i18n.add_text(Locale::En, Key::AbsenceFilterStatusLabel, "Status");
    i18n.add_text(Locale::En, Key::AbsenceFilterStatusAll, "All");
    i18n.add_text(Locale::En, Key::AbsenceStatusActive, "Active");
    i18n.add_text(Locale::En, Key::AbsenceStatusPlanned, "Planned");
    i18n.add_text(Locale::En, Key::AbsenceStatusFinished, "Finished");
    i18n.add_text(Locale::En, Key::AbsenceColEmployee, "Employee");
    i18n.add_text(Locale::En, Key::AbsenceColRange, "Range");
    i18n.add_text(Locale::En, Key::AbsenceColCategory, "Category");
    i18n.add_text(Locale::En, Key::AbsenceColStatus, "Status");
    i18n.add_text(Locale::En, Key::AbsenceColWarnings, "Warnings");
    i18n.add_text(Locale::En, Key::AbsenceDayUnit, "day");
    i18n.add_text(Locale::En, Key::AbsenceDaysUnit, "days");
    i18n.add_text(Locale::En, Key::AbsenceOneWeek, "1 week");
    i18n.add_text(
        Locale::En,
        Key::VacationEntitlementHero,
        "Vacation entitlement {year}",
    );
    i18n.add_text(Locale::En, Key::VacationDaysRemaining, "days remaining");
    i18n.add_text(
        Locale::En,
        Key::VacationCardSelfTitle,
        "Your vacation balance",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationCardSelfSubtitle,
        "Entitlement from contract + carryover from previous year.",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationCardTeamTitle,
        "Vacation entitlement team · {count} people",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationCardTeamSubtitle,
        "Sum across all paid employees.",
    );
    i18n.add_text(Locale::En, Key::VacationStatContract, "Contract");
    i18n.add_text(
        Locale::En,
        Key::VacationStatCarryover,
        "Carryover '{year-1}'",
    );
    i18n.add_text(Locale::En, Key::VacationStatUsed, "Used");
    i18n.add_text(Locale::En, Key::VacationStatPending, "Pending");
    i18n.add_text(Locale::En, Key::VacationStatRemaining, "Remaining");
    i18n.add_text(Locale::En, Key::VacationOffsetComputedLabel, "calculated");
    i18n.add_text(Locale::En, Key::VacationOffsetLabel, "Offset");
    i18n.add_text(
        Locale::En,
        Key::VacationPerPersonHeader,
        "Per person · sorted by days remaining",
    );
    i18n.add_text(
        Locale::En,
        Key::VacationPerPersonShowAll,
        "All ({count})",
    );
    i18n.add_text(Locale::En, Key::VacationPerPersonShowLess, "Less");
    i18n.add_text(
        Locale::En,
        Key::AbsenceStatSickLeaveDays,
        "Sick days {year}",
    );
    i18n.add_text(Locale::En, Key::AbsenceStatUnpaidDays, "Unpaid {year}");
    i18n.add_text(Locale::En, Key::AbsenceStatActive, "Active absences");
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorRangeInverted,
        "End date is before start date",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorSelfOverlapHeader,
        "Self-overlap",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorSelfOverlapBody,
        "A {category} entry from {from} to {to} overlaps. Please adjust the range or category.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorVersionConflictHeader,
        "Entry changed elsewhere",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorVersionConflictBody,
        "Please reload, then save again. Your input is preserved.",
    );
    i18n.add_text(Locale::En, Key::AbsenceErrorVersionConflictReload, "Reload");
    i18n.add_text(
        Locale::En,
        Key::AbsenceErrorNetwork,
        "Network error. Please try again.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceWarningHeaderSingular,
        "Notice · 1 conflict (non-blocking)",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceWarningHeaderPlural,
        "Notice · {count} conflicts (non-blocking)",
    );
    i18n.add_text(Locale::En, Key::AbsenceWarningAcknowledgeBtn, "Got it");
    i18n.add_text(
        Locale::En,
        Key::AbsenceWarningOverlapsBooking,
        "Existing booking on {date} overlaps with this absence.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceWarningOverlapsManual,
        "Manually marked unavailable day overlaps. After cutover this entry is redundant.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceDeleteConfirmTitle,
        "Delete absence?",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceDeleteConfirmBody,
        "Soft-delete — the entry stays in audit logs but is no longer counted in reports or shown in the shiftplan.",
    );
    i18n.add_text(Locale::En, Key::AbsenceDeleteConfirmBtn, "Delete");
    i18n.add_text(Locale::En, Key::AbsenceDeleteCancelBtn, "Cancel");
    i18n.add_text(Locale::En, Key::EmployeeWorkDetailsDeleteBtn, "Delete contract");
    i18n.add_text(Locale::En, Key::EmployeeWorkDetailsDeleteConfirmTitle, "Delete contract?");
    i18n.add_text(Locale::En, Key::EmployeeWorkDetailsDeleteConfirmBody, "Really delete this contract? The entry stays in audit logs but will no longer be counted in reports.");
    i18n.add_text(Locale::En, Key::EmployeeWorkDetailsDeleteConfirmBtn, "Delete");
    i18n.add_text(Locale::En, Key::AbsenceFilterShowPast, "Show past");
    i18n.add_text(Locale::En, Key::AbsenceFilterCounter, "{n} of {m}");

    // AbsenceConvertModal (Phase 8.6: renamed from CutoverManualConvert*/CutoverEdit*).
    i18n.add_text(Locale::En, Key::AbsenceConvertModalTitle, "Set absence range manually");
    i18n.add_text(
        Locale::En,
        Key::AbsenceConvertModalHelp,
        "You set the date range directly instead of letting the heuristic guess.",
    );
    i18n.add_text(Locale::En, Key::AbsenceConvertAmountLabel, "Amount (h)");
    i18n.add_text(Locale::En, Key::AbsenceConvertStartLabel, "Start date");
    i18n.add_text(Locale::En, Key::AbsenceConvertEndLabel, "End date");
    i18n.add_text(Locale::En, Key::AbsenceConvertBtnSubmit, "Create");
    i18n.add_text(Locale::En, Key::AbsenceConvertBtnCancel, "Discard Changes");
    i18n.add_text(
        Locale::En,
        Key::AbsenceConvertErrStartAfterEnd,
        "Start date must be on or before end date.",
    );

    // Phase 8.3 — Halbtag-Support (Absence).
    i18n.add_text(Locale::En, Key::AbsenceFieldDayFraction, "Day fraction");
    i18n.add_text(Locale::En, Key::AbsenceDayFractionFull, "Full day");
    i18n.add_text(Locale::En, Key::AbsenceDayFractionHalf, "Half day");
    i18n.add_text(
        Locale::En,
        Key::AbsenceFieldDayFractionFullHint,
        "Uses the full contract-hours rate per day.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsenceFieldDayFractionHalfHint,
        "Uses 0.5 vacation days per day in the range.",
    );
    i18n.add_text(
        Locale::En,
        Key::AbsencePreviewFooterHalfDay,
        "Half-day: shown hours are already halved.",
    );

    // Phase 8.5 Plan 06 — Stundenbasierte Marker inline in Absence-Liste.
    i18n.add_text(Locale::En, Key::AbsenceHourlyBadge, "hours-based");
    i18n.add_text(Locale::En, Key::AbsenceEditHoursAction, "Edit hours");
    i18n.add_text(
        Locale::En,
        Key::AbsenceConvertToRangeAction,
        "Convert to range",
    );
    i18n.add_text(Locale::En, Key::AbsenceHourlyAmountLabel, "hrs");
    // Phase 20 Plan 01 — UV-03: ⚠️ indicator tooltip.
    i18n.add_text(
        Locale::En,
        Key::AbsenceHourlyWarnIndicator,
        "Not yet converted to a period — please convert",
    );

    // Phase 8.5 Plan 07 — Soft-migration hint in Working-Hours dialog (D-10/D-11).
    i18n.add_text(
        Locale::En,
        Key::ExtraHoursAbsenceHint,
        "For full vacation or absence periods, please use the absences form.",
    );
    i18n.add_text(Locale::En, Key::ExtraHoursAbsenceHintLink, "Go to absences");

    // Phase 9 — Booking-Flow Reverse-Warnings (FUI-A-05).
    i18n.add_text(
        Locale::En,
        Key::BookingWarningDialogHeaderSingular,
        "Notice · 1 Conflict",
    );
    i18n.add_text(
        Locale::En,
        Key::BookingWarningDialogHeaderPlural,
        "Notice · {count} Conflicts",
    );
    i18n.add_text(Locale::En, Key::BookingWarningDismiss, "Dismiss warning");
    i18n.add_text(
        Locale::En,
        Key::BookingWarningOnAbsenceDay,
        "{person} is absent on {date} as {category}.",
    );
    i18n.add_text(
        Locale::En,
        Key::BookingWarningOnUnavailableDay,
        "{person} is marked as unavailable in week {week}/{year} ({day}).",
    );
    i18n.add_text(
        Locale::En,
        Key::BookingWarningPaidLimitExceeded,
        "Paid employee limit exceeded ({current}/{max}).",
    );

    // Phase 23 — Slot paid-capacity editor (FUI-02).
    i18n.add_text(Locale::En, Key::MaxPaidEmployeesLabel, "Max paid employees");
    i18n.add_text(Locale::En, Key::MaxPaidEmployeesHint, "Empty = no limit");
    i18n.add_text(
        Locale::En,
        Key::MaxPaidEmployeesOverageHint,
        "Currently {current} paid ({limit} allowed)",
    );

    // Quick-260613-jxe — Year navigation on the Absences page.
    i18n.add_text(Locale::En, Key::AbsenceYearNavPrev, "Previous year");
    i18n.add_text(Locale::En, Key::AbsenceYearNavNext, "Next year");

    // Phase 22 — HR-only employee statistics block (STAT-01/STAT-02).
    i18n.add_text(Locale::En, Key::StatisticsHeading, "Statistics");
    i18n.add_text(
        Locale::En,
        Key::AverageWorkedHoursPerWeek,
        "Average worked hours per week",
    );
    i18n.add_text(Locale::En, Key::StatisticsIncludedWeeks, "Included weeks");
    // Phase 47 — Weekday attendance distribution (RPT-02/RPT-03).
    i18n.add_text(Locale::En, Key::WeekdayShortMon, "Mon");
    i18n.add_text(Locale::En, Key::WeekdayShortTue, "Tue");
    i18n.add_text(Locale::En, Key::WeekdayShortWed, "Wed");
    i18n.add_text(Locale::En, Key::WeekdayShortThu, "Thu");
    i18n.add_text(Locale::En, Key::WeekdayShortFri, "Fri");
    i18n.add_text(Locale::En, Key::WeekdayShortSat, "Sat");
    i18n.add_text(Locale::En, Key::WeekdayShortSun, "Sun");
    i18n.add_text(
        Locale::En,
        Key::WeekdayAttendanceTooltip,
        "Attendance-day count per weekday and share relative to counted calendar weeks in the range",
    );
    i18n.add_text(
        Locale::En,
        Key::WeekdayAttendanceEmpty,
        "No counted calendar weeks in range",
    );
    i18n.add_text(Locale::En, Key::WeekdayAttendanceLabel, "Attendance / day");

    // Phase 24 — Paid-limit enforcement config (D-24-06, D-24-03, D-24-05).
    i18n.add_text(
        Locale::En,
        Key::SettingsPaidLimitToggleLabel,
        "Paid employee limit enforcement",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsPaidLimitToggleDescription,
        "When enabled, booking over the paid limit is blocked for non-shift-planners.",
    );
    i18n.add_text(Locale::En, Key::SettingsPaidLimitToggleOn, "Hard (enforced)");
    i18n.add_text(
        Locale::En,
        Key::SettingsPaidLimitToggleOff,
        "Soft (warnings only)",
    );
    i18n.add_text(Locale::En, Key::SettingsSaved, "Saved.");
    i18n.add_text(
        Locale::En,
        Key::SettingsSaveError,
        "Could not save setting.",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanPaidOverageSectionHeader,
        "Paid employee limit exceeded this week",
    );
    i18n.add_text(
        Locale::En,
        Key::ShiftplanPaidOverageRow,
        "{slot}: {current}/{max} paid",
    );
    i18n.add_text(
        Locale::En,
        Key::BookingBlockedPaidLimit,
        "Paid employee limit reached — only shift planners may book beyond the limit.",
    );

    // Phase 25 — Holiday auto-credit activation date (D-25-06, HCFG-02).
    i18n.add_text(
        Locale::En,
        Key::SettingsHolidayAutoCreditLabel,
        "Holiday auto-credit activation date",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsHolidayAutoCreditDescription,
        "Holidays on or after this date are credited automatically. Leave empty to disable.",
    );
    i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditSave, "Save date");
    i18n.add_text(
        Locale::En,
        Key::SettingsHolidayAutoCreditClear,
        "Clear (disable)",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsHolidayAutoCreditUnsetHint,
        "Not set — automation is off.",
    );

    // Phase 26 — NAV-01 bidirectional cross-navigation links (D-26-06).
    i18n.add_text(Locale::En, Key::NavToMyAbsences, "My absences");
    i18n.add_text(Locale::En, Key::NavToEmployeeAbsences, "{name}'s absences");
    i18n.add_text(Locale::En, Key::NavToMyTimeAccount, "My time account");
    i18n.add_text(Locale::En, Key::NavToEmployeeReport, "{name}'s time account");

    // Phase 32 — Impersonation UI (D-32-08).
    i18n.add_text(Locale::En, Key::ImpersonateActAs, "🥸 Act as");
    i18n.add_text(Locale::En, Key::ImpersonateBanner, "You are acting as {user}.");
    i18n.add_text(Locale::En, Key::ImpersonateStop, "Stop impersonation");
    i18n.add_text(
        Locale::En,
        Key::ImpersonateP10Hint,
        "Admin-only functions are disabled while acting as a non-admin; you can stop at any time.",
    );

    // Phase 33 — Special Days Settings Card-3 + Shiftplan dropdown (SPD-04).
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysSectionLabel,
        "Special Days",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysSectionDescription,
        "Manage holidays and short days for the shift plan.",
    );
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysYearLabel, "Year");
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysDateLabel, "Date");
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysTypeLabel, "Type");
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysTypeHoliday, "Holiday");
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysTypeShortDay, "Short Day");
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysTimeLabel,
        "Short day end time",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysAddBtn,
        "Add Special Day",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysEmptyBody,
        "No special days in {year}. Pick a date above to add the first one.",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysDuplicateHint,
        "A special day is already set for this date — creating will replace it.",
    );
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysDeleteBtn, "Delete");
    i18n.add_text(
        Locale::En,
        Key::SettingsSpecialDaysDeleteError,
        "Delete failed.",
    );
    i18n.add_text(Locale::En, Key::SettingsSpecialDaysCalendarWeekAbbr, "W");
    i18n.add_text(Locale::En, Key::ShiftplanDayTypeHoliday, "Holiday");
    i18n.add_text(Locale::En, Key::ShiftplanDayTypeShortDay, "Short Day");
    i18n.add_text(Locale::En, Key::ShiftplanDayTypeNone, "None");
    i18n.add_text(Locale::En, Key::ShiftplanDayShortDayConfirm, "Save time");

    // Week status (KW-Status) — D-39-09
    i18n.add_text(Locale::En, Key::WeekStatusUnset, "None");
    i18n.add_text(Locale::En, Key::WeekStatusInPlanning, "In planning");
    i18n.add_text(Locale::En, Key::WeekStatusPlanned, "Planned");
    i18n.add_text(Locale::En, Key::WeekStatusLocked, "Locked");
    i18n.add_text(
        Locale::En,
        Key::WeekStatusSetError,
        "Failed to save week status.",
    );
    i18n.add_text(
        Locale::En,
        Key::WeekStatusChangeAriaLabel,
        "Change week status",
    );

    // Phase 46 (HYG-04): Shiftplan structure-dropdown labels.
    i18n.add_text(Locale::En, Key::ShiftplanEditStructure, "Edit structure");
    i18n.add_text(Locale::En, Key::ShiftplanNormalMode, "Normal mode");
    i18n.add_text(Locale::En, Key::ShiftplanNewSlot, "New slot");

    // Phase 48 (EXP-02 / EXP-03) — Nextcloud PDF-Export admin-card.
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportTitle,
        "PDF export to Nextcloud",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportHelp,
        "The backend task renders the weekly shift plans as PDF and uploads them to Nextcloud regularly.",
    );
    i18n.add_text(Locale::En, Key::SettingsPdfExportEnabled, "Export enabled");
    i18n.add_text(Locale::En, Key::SettingsPdfExportUrl, "Nextcloud URL");
    i18n.add_text(Locale::En, Key::SettingsPdfExportUser, "WebDAV user");
    i18n.add_text(Locale::En, Key::SettingsPdfExportToken, "App token");
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportTokenPlaceholder,
        "(unchanged, enter a new token here)",
    );
    i18n.add_text(Locale::En, Key::SettingsPdfExportTargetFolder, "Target folder");
    i18n.add_text(Locale::En, Key::SettingsPdfExportWeeksHorizon, "Weeks horizon");
    i18n.add_text(Locale::En, Key::SettingsPdfExportCronSchedule, "Cron schedule");
    i18n.add_text(Locale::En, Key::SettingsPdfExportSave, "Save");
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportSaveSuccess,
        "Configuration saved",
    );
    i18n.add_text(Locale::En, Key::SettingsPdfExportSaveError, "Failed to save");
    i18n.add_text(Locale::En, Key::SettingsPdfExportTriggerNow, "Export now");
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportTriggerNowSuccess,
        "Export started (running in background)",
    );
    i18n.add_text(
        Locale::En,
        Key::SettingsPdfExportTriggerNowError,
        "Failed to trigger export",
    );
    i18n.add_text(Locale::En, Key::SettingsPdfExportLastSuccess, "Last success:");
    i18n.add_text(Locale::En, Key::SettingsPdfExportLastError, "Last error:");
    i18n.add_text(Locale::En, Key::SettingsPdfExportStatusEmpty, "No runs yet");
}
