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
}
impl Locale {
    pub fn from_str(locale: &str) -> Self {
        match locale {
            "en" => Locale::En,
            "de" => Locale::De,
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

    // Shiftplan
    ShiftplanCalendarWeek,
    ShiftplanTakeLastWeek,
    ShiftplanEditAs,
    ShiftplanYouAre,
    ConflictBookingsHeader,
    PersonalCalendarExport,
    UnsufficientlyBookedCalendarExport,

    // Weekly overview page
    WeeklyOverviewTitle,
    AvailableRequiredHours,
    MissingHours,

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
    CategoryCustom,

    VacationDaysLabel,
    VacationCarryoverLabel,

    ShowDetails,
    HideDetails,

    Hours,
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
    DaysPerWeekLabel,
    VacationEntitlementsPerYearLabel,
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
}

pub fn generate(locale: Locale) -> I18n<Key, Locale> {
    let mut i18n = I18n::new(locale, Locale::En);

    match locale {
        Locale::En => en::add_i18n_en(&mut i18n),
        Locale::De => de::add_i18n_de(&mut i18n),
    }

    i18n
}

pub type I18nType = I18n<Key, Locale>;
