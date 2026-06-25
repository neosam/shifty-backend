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
    PaidCommittedVolunteer,
    Committed,
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
    /// Schließen-Button (aria-label) im globalen Fehler-Banner (`ErrorView`).
    ErrorBannerDismiss,

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
    CommittedVoluntaryLabel,
    EmployeesShowAll,
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
    AbsenceOneWeek,
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

    // AbsenceConvertModal i18n keys (Phase 8.6: renamed from CutoverManualConvert* /
    // CutoverEdit* namespace; used by `component/absence_convert_modal.rs`).
    AbsenceConvertModalTitle,
    AbsenceConvertModalHelp,
    AbsenceConvertAmountLabel,
    AbsenceConvertStartLabel,
    AbsenceConvertEndLabel,
    AbsenceConvertBtnSubmit,
    AbsenceConvertBtnCancel,
    AbsenceConvertErrStartAfterEnd,

    // Phase 8.5 Plan 06 — Stundenbasierte Marker inline in Absence-Liste.
    /// Badge-Text auf der HourlyMarkerRow ("stundenbasiert" / "hours-based" / …).
    AbsenceHourlyBadge,
    /// Button-Label „Stunden bearbeiten" (self+hr, D-08).
    AbsenceEditHoursAction,
    /// Button-Label „In Zeitraum umwandeln" (HR-only, D-09).
    AbsenceConvertToRangeAction,
    /// Suffix für die Stundenanzahl in der Marker-Row (De: „Std.", En: „hrs").
    AbsenceHourlyAmountLabel,
    /// Tooltip/aria-label für den ⚠️-Indikator in Spalte 1 der HourlyMarkerRow
    /// (Zweck: „Noch nicht in einen Zeitraum umgewandelt — bitte konvertieren").
    /// Phase 20 Plan 01 — UV-03.
    AbsenceHourlyWarnIndicator,

    // Phase 8.5 Plan 07 — Soft-Migration-Hinweis im Working-Hours-Dialog (D-10/D-11).
    /// Empfehlungs-Satz unter dem Kategorie-Select wenn Vacation/SickLeave/UnpaidLeave gewählt.
    ExtraHoursAbsenceHint,
    /// Link-Text im Hinweis (De: „Zu Abwesenheiten", En: „Go to absences").
    ExtraHoursAbsenceHintLink,

    // Quick-260613-jxe — Year navigation on the Absences page.
    /// Aria-label / title for the "previous year" button (◀).
    AbsenceYearNavPrev,
    /// Aria-label / title for the "next year" button (▶).
    AbsenceYearNavNext,

    // Phase 9 — Booking-Flow Reverse-Warnings (FUI-A-05).
    /// Banner-Titel wenn genau 1 Buchungs-Konflikt vorliegt.
    BookingWarningDialogHeaderSingular,
    /// Banner-Titel wenn N > 1 Buchungs-Konflikte vorliegen (Platzhalter `{count}`).
    BookingWarningDialogHeaderPlural,
    /// Schließen-Button im Warnungs-Banner (aria-label).
    BookingWarningDismiss,
    /// Pro-Item-Text wenn Mitarbeiter am gebuchten Tag abwesend ist (Platzhalter `{person}`, `{date}`, `{category}`).
    BookingWarningOnAbsenceDay,
    /// Pro-Item-Text wenn Mitarbeiter in der gebuchten KW als nicht verfügbar markiert ist (Platzhalter `{person}`, `{week}`, `{year}`, `{day}`).
    BookingWarningOnUnavailableDay,
    /// Pro-Item-Text wenn das Bezahlt-Limit überschritten ist (Platzhalter `{current}`, `{max}`).
    BookingWarningPaidLimitExceeded,

    // Phase 22 — HR-only employee statistics block (STAT-01/STAT-02).
    /// Heading of the HR-only statistics block on the employee detail page.
    StatisticsHeading,
    /// Label for the average worked hours per week row.
    AverageWorkedHoursPerWeek,
    /// Label for the included weeks row.
    StatisticsIncludedWeeks,
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

    /// Regression: the self-overlap banner body carries {category}/{from}/{to}
    /// placeholders. They MUST be substituted via `t_m` — rendering the raw
    /// template (the original bug) left the literal `{category}-Eintrag ...`
    /// visible to the user.
    #[test]
    fn i18n_self_overlap_body_substitutes_placeholders_in_all_locales() {
        use std::collections::HashMap;

        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);

            // Sanity: the stored template still contains placeholders, otherwise
            // this test would pass vacuously.
            let raw = i18n.t(Key::AbsenceErrorSelfOverlapBody);
            assert!(
                raw.contains("{category}")
                    && raw.contains("{from}")
                    && raw.contains("{to}"),
                "template for {:?} lost its placeholders: `{}`",
                locale,
                raw
            );

            let mut values: HashMap<&str, &str> = HashMap::new();
            values.insert("category", "Urlaub");
            values.insert("from", "13.06.2026");
            values.insert("to", "20.06.2026");
            let body = i18n.t_m(Key::AbsenceErrorSelfOverlapBody, values);

            assert!(
                !body.contains('{') && !body.contains('}'),
                "unsubstituted placeholder remained for {:?}: `{}`",
                locale,
                body
            );
            assert!(
                body.contains("Urlaub")
                    && body.contains("13.06.2026")
                    && body.contains("20.06.2026"),
                "substituted values missing for {:?}: `{}`",
                locale,
                body
            );
        }
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
    fn i18n_committed_keys_match_german_reference() {
        // Phase-16 D-02/CVC-08 guard: pins the De copy for the committed token +
        // three-band header, and protects against the Locale::De/Locale::En swap bug.
        let i18n = generate(Locale::De);
        assert_eq!(i18n.t(Key::Committed).as_ref(), "Freiwillig zugesagt");
        assert_eq!(
            i18n.t(Key::PaidCommittedVolunteer).as_ref(),
            "Bezahlt / Freiwillig zugesagt / Freiwillig"
        );
    }

    #[test]
    fn i18n_committed_keys_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(i18n.t(Key::Committed).as_ref(), "Voluntary committed");
        assert_eq!(
            i18n.t(Key::PaidCommittedVolunteer).as_ref(),
            "Paid / Voluntary committed / Volunteer"
        );
    }

    #[test]
    fn i18n_committed_keys_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(i18n.t(Key::Committed).as_ref(), "Dobrovolně přislíbeno");
        assert_eq!(
            i18n.t(Key::PaidCommittedVolunteer).as_ref(),
            "Placené / Dobrovolně přislíbeno / Dobrovolné"
        );
    }

    #[test]
    fn i18n_czech_closes_volunteer_and_paid_volunteer_gaps() {
        // Pre-existing cs.rs gaps (verified absent at HEAD) closed by Phase 16 Plan 03.
        let i18n = generate(Locale::Cs);
        assert_eq!(i18n.t(Key::Volunteer).as_ref(), "Dobrovolné");
        assert_eq!(i18n.t(Key::PaidVolunteer).as_ref(), "Placené / Dobrovolné");
    }

    // ===== Phase 17 Plan 03 — CommittedVoluntaryLabel + EmployeesShowAll Per-Locale Tests =====

    #[test]
    fn i18n_phase17_keys_match_german_reference() {
        // Pitfall-6 guard: ensures de.rs uses Locale::De (not accidentally Locale::En).
        let i18n = generate(Locale::De);
        assert_eq!(
            i18n.t(Key::CommittedVoluntaryLabel).as_ref(),
            "Freiwillige Zusage (h)"
        );
        assert_eq!(i18n.t(Key::EmployeesShowAll).as_ref(), "alle");
    }

    #[test]
    fn i18n_phase17_keys_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(
            i18n.t(Key::CommittedVoluntaryLabel).as_ref(),
            "Voluntary Commitment (h)"
        );
        assert_eq!(i18n.t(Key::EmployeesShowAll).as_ref(), "all");
    }

    #[test]
    fn i18n_phase17_keys_match_czech_reference() {
        // Pitfall-6 guard: ensures cs.rs uses Locale::Cs (not accidentally Locale::En).
        let i18n = generate(Locale::Cs);
        assert_eq!(
            i18n.t(Key::CommittedVoluntaryLabel).as_ref(),
            "Dobrovolný závazek (h)"
        );
        assert_eq!(i18n.t(Key::EmployeesShowAll).as_ref(), "vše");
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
                Key::AbsenceHourlyWarnIndicator,
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

    // ===== Phase 8.5 Plan 07 — Soft-Migration-Hinweis i18n Tests =====

    #[test]
    fn i18n_extra_hours_absence_hint_keys_present_in_all_locales() {
        // Locks the contract: both hint keys have a translation in all three
        // locales and never fall back to "??". Primary guard against Pitfall 2.
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [Key::ExtraHoursAbsenceHint, Key::ExtraHoursAbsenceHintLink] {
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
    fn i18n_extra_hours_absence_hint_match_german_reference() {
        // Pitfall-2 guard: ensures de.rs uses Locale::De (not accidentally Locale::En).
        let i18n = generate(Locale::De);
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHint).as_ref(),
            "Für ganze Urlaubs-/Abwesenheits-Zeiträume nutze bitte die Abwesenheits-Maske."
        );
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHintLink).as_ref(),
            "Zu Abwesenheiten"
        );
    }

    #[test]
    fn i18n_extra_hours_absence_hint_match_english_reference() {
        let i18n = generate(Locale::En);
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHint).as_ref(),
            "For full vacation or absence periods, please use the absences form."
        );
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHintLink).as_ref(),
            "Go to absences"
        );
    }

    #[test]
    fn i18n_extra_hours_absence_hint_match_czech_reference() {
        let i18n = generate(Locale::Cs);
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHint).as_ref(),
            "Pro celé dovolené nebo nepřítomnosti prosím použij masku nepřítomností."
        );
        assert_eq!(
            i18n.t(Key::ExtraHoursAbsenceHintLink).as_ref(),
            "Na nepřítomnosti"
        );
    }

    // ===== Phase 9 — Booking-Warning i18n Tests =====

    #[test]
    fn i18n_booking_warning_keys_present_in_all_locales() {
        // Locks the contract: every booking-warning key has a translation in
        // every locale and never falls back to "??". Primary guard against
        // Pitfall 1 (Locale::En-instead-of-Locale::De in de.rs).
        for locale in [Locale::En, Locale::De, Locale::Cs] {
            let i18n = generate(locale);
            for key in [
                Key::BookingWarningDialogHeaderSingular,
                Key::BookingWarningDialogHeaderPlural,
                Key::BookingWarningDismiss,
                Key::BookingWarningOnAbsenceDay,
                Key::BookingWarningOnUnavailableDay,
                Key::BookingWarningPaidLimitExceeded,
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
    fn i18n_booking_warning_keys_match_german_reference() {
        // Pitfall-1 guard: de.rs must use Locale::De, not Locale::En.
        // Also guards against dropping the {person} placeholder back to a
        // hardcoded "Mitarbeiter".
        let i18n = generate(Locale::De);
        assert_eq!(
            i18n.t(Key::BookingWarningDismiss).as_ref(),
            "Hinweis schließen"
        );
        assert_eq!(
            i18n.t(Key::BookingWarningDialogHeaderSingular).as_ref(),
            "Hinweis · 1 Konflikt"
        );
        // Guard: {person} placeholder must be present in BookingWarningOnAbsenceDay
        let on_absence_day = i18n.t(Key::BookingWarningOnAbsenceDay);
        assert!(
            on_absence_day.as_ref().contains("{person}"),
            "BookingWarningOnAbsenceDay de.rs must contain {{person}} placeholder, got: `{}`",
            on_absence_day
        );
    }

    /// Guard against reintroduction of the removed Copy-Week frontend dead code (D-06).
    /// If any of these symbols reappear in the frontend sources, this test fails immediately.
    #[test]
    fn no_copy_week_in_frontend_source() {
        // Symbols are split to avoid self-matching in the source text of this test
        let copy_from_prev = ["Copy", "FromPreviousWeek"].concat();
        let copy_from_prev_fn = ["copy", "_from_previous_week"].concat();
        let copy_week_fn = ["fn ", "copy_week"].concat();
        let take_last_week_key = ["Shiftplan", "TakeLastWeek"].concat();

        let shiftplan = include_str!("../page/shiftplan.rs");
        let api = include_str!("../api.rs");
        let loader = include_str!("../loader.rs");
        // Only check the non-test part of mod.rs
        let full_modrs = include_str!("mod.rs");
        let modrs = full_modrs
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(full_modrs);

        assert!(
            !shiftplan.contains(&copy_from_prev),
            "dead copy-week action variant must not exist in shiftplan.rs"
        );
        assert!(
            !loader.contains(&copy_from_prev_fn),
            "dead copy-week loader fn must not exist"
        );
        assert!(
            !api.contains(&copy_week_fn),
            "dead copy-week api fn must not exist"
        );
        assert!(
            !modrs.contains(&take_last_week_key),
            "dead i18n key for copy-week must not exist in non-test section of mod.rs"
        );
    }
}
