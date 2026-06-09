pub mod cs;
pub mod de;
pub mod en;
pub mod i18n;

use std::rc::Rc;

pub use i18n::I18n;
use time::macros::format_description;

use crate::{error::ShiftyError, state::week::Week};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,
    De,
    Cs,
}
impl Locale {
    pub fn from_str(locale: &str) -> Self {
        match locale {
            "en" => Locale::En,
            "de" => Locale::De,
            "cs" => Locale::Cs,
            _ => Locale::En,
        }
    }
}

pub trait LocaleDef {
    fn format_date(&self, date: &time::Date) -> Rc<str>;
    fn format_week(&self, week: &Week) -> Result<Rc<str>, ShiftyError>;
}
impl LocaleDef for Locale {
    fn format_date(&self, date: &time::Date) -> Rc<str> {
        let formatter = match self {
            Locale::En => format_description!("[year]-[month]-[day]"),
            Locale::De => format_description!("[day].[month].[year]"),
            Locale::Cs => format_description!("[day]. [month]. [year]"),
        };
        date.format(formatter).unwrap_or(date.to_string()).into()
    }
    fn format_week(&self, week: &Week) -> Result<Rc<str>, ShiftyError> {
        Ok(format!(
            "#{}: {} - {}",
            week.week,
            self.format_date(&week.monday()?),
            self.format_date(&week.sunday()?)
        )
        .into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Home,
    About,

    // Weekdays
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,

    // Top bar
    Shiftplan,
    Employees,
    MyTime,
    YearOverview,
    Logout,
    TopBarYouAreLabel,
    TopBarAdminGroupLabel,

    // Shiftplan
    ShiftplanCalendarWeek,
    ShiftplanTakeLastWeek,
    ShiftplanEditAs,
    ShiftplanYouAre,
    ConflictBookingsHeader,
    PersonalCalendarExport,
    UnsufficientlyBookedCalendarExport,
    WeekMessage,
    ShiftplanFilledOfNeed,
    ShiftplanLastWeek,
    ShiftplanCellAddTitle,
    ShiftplanCellRemoveTitle,
    ShiftplanCreateTitle,
    ShiftplanEditTitle,
    ShiftplanDeleteConfirmTitle,
    ShiftplanDeleteConfirmBody,
    ShiftplanIsPlanningLabel,

    // Booking log
    BookingLogTitle,
    BookingLogShow,
    BookingLogHide,
    BookingLogLoading,
    BookingLogDay,
    BookingLogName,
    BookingLogTime,
    BookingLogCreated,
    BookingLogCreatedBy,
    BookingLogDeleted,
    BookingLogDeletedBy,
    BookingLogFilterName,
    BookingLogFilterDay,
    BookingLogFilterStatus,
    BookingLogFilterCreatedBy,
    BookingLogFilterClear,
    BookingLogFilterAll,
    BookingLogFilterActiveOnly,
    BookingLogFilterDeletedOnly,
    BookingLogDeletedTag,
    BookingNoInfo,

    // Weekly overview page
    WeeklyOverviewTitle,
    PaidVolunteer,
    AvailableRequiredHours,
    MissingHours,
    UnsavedChanges,

    // Employee report
    OverallHeading,
    WorkingHoursPerWeekHeading,
    WorkingHoursPerDayHeading,
    ExtraHoursHeading,
    WorkDetailsHeading,

    Balance,
    Required,
    Overall,
    CarryoverBalance,
    CategoryShiftplan,
    CategoryExtraWork,
    CategoryVacation,
    CategoryVacationHours,
    CategoryVacationDays,
    CategorySickLeave,
    CategoryHolidays,
    CategoryUnavailable,
    CategoryUnpaidLeave,
    CategoryVolunteerWork,
    CategoryCustom,

    CapPlannedHoursLabel,
    CapPlannedHoursHelp,

    VacationDaysLabel,
    VacationCarryoverLabel,

    ShowDetails,
    HideDetails,

    Hours,
    HoursShort,
    Days,

    AddEntry,
    WorkHoursDescription,
    UnavailableDescription,
    ActionsLabel,
    ShowFullYearLabel,
    ShowUntilNowLabel,
    AddWorkDetailsLabel,

    CurrentWeekNote,

    // Add extra hours form
    AddExtraHoursFormTitle,
    EditExtraHoursFormTitle,
    EditExtraHourLabel,
    ExtraHoursConflictNotice,
    Category,
    AmountOfHours,
    AmountOfDays,
    Description,
    When,
    Submit,
    Cancel,

    // Add extra hours choice form
    AddExtraHoursChoiceTitle,
    AddVacationTitle,
    AddHolidaysTitle,
    AddSickLeaveTitle,
    WeekLabel,
    FullWeekLabel,

    // Non-prod warnings
    NonProdWarning,
    NonProdWarningDetails,

    // Not authenticated and home page
    WelcomeTitle,
    PleaseLogin,
    PleaseChoose,

    // Employee work details form
    AddWorkDetailsFormTitle,
    FromLabel,
    ToLabel,
    WorkdaysLabel,
    ExpectedHoursPerWeekLabel,
    ExpectedHours,
    DaysPerWeekLabel,
    VacationEntitlementsPerYearLabel,
    DynamicHourLabel,
    HolidaysInHoursLabel,
    WorkdaysInHoursLabel,

    // Slot edit
    SlotEditTitle,
    SlotNewTitle,
    SlotEditExplanation,
    SlotEditValidUntilExplanation,
    WeekdayLabel,
    MinPersonsLabel,
    SaveLabel,
    CancelLabel,
    SlotEditSaveError,

    // Custom extra hours management
    CustomExtraHoursManagement,
    Name,
    ModifiesBalance,
    Actions,
    AddNew,
    Save,
    Edit,
    Delete,
    Create,
    ConfirmDelete,

    // Billing period management
    BillingPeriods,
    BillingPeriodDetails,
    CreateNewBillingPeriod,
    BillingPeriod,
    StartDate,
    EndDate,
    CreatedAt,
    CreatedBy,
    DeletedAt,
    DeletedBy,
    Active,
    Deleted,
    SalesPersons,
    BasicInformation,
    LoadingBillingPeriods,
    LoadingBillingPeriodDetails,
    CreateBillingPeriod,
    Period,
    NoSalesPersonsInBillingPeriod,
    SalesPersonsIncluded,
    FilterSalesPersonsByName,
    NoSalesPersonsMatchFilter,
    ShowActive,
    ShowInactive,
    ShowPaid,
    ShowUnpaid,
    Values,
    Delta,
    YtdFrom,
    YtdTo,
    FullYear,
    InvalidBillingPeriodId,
    SelectEndDateForNewBillingPeriod,

    // Text templates
    TemplateEngine,
    TemplateEngineTera,
    TemplateEngineMiniJinja,
    TextTemplateManagement,
    TemplateType,
    TemplateText,
    AddNewTemplate,
    EditTemplate,
    CustomReports,
    GenerateReport,
    SelectTemplate,
    GeneratingReport,
    GeneratedReport,
    CreateNewTemplate,
    Saving,
    TemplateName,

    // User management
    UserManagement,
    UserDetails,
    SalesPersonDetails,
    Users,
    UsersCount,
    SalesPersonsCount,
    NoUsersFound,
    AddFirstUserBelow,
    NoSalesPersonsFound,
    CreateFirstSalesPersonBelow,
    DeleteUser,
    AddNewUser,
    CreateUser,
    CreateNewSalesPerson,
    ManageRolesAndPermissions,
    RoleAssignments,
    RolesCount,
    NoRolesAvailable,
    ContactAdministratorForRoles,
    BackToUserManagement,
    ShiftplanColor,
    ColorPreview,
    Settings,
    ThisPersonReceivesPayment,
    ThisPersonIsInactive,
    UserAccount,
    ConnectUserAccount,
    // User invitations
    UserInvitations,
    GenerateInvitation,
    InvitationLink,
    RevokeInvitation,
    RevokeSession,
    InvitationStatus,
    ExpirationHours,
    InvitationCopied,
    Valid,
    Expired,
    Redeemed,
    SessionRevoked,
    NoInvitationsFound,
    GenerateFirstInvitation,
    InvitationsCount,
    GenerateNewInvitation,
    OptionalExpirationHours,
    SaveChanges,
    LoadingSalesPersonDetails,
    SalesPersonSavedSuccessfully,
    EditSalesPersonInformation,
    CreateNewSalesPersonTitle,
    Paid,
    Volunteer,
    Inactive,
    Login,
    LogoutUser,
    ShiftplanReport,
    GenerateShiftplanReport,
    ShiftplanReportGenerated,
    CopyToClipboard,
    CopiedToClipboard,
    CopyFailed,

    // Delete billing period
    DeleteBillingPeriod,
    ConfirmDeleteBillingPeriod,
    DeleteBillingPeriodError,

    // My Shifts page
    MyShifts,
    NoShiftsFound,

    // Day view
    ViewModeWeek,
    ViewModeDay,

    // Weekly overview chart
    ChartRequiredHours,
    PreviousYear,
    NextYear,
    WeekShort,

    // Shiftplan assignments
    ShiftplanAssignments,
    ShiftplanAssignmentsInfo,
    PermissionLevelAvailable,
    PermissionLevelPlannerOnly,
    BookingForbidden,

    // Employees page
    SearchPlaceholder,
    OtherHours,
    More,
    BackToList,
    HoursUnderTarget,
    HoursOverTarget,
    TargetReached,

    // User management page
    ColumnLinkedUser,
    ColumnLinkedSalesPerson,
    ColumnRoles,
    ColumnType,
    Unlinked,
    DeleteUserConfirmTitle,
    DeleteUserConfirmBody,

    // Working-hours mini overview (cards / table layout toggle)
    WorkingHoursLayoutCards,
    WorkingHoursLayoutTable,
    WorkingHoursTableEmployee,
    WorkingHoursTableActual,
    WorkingHoursTableTarget,
    WorkingHoursTableUtilization,
    WorkingHoursTableTotal,

    // Absence management (Phase 8)
    // Page-Level
    AbsencePageTitle,
    AbsencePageSubtitle,
    AbsenceMenuLabel,
    // Primary CTA
    AbsenceNewBtn,
    AbsenceModalCreateBtn,
    AbsenceModalSaveBtn,
    AbsenceModalCancelBtn,
    AbsenceModalDeleteBtn,
    // Empty State
    AbsenceEmptyFilterHeading,
    AbsenceEmptyFilterBody,
    AbsenceEmptySelfHeading,
    AbsenceEmptySelfBody,
    // Form Labels & Hints
    AbsenceFieldEmployee,
    AbsenceFieldCategory,
    AbsenceFieldFrom,
    AbsenceFieldTo,
    AbsenceFieldDescription,
    AbsenceFieldDescriptionHint,
    AbsenceModalCreateSubtitle,
    AbsenceModalEditSubtitle,
    AbsencePreviewHeader,
    AbsencePreviewFooter,
    // Categories
    AbsenceCategoryVacation,
    AbsenceCategorySickLeave,
    AbsenceCategoryUnpaidLeave,
    AbsenceFilterCategoryLabel,
    AbsenceFilterCategoryAll,
    AbsenceFilterPersonLabel,
    AbsenceFilterPersonAll,
    AbsenceFilterStatusLabel,
    AbsenceFilterStatusAll,
    // Status
    AbsenceStatusActive,
    AbsenceStatusPlanned,
    AbsenceStatusFinished,
    // Liste-Spaltenheader
    AbsenceColEmployee,
    AbsenceColRange,
    AbsenceColCategory,
    AbsenceColStatus,
    AbsenceColWarnings,
    AbsenceDayUnit,
    AbsenceDaysUnit,
    // VacationEntitlementCard
    VacationEntitlementHero,
    VacationDaysRemaining,
    VacationCardSelfTitle,
    VacationCardSelfSubtitle,
    VacationCardTeamTitle,
    VacationCardTeamSubtitle,
    VacationStatContract,
    VacationStatCarryover,
    VacationStatUsed,
    VacationStatPending,
    VacationStatRemaining,
    VacationPerPersonHeader,
    VacationPerPersonShowAll,
    VacationPerPersonShowLess,
    // Statistik-Cards
    AbsenceStatSickLeaveDays,
    AbsenceStatUnpaidDays,
    AbsenceStatActive,
    // Errors & Warnings
    AbsenceErrorRangeInverted,
    AbsenceErrorSelfOverlapHeader,
    AbsenceErrorSelfOverlapBody,
    AbsenceErrorVersionConflictHeader,
    AbsenceErrorVersionConflictBody,
    AbsenceErrorVersionConflictReload,
    AbsenceErrorNetwork,
    AbsenceWarningHeaderSingular,
    AbsenceWarningHeaderPlural,
    AbsenceWarningAcknowledgeBtn,
    AbsenceWarningOverlapsBooking,
    AbsenceWarningOverlapsManual,
    // Destructive Confirmation
    AbsenceDeleteConfirmTitle,
    AbsenceDeleteConfirmBody,
    AbsenceDeleteConfirmBtn,
    AbsenceDeleteCancelBtn,
    // Employee Work Details Destructive Confirmation
    EmployeeWorkDetailsDeleteBtn,
    EmployeeWorkDetailsDeleteConfirmTitle,
    EmployeeWorkDetailsDeleteConfirmBody,
    EmployeeWorkDetailsDeleteConfirmBtn,
    // Filter
    AbsenceFilterShowPast,
    AbsenceFilterCounter,

    // Phase 8.3 — Halbtag-Support (Absence).
    AbsenceFieldDayFraction,
    AbsenceDayFractionFull,
    AbsenceDayFractionHalf,
    AbsenceFieldDayFractionFullHint,
    AbsenceFieldDayFractionHalfHint,
    AbsencePreviewFooterHalfDay,

    // Cutover migration (Phase 8.1 — see 08.1-UI-SPEC.md § Copywriting Contract).
    CutoverMenuLabel,           // TopBar Verwaltung-Submenu entry
    CutoverPageTitle,
    CutoverPageSubtitle,
    CutoverStage1Label,         // "Profile" / "Übersicht"
    CutoverStage2Label,         // "Dry-Run" / "Vorschau"
    CutoverStage3Label,         // "Commit" / "Durchführen"
    CutoverBtnContinue,         // "Continue" / "Weiter"
    CutoverBtnBack,             // "Back" / "Zurück"
    CutoverStatTotalRows,
    CutoverStatPersons,
    CutoverStatQuarantine,
    CutoverStatCarryoverDiff,
    CutoverBtnBulkConvert,      // "Convert all in group"
    CutoverRowBtnConvert,
    CutoverRowBtnEdit,
    CutoverRowBtnDelete,
    CutoverRowBtnSkip,
    CutoverDriftEmptyHeading,
    CutoverDriftEmptyBody,
    CutoverCommitSummaryHeading,
    CutoverCommitTypeLabel,
    CutoverCommitBtn,
    CutoverSuccessHeading,
    CutoverSuccessBody,
    CutoverAlreadyDoneHeading,
    CutoverAlreadyDoneBody,
    CutoverEditModalTitle,
    CutoverEditAmountLabel,
    CutoverEditDateLabel,
    CutoverEditBtnSave,
    CutoverEditBtnCancel,
    CutoverErrorApiFailure,
    CutoverCommitDisabledTooltip,
    CutoverPrivilegeStage3,

    // Cutover Manual Range (Phase 8.2 — D-29) — operator-driven date-range
    // override for the convert-quarantine-entry endpoint. See
    // .planning/phases/08.2-manual-range-convert-quarantine/08.2-RESEARCH.md § 6.
    CutoverManualConvertModalTitle,
    CutoverManualConvertHelp,
    CutoverManualConvertStartLabel,
    CutoverManualConvertEndLabel,
    CutoverManualConvertBtnSubmit,
    CutoverManualConvertErrStartAfterEnd,
    CutoverManualConvertErrYearMismatch,
    CutoverManualConvertErrOverlap,

    // Phase 8.3 — Halbtag-Support (Cutover).
    CutoverDriftColDayFraction,
    CutoverDriftDayFractionAria,
    CutoverBulkDayFractionLabel,
    CutoverDriftHalfDaySuggestion,
    CutoverManualConvertDayFractionLabel,
    CutoverDayFractionFull,
    CutoverDayFractionHalf,

    // Phase 8.5 Plan 06 — Stundenbasierte Marker inline in Absence-Liste.
    /// Badge-Text auf der HourlyMarkerRow ("stundenbasiert" / "hours-based" / …).
    AbsenceHourlyBadge,
    /// Button-Label „Stunden bearbeiten" (self+hr, D-08).
    AbsenceEditHoursAction,
    /// Button-Label „In Zeitraum umwandeln" (HR-only, D-09).
    AbsenceConvertToRangeAction,
    /// Suffix für die Stundenanzahl in der Marker-Row (De: „Std.", En: „hrs").
    AbsenceHourlyAmountLabel,
}

pub fn generate(locale: Locale) -> I18n<Key, Locale> {
    let mut i18n = I18n::new(locale, Locale::En);

    match locale {
        Locale::En => en::add_i18n_en(&mut i18n),
        Locale::De => de::add_i18n_de(&mut i18n),
        Locale::Cs => cs::add_i18n_cs(&mut i18n),
    }

    i18n
}

pub type I18nType = I18n<Key, Locale>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i18n_employees_keys_present_in_all_locales() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::SearchPlaceholder,
                Key::OtherHours,
                Key::More,
                Key::BackToList,
                Key::HoursUnderTarget,
                Key::HoursOverTarget,
                Key::TargetReached,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_employees_keys_match_german_reference() {
        let de = generate(Locale::De);
        assert_eq!(de.t(Key::OtherHours).as_ref(), "Sonstige Stunden");
        assert_eq!(de.t(Key::More).as_ref(), "Mehr");
        assert_eq!(de.t(Key::BackToList).as_ref(), "Zurück");
    }

    #[test]
    fn i18n_user_management_keys_present_in_all_locales() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::ColumnLinkedUser,
                Key::ColumnLinkedSalesPerson,
                Key::ColumnRoles,
                Key::ColumnType,
                Key::Unlinked,
                Key::DeleteUserConfirmTitle,
                Key::DeleteUserConfirmBody,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_redesign_keys_present_in_all_locales() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::ShiftplanFilledOfNeed,
                Key::ShiftplanLastWeek,
                Key::ShiftplanCellAddTitle,
                Key::ShiftplanCellRemoveTitle,
                Key::ShiftplanCreateTitle,
                Key::ShiftplanEditTitle,
                Key::ShiftplanDeleteConfirmTitle,
                Key::ShiftplanDeleteConfirmBody,
                Key::ShiftplanIsPlanningLabel,
                Key::Create,
                Key::BookingLogDeletedTag,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn shiftplan_filled_of_need_substitutes_placeholders() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            let result = i18n.t_m(
                Key::ShiftplanFilledOfNeed,
                [("filled", "2"), ("need", "3")].into(),
            );
            assert!(
                result.contains('2'),
                "missing filled `2` in {:?}: got `{}`",
                locale,
                result
            );
            assert!(
                result.contains('3'),
                "missing need `3` in {:?}: got `{}`",
                locale,
                result
            );
        }
    }

    #[test]
    fn i18n_absence_keys_present_in_all_locales() {
        // Locks the contract: every absence-domain key has a translation in
        // every locale and never falls back to "??". This is the primary
        // safety net against the historical Locale::En-instead-of-Locale::De
        // bug (Pitfall 2 / 08-RESEARCH.md).
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::AbsencePageTitle,
                Key::AbsencePageSubtitle,
                Key::AbsenceMenuLabel,
                Key::AbsenceNewBtn,
                Key::AbsenceCategoryVacation,
                Key::AbsenceCategorySickLeave,
                Key::AbsenceCategoryUnpaidLeave,
                Key::AbsenceStatusActive,
                Key::AbsenceStatusPlanned,
                Key::AbsenceStatusFinished,
                Key::AbsenceErrorRangeInverted,
                Key::AbsenceErrorSelfOverlapHeader,
                Key::AbsenceErrorVersionConflictHeader,
                Key::VacationCardSelfTitle,
                Key::VacationStatContract,
                Key::VacationStatCarryover,
                Key::VacationStatUsed,
                Key::VacationStatPending,
                Key::VacationStatRemaining,
                Key::AbsenceDeleteConfirmTitle,
                Key::AbsenceWarningAcknowledgeBtn,
                Key::AbsenceFilterShowPast,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_absence_keys_match_german_reference() {
        // Pitfall-2 guard: ensures the de.rs block uses Locale::De (and not
        // accidentally Locale::En, which would still type-check but route
        // English copy through `generate(Locale::De)`).
        let i18n = generate(Locale::De);
        assert_eq!(i18n.t(Key::AbsencePageTitle).as_ref(), "Abwesenheiten");
        assert_eq!(i18n.t(Key::AbsenceCategoryVacation).as_ref(), "Urlaub");
        assert_eq!(i18n.t(Key::AbsenceCategorySickLeave).as_ref(), "Krankheit");
        assert_eq!(
            i18n.t(Key::AbsenceCategoryUnpaidLeave).as_ref(),
            "Unbezahlt"
        );
        assert_eq!(i18n.t(Key::AbsenceStatusActive).as_ref(), "Aktiv");
    }

    #[test]
    fn i18n_absence_keys_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(i18n.t(Key::AbsencePageTitle).as_ref(), "Absences");
        assert_eq!(i18n.t(Key::AbsenceCategoryVacation).as_ref(), "Vacation");
        assert_eq!(i18n.t(Key::AbsenceCategorySickLeave).as_ref(), "Sick leave");
        assert_eq!(i18n.t(Key::AbsenceStatusActive).as_ref(), "Active");
    }

    #[test]
    fn i18n_absence_keys_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(i18n.t(Key::AbsencePageTitle).as_ref(), "Nepřítomnosti");
        assert_eq!(i18n.t(Key::AbsenceCategoryVacation).as_ref(), "Dovolená");
        assert_eq!(i18n.t(Key::AbsenceCategorySickLeave).as_ref(), "Nemoc");
        assert_eq!(i18n.t(Key::AbsenceStatusActive).as_ref(), "Aktivní");
    }

    #[test]
    fn shiftplan_delete_confirm_body_interpolates_name() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            let result = i18n.t_m(
                Key::ShiftplanDeleteConfirmBody,
                [("name", "Hauptplan")].into(),
            );
            assert!(
                result.contains("Hauptplan"),
                "missing interpolated name in {:?}: got `{}`",
                locale,
                result
            );
        }
    }

    #[test]
    fn i18n_cutover_keys_present_in_all_locales() {
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::CutoverMenuLabel,
                Key::CutoverPageTitle,
                Key::CutoverPageSubtitle,
                Key::CutoverStage1Label,
                Key::CutoverStage2Label,
                Key::CutoverStage3Label,
                Key::CutoverBtnContinue,
                Key::CutoverBtnBack,
                Key::CutoverStatTotalRows,
                Key::CutoverStatPersons,
                Key::CutoverStatQuarantine,
                Key::CutoverStatCarryoverDiff,
                Key::CutoverBtnBulkConvert,
                Key::CutoverRowBtnConvert,
                Key::CutoverRowBtnEdit,
                Key::CutoverRowBtnDelete,
                Key::CutoverRowBtnSkip,
                Key::CutoverDriftEmptyHeading,
                Key::CutoverDriftEmptyBody,
                Key::CutoverCommitSummaryHeading,
                Key::CutoverCommitTypeLabel,
                Key::CutoverCommitBtn,
                Key::CutoverSuccessHeading,
                Key::CutoverSuccessBody,
                Key::CutoverAlreadyDoneHeading,
                Key::CutoverAlreadyDoneBody,
                Key::CutoverEditModalTitle,
                Key::CutoverEditAmountLabel,
                Key::CutoverEditDateLabel,
                Key::CutoverEditBtnSave,
                Key::CutoverEditBtnCancel,
                Key::CutoverErrorApiFailure,
                Key::CutoverCommitDisabledTooltip,
                Key::CutoverPrivilegeStage3,
                // Phase 8.2 manual range
                Key::CutoverManualConvertModalTitle,
                Key::CutoverManualConvertHelp,
                Key::CutoverManualConvertStartLabel,
                Key::CutoverManualConvertEndLabel,
                Key::CutoverManualConvertBtnSubmit,
                Key::CutoverManualConvertErrStartAfterEnd,
                Key::CutoverManualConvertErrYearMismatch,
                Key::CutoverManualConvertErrOverlap,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_cutover_keys_match_german_reference() {
        // Pitfall-2 guard — defends against historical Locale::En-statt-Locale::De bug.
        let i18n = generate(Locale::De);
        assert_eq!(i18n.t(Key::CutoverPageTitle).as_ref(), "Datenmigration");
        assert_eq!(i18n.t(Key::CutoverStage1Label).as_ref(), "Übersicht");
        assert_eq!(i18n.t(Key::CutoverStage2Label).as_ref(), "Vorschau");
        assert_eq!(i18n.t(Key::CutoverStage3Label).as_ref(), "Durchführen");
        assert_eq!(i18n.t(Key::CutoverBtnContinue).as_ref(), "Weiter");
        assert_eq!(i18n.t(Key::CutoverBtnBack).as_ref(), "Zurück");
        assert_eq!(i18n.t(Key::CutoverCommitBtn).as_ref(), "Cutover durchführen");
        // Phase 8.2 (D-29) manual-range keys — sample 3 strings.
        assert_eq!(
            i18n.t(Key::CutoverManualConvertModalTitle).as_ref(),
            "Urlaub manuell anlegen"
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertStartLabel).as_ref(),
            "Datum von"
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertBtnSubmit).as_ref(),
            "Anlegen"
        );
    }

    #[test]
    fn i18n_cutover_keys_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(i18n.t(Key::CutoverPageTitle).as_ref(), "Data Migration");
        assert_eq!(i18n.t(Key::CutoverStage1Label).as_ref(), "Profile");
        assert_eq!(i18n.t(Key::CutoverStage2Label).as_ref(), "Dry-Run");
        assert_eq!(i18n.t(Key::CutoverStage3Label).as_ref(), "Commit");
        assert_eq!(i18n.t(Key::CutoverBtnContinue).as_ref(), "Continue");
        assert_eq!(i18n.t(Key::CutoverCommitBtn).as_ref(), "Commit Cutover");
        // Phase 8.2 (D-29) manual-range keys — sample 2 strings.
        assert_eq!(
            i18n.t(Key::CutoverManualConvertModalTitle).as_ref(),
            "Set absence range manually"
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertStartLabel).as_ref(),
            "Start date"
        );
    }

    #[test]
    fn i18n_cutover_keys_match_czech_reference() {
        // Czech reference strings — cf. cs.rs implementation (Task 4).
        let i18n = generate(Locale::Cs);
        // 5 sample keys; full set of 34 covered by the presence test.
        assert_eq!(i18n.t(Key::CutoverPageTitle).as_ref(), "Migrace dat");
        assert_eq!(i18n.t(Key::CutoverStage1Label).as_ref(), "Přehled");
        assert_eq!(i18n.t(Key::CutoverBtnContinue).as_ref(), "Pokračovat");
        assert_eq!(i18n.t(Key::CutoverBtnBack).as_ref(), "Zpět");
        assert_eq!(i18n.t(Key::CutoverCommitBtn).as_ref(), "Provést cutover");
        // Phase 8.2 (D-29) manual-range keys — sample 2 strings.
        assert_eq!(
            i18n.t(Key::CutoverManualConvertModalTitle).as_ref(),
            "Ručně nastavit dovolenou"
        );
        assert_eq!(i18n.t(Key::CutoverManualConvertEndLabel).as_ref(), "Do");
    }

    // ===== Phase 8.3 — Halbtag-Support i18n Tests =====

    #[test]
    fn i18n_absence_day_fraction_keys_present_in_all_locales() {
        // FUI-A-09 — alle 3 Locales (De / En / Cs) müssen alle 6 Absence-
        // Halbtag-Keys decken (non-empty + ≠ "??").
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::AbsenceFieldDayFraction,
                Key::AbsenceDayFractionFull,
                Key::AbsenceDayFractionHalf,
                Key::AbsenceFieldDayFractionFullHint,
                Key::AbsenceFieldDayFractionHalfHint,
                Key::AbsencePreviewFooterHalfDay,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_cutover_day_fraction_keys_present_in_all_locales() {
        // FUI-A-09 — alle 3 Locales × alle 7 Cutover-Halbtag-Keys.
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::CutoverDriftColDayFraction,
                Key::CutoverDriftDayFractionAria,
                Key::CutoverBulkDayFractionLabel,
                Key::CutoverDriftHalfDaySuggestion,
                Key::CutoverManualConvertDayFractionLabel,
                Key::CutoverDayFractionFull,
                Key::CutoverDayFractionHalf,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_absence_day_fraction_match_german_reference() {
        // Pitfall-2 guard: ensures the de.rs block uses Locale::De
        // (not accidentally Locale::En, which would still type-check but
        // route English copy through generate(Locale::De)). Pattern aus
        // Plan 08-04 D-26.
        let i18n = generate(Locale::De);
        assert_eq!(i18n.t(Key::AbsenceFieldDayFraction).as_ref(), "Tageshälfte");
        assert_eq!(i18n.t(Key::AbsenceDayFractionFull).as_ref(), "Ganztag");
        assert_eq!(i18n.t(Key::AbsenceDayFractionHalf).as_ref(), "Halber Tag");
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionFullHint).as_ref(),
            "Verbraucht den vollen Vertrags-Stundensatz pro Tag."
        );
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionHalfHint).as_ref(),
            "Verbraucht 0,5 Urlaubstage pro Tag im Bereich."
        );
        assert_eq!(
            i18n.t(Key::AbsencePreviewFooterHalfDay).as_ref(),
            "Bei Halbtag: angezeigte Stunden sind bereits halbiert."
        );
    }

    #[test]
    fn i18n_absence_day_fraction_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(i18n.t(Key::AbsenceFieldDayFraction).as_ref(), "Day fraction");
        assert_eq!(i18n.t(Key::AbsenceDayFractionFull).as_ref(), "Full day");
        assert_eq!(i18n.t(Key::AbsenceDayFractionHalf).as_ref(), "Half day");
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionFullHint).as_ref(),
            "Uses the full contract-hours rate per day."
        );
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionHalfHint).as_ref(),
            "Uses 0.5 vacation days per day in the range."
        );
        assert_eq!(
            i18n.t(Key::AbsencePreviewFooterHalfDay).as_ref(),
            "Half-day: shown hours are already halved."
        );
    }

    #[test]
    fn i18n_absence_day_fraction_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(i18n.t(Key::AbsenceFieldDayFraction).as_ref(), "Část dne");
        assert_eq!(i18n.t(Key::AbsenceDayFractionFull).as_ref(), "Celý den");
        assert_eq!(i18n.t(Key::AbsenceDayFractionHalf).as_ref(), "Půl dne");
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionFullHint).as_ref(),
            "Spotřebovává plnou smluvní sazbu za každý den."
        );
        assert_eq!(
            i18n.t(Key::AbsenceFieldDayFractionHalfHint).as_ref(),
            "Spotřebovává 0,5 dne dovolené za každý den v období."
        );
        assert_eq!(
            i18n.t(Key::AbsencePreviewFooterHalfDay).as_ref(),
            "Půldenní: zobrazené hodiny jsou již vydělené dvěma."
        );
    }

    #[test]
    fn i18n_cutover_day_fraction_match_german_reference() {
        let i18n = generate(Locale::De);
        assert_eq!(
            i18n.t(Key::CutoverDriftColDayFraction).as_ref(),
            "Tageshälfte"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftDayFractionAria).as_ref(),
            "Tageshälfte für diesen Eintrag wählen"
        );
        assert_eq!(
            i18n.t(Key::CutoverBulkDayFractionLabel).as_ref(),
            "Halb/Ganz für ganze Gruppe"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftHalfDaySuggestion).as_ref(),
            "Stunden deuten auf Halbtag hin ({amount:.2}h ≈ ½ × Vertragstag)."
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertDayFractionLabel).as_ref(),
            "Tageshälfte"
        );
        assert_eq!(i18n.t(Key::CutoverDayFractionFull).as_ref(), "Ganztag");
        assert_eq!(i18n.t(Key::CutoverDayFractionHalf).as_ref(), "Halber Tag");
    }

    #[test]
    fn i18n_cutover_day_fraction_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(
            i18n.t(Key::CutoverDriftColDayFraction).as_ref(),
            "Day fraction"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftDayFractionAria).as_ref(),
            "Pick day fraction for this entry"
        );
        assert_eq!(
            i18n.t(Key::CutoverBulkDayFractionLabel).as_ref(),
            "Half/Full for whole group"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftHalfDaySuggestion).as_ref(),
            "Hours suggest half-day ({amount:.2}h ≈ ½ × contract day)."
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertDayFractionLabel).as_ref(),
            "Day fraction"
        );
        assert_eq!(i18n.t(Key::CutoverDayFractionFull).as_ref(), "Full day");
        assert_eq!(i18n.t(Key::CutoverDayFractionHalf).as_ref(), "Half day");
    }

    #[test]
    fn i18n_cutover_day_fraction_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(
            i18n.t(Key::CutoverDriftColDayFraction).as_ref(),
            "Část dne"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftDayFractionAria).as_ref(),
            "Zvol část dne pro tuto položku"
        );
        assert_eq!(
            i18n.t(Key::CutoverBulkDayFractionLabel).as_ref(),
            "Půl/Celý pro celou skupinu"
        );
        assert_eq!(
            i18n.t(Key::CutoverDriftHalfDaySuggestion).as_ref(),
            "Hodiny naznačují půldenní ({amount:.2}h ≈ ½ × smluvní den)."
        );
        assert_eq!(
            i18n.t(Key::CutoverManualConvertDayFractionLabel).as_ref(),
            "Část dne"
        );
        assert_eq!(i18n.t(Key::CutoverDayFractionFull).as_ref(), "Celý den");
        assert_eq!(i18n.t(Key::CutoverDayFractionHalf).as_ref(), "Půl dne");
    }

    // ===== Phase 8.5 Plan 06 — Stundenbasierte Marker i18n Tests =====

    #[test]
    fn i18n_absence_hourly_marker_keys_present_in_all_locales() {
        // Locks the contract: every hourly-marker key has a translation in all
        // three locales and never falls back to "??".
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::AbsenceHourlyBadge,
                Key::AbsenceEditHoursAction,
                Key::AbsenceConvertToRangeAction,
                Key::AbsenceHourlyAmountLabel,
            ] {
                let value = i18n.t(key);
                assert!(
                    !value.is_empty() && value.as_ref() != "??",
                    "missing translation for {:?} in {:?}: got `{}`",
                    key,
                    locale,
                    value
                );
            }
        }
    }

    #[test]
    fn i18n_absence_hourly_marker_match_german_reference() {
        // Pitfall-2 guard: ensures de.rs uses Locale::De (not Locale::En).
        let i18n = generate(Locale::De);
        assert_eq!(i18n.t(Key::AbsenceHourlyBadge).as_ref(), "stundenbasiert");
        assert_eq!(
            i18n.t(Key::AbsenceEditHoursAction).as_ref(),
            "Stunden bearbeiten"
        );
        assert_eq!(
            i18n.t(Key::AbsenceConvertToRangeAction).as_ref(),
            "In Zeitraum umwandeln"
        );
        assert_eq!(i18n.t(Key::AbsenceHourlyAmountLabel).as_ref(), "Std.");
    }

    #[test]
    fn i18n_absence_hourly_marker_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(i18n.t(Key::AbsenceHourlyBadge).as_ref(), "hours-based");
        assert_eq!(
            i18n.t(Key::AbsenceEditHoursAction).as_ref(),
            "Edit hours"
        );
        assert_eq!(
            i18n.t(Key::AbsenceConvertToRangeAction).as_ref(),
            "Convert to range"
        );
        assert_eq!(i18n.t(Key::AbsenceHourlyAmountLabel).as_ref(), "hrs");
    }

    #[test]
    fn i18n_absence_hourly_marker_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(
            i18n.t(Key::AbsenceHourlyBadge).as_ref(),
            "hodinové záznamy"
        );
        assert_eq!(
            i18n.t(Key::AbsenceEditHoursAction).as_ref(),
            "Upravit hodiny"
        );
        assert_eq!(
            i18n.t(Key::AbsenceConvertToRangeAction).as_ref(),
            "Převést na rozsah"
        );
        assert_eq!(i18n.t(Key::AbsenceHourlyAmountLabel).as_ref(), "hod.");
    }
}
