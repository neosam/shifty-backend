#[cfg(test)]
mod integration_test;

use std::sync::Arc;

use dao_impl_sqlite::{
    absence::AbsenceDaoImpl,
    billing_period::BillingPeriodDaoImpl,
    billing_period_sales_person::BillingPeriodSalesPersonDaoImpl, booking::BookingDaoImpl,
    carryover::CarryoverDaoImpl, employee_work_details::EmployeeWorkDetailsDaoImpl,
    extra_hours::ExtraHoursDaoImpl, feature_flag::FeatureFlagDaoImpl,
    sales_person::SalesPersonDaoImpl,
    sales_person_unavailable::SalesPersonUnavailableDaoImpl, session::SessionDaoImpl,
    shiftplan_report::ShiftplanReportDaoImpl, slot::SlotDaoImpl, special_day::SpecialDayDaoImpl,
    BasicDaoImpl, PermissionDaoImpl, TransactionDaoImpl, TransactionImpl,
};
use service::pdf_export::PdfExportScheduler as _;
use service::scheduler::SchedulerService;
use service_impl::pdf_export_scheduler::{
    PdfExportSchedulerDeps, PdfExportSchedulerImpl, ProductionWebDavUploadFactory,
    WebDavUploadFactory,
};
use service_impl::{
    carryover::CarryoverServiceDeps,
    permission::PermissionServiceDeps,
    shiftplan::{ShiftplanViewServiceDeps, ShiftplanViewServiceImpl},
    shiftplan_catalog::{ShiftplanServiceDeps, ShiftplanServiceImpl},
};
use sqlx::SqlitePool;
#[cfg(feature = "json_logging")]
use tracing_subscriber::fmt::format::FmtSpan;

type UserService = service_impl::UserServiceImpl;
type Context = Option<Arc<str>>;
type Transaction = TransactionImpl;
type TransactionDao = TransactionDaoImpl;
type PermissionDao = PermissionDaoImpl;
type SlotDao = SlotDaoImpl;
type SalesPersonDao = SalesPersonDaoImpl;
type BookingDao = BookingDaoImpl;
type SpecialDayDao = SpecialDayDaoImpl;
type SalesPersonUnavailableDao = SalesPersonUnavailableDaoImpl;
type SessionDao = SessionDaoImpl;
type ShiftplanReportDao = ShiftplanReportDaoImpl;
type AbsenceDao = AbsenceDaoImpl;
type ExtraHoursDao = ExtraHoursDaoImpl;
type FeatureFlagDao = FeatureFlagDaoImpl;
type MigrationSourceDao = dao_impl_sqlite::migration_source::MigrationSourceDaoImpl;
type CarryoverDao = CarryoverDaoImpl;
type EmployeeWorkDetailsDao = EmployeeWorkDetailsDaoImpl;
type WeekMessageDao = dao_impl_sqlite::week_message::WeekMessageDaoImpl;
type WeekStatusDao = dao_impl_sqlite::week_status::WeekStatusDaoImpl;
type BillingPeriodDao = BillingPeriodDaoImpl;
type BillingPeriodSalesPersonDao = BillingPeriodSalesPersonDaoImpl;
type TextTemplateDao = dao_impl_sqlite::text_template::TextTemplateDaoImpl;
type UserInvitationDao = dao_impl_sqlite::user_invitation::UserInvitationDaoImpl;
type ToggleDao = dao_impl_sqlite::toggle::ToggleDaoImpl;
type ShiftplanDao = dao_impl_sqlite::shiftplan::ShiftplanDaoImpl;
// Phase 28 (VAC-OFFSET-01): Basic-Offset-DAO für den Urlaubsanspruch-Offset.
type VacationEntitlementOffsetDao =
    dao_impl_sqlite::vacation_entitlement_offset::VacationEntitlementOffsetDaoImpl;
// Phase 48 (EXP-02/EXP-03): Basic-Config-DAO für den Nextcloud-PDF-Export.
type PdfExportConfigDao = dao_impl_sqlite::pdf_export_config::PdfExportConfigDaoImpl;

type ConfigService = service_impl::config::ConfigServiceImpl;

pub struct PermissionServiceDependencies;
impl PermissionServiceDeps for PermissionServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PermissionDao = PermissionDao;
    type UserService = UserService;
}
type PermissionService = service_impl::PermissionServiceImpl<PermissionServiceDependencies>;

pub struct SessionServiceDependencies;
impl service_impl::session::SessionServiceDeps for SessionServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SessionDao = SessionDao;
    type ClockService = service_impl::clock::ClockServiceImpl;
    type UuidService = service_impl::uuid_service::UuidServiceImpl;
}
type SessionService = service_impl::session::SessionServiceImpl<SessionServiceDependencies>;

pub struct UserInvitationServiceDependencies;
impl service_impl::user_invitation::UserInvitationServiceDeps for UserInvitationServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type UserInvitationDao = UserInvitationDao;
    type PermissionDao = PermissionDao;
    type PermissionService = PermissionService;
    type SessionService = SessionService;
    type UuidService = service_impl::uuid_service::UuidServiceImpl;
    type TransactionDao = TransactionDao;
}
type UserInvitationService = service_impl::user_invitation::UserInvitationServiceImpl<UserInvitationServiceDependencies>;

type ClockService = service_impl::clock::ClockServiceImpl;
type UuidService = service_impl::uuid_service::UuidServiceImpl;
type SlotService = service_impl::slot::SlotServiceImpl<
    SlotDao,
    PermissionService,
    ClockService,
    UuidService,
    TransactionDao,
>;

pub struct SalesPersonServiceDependencies;
impl service_impl::sales_person::SalesPersonServiceDeps for SalesPersonServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SalesPersonDao = SalesPersonDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type SalesPersonService =
    service_impl::sales_person::SalesPersonServiceImpl<SalesPersonServiceDependencies>;
type SpecialDayService = service_impl::special_days::SpecialDayServiceImpl<
    SpecialDayDao,
    PermissionService,
    ClockService,
    UuidService,
>;

pub struct SalesPersonUnavailableServiceDependencies;
impl service_impl::sales_person_unavailable::SalesPersonUnavailableServiceDeps
    for SalesPersonUnavailableServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type SalesPersonUnavailableDao = SalesPersonUnavailableDao;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type SalesPersonUnavailableService =
    service_impl::sales_person_unavailable::SalesPersonUnavailableServiceImpl<
        SalesPersonUnavailableServiceDependencies,
    >;
type SalesPersonShiftplanDao = dao_impl_sqlite::sales_person_shiftplan::SalesPersonShiftplanDaoImpl;

pub struct SalesPersonShiftplanServiceDependencies;
impl service_impl::sales_person_shiftplan::SalesPersonShiftplanServiceDeps
    for SalesPersonShiftplanServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type SalesPersonShiftplanDao = SalesPersonShiftplanDao;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type SalesPersonShiftplanService = service_impl::sales_person_shiftplan::SalesPersonShiftplanServiceImpl<SalesPersonShiftplanServiceDependencies>;

pub struct BookingServiceDependencies;
impl service_impl::booking::BookingServiceDeps for BookingServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type BookingDao = BookingDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type SalesPersonService = SalesPersonService;
    type SlotService = SlotService;
    type SalesPersonShiftplanService = SalesPersonShiftplanService;
    type TransactionDao = TransactionDao;
}
type BookingService = service_impl::booking::BookingServiceImpl<BookingServiceDependencies>;

pub struct CustomExtraHoursServiceDependencies;
impl service_impl::custom_extra_hours::CustomExtraHoursDeps
    for CustomExtraHoursServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type CustomExtraHoursDao = dao_impl_sqlite::custom_extra_hours::CustomExtraHoursDaoImpl;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type CustomExtraHoursService = service_impl::custom_extra_hours::CustomExtraHoursServiceImpl<
    CustomExtraHoursServiceDependencies,
>;

pub struct ShiftplanReportServiceDependencies;
impl service_impl::shiftplan_report::ShiftplanReportServiceDeps
    for ShiftplanReportServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanReportDao = ShiftplanReportDao;
    type TransactionDao = TransactionDao;
}
type ShiftplanReportService =
    service_impl::shiftplan_report::ShiftplanReportServiceImpl<ShiftplanReportServiceDependencies>;

pub struct BookingInformationServiceDependencies;
impl service_impl::booking_information::BookingInformationServiceDeps
    for BookingInformationServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanReportService = ShiftplanReportService;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type ReportingService = ReportingService;
    type SpecialDayService = SpecialDayService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
    type EmployeeWorkDetailsService = WorkingHoursService;
    // VFA-01 (D-26-01/D-26-03): AbsenceService wired into BookingInformationService.
    // absence_service is constructed at line ~821 (before booking_information_service at ~909)
    // — no construction-order change needed. No DI cycle: AbsenceService does not consume
    // BookingInformationService (D-Phase3-18 regression-lock).
    type AbsenceService = AbsenceService;
}
type BookingInformationService = service_impl::booking_information::BookingInformationServiceImpl<
    BookingInformationServiceDependencies,
>;

pub struct BookingLogServiceDependencies;
impl service_impl::booking_log::BookingLogServiceDeps for BookingLogServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type BookingLogDao = dao_impl_sqlite::booking_log::BookingLogDaoImpl;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type BookingLogService = service_impl::booking_log::BookingLogServiceImpl<
    BookingLogServiceDependencies,
>;

pub struct AbsenceServiceDependencies;
impl service_impl::absence::AbsenceServiceDeps for AbsenceServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type AbsenceDao = AbsenceDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    // Phase 2 plan 02-02: SpecialDayService + EmployeeWorkDetailsService
    // werden fuer derive_hours_for_range benoetigt.
    type SpecialDayService = SpecialDayService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type TransactionDao = TransactionDao;
    // Phase 3 plan 03-03 (D-Phase3-08): AbsenceService konsumiert
    // BookingService + SalesPersonUnavailableService + SlotService für den
    // Forward-Warning-Loop in create/update. Service-Tier-Konvention:
    // AbsenceService ist Business-Logic, BookingService/SalesPersonUnavailable/
    // Slot sind Basic — Direction Business-Logic ↑ konsumiert Basic ↓; kein
    // Cycle, da BookingService nichts von AbsenceService weiß (D-Phase3-18
    // Regression-Lock).
    type BookingService = BookingService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type SlotService = SlotService;
}
// type AbsenceService = service_impl::absence::AbsenceServiceImpl<AbsenceServiceDependencies>;
type AbsenceService =
    service_impl::absence::AbsenceServiceImpl<AbsenceServiceDependencies>;

// Phase 28 (VAC-OFFSET-01, D-28-06): VacationEntitlementOffsetServiceImpl ist
// Basic-Tier (Entity-Manager) — nur DAO + Permission + Clock + Uuid +
// Transaction. Konsumiert KEINEN Domain-Service, damit kein Zyklus mit dem
// Business-Logic VacationBalanceService entsteht.
pub struct VacationEntitlementOffsetServiceDependencies;
impl service_impl::vacation_entitlement_offset::VacationEntitlementOffsetServiceDeps
    for VacationEntitlementOffsetServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type VacationEntitlementOffsetDao = VacationEntitlementOffsetDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type VacationEntitlementOffsetService =
    service_impl::vacation_entitlement_offset::VacationEntitlementOffsetServiceImpl<
        VacationEntitlementOffsetServiceDependencies,
    >;

// Phase 48 (EXP-02/EXP-03, D-48-BASIC): PdfExportConfigServiceImpl ist
// Basic-Tier — konsumiert AUSSCHLIESSLICH PdfExportConfigDao + Permission +
// Clock + Uuid + Transaction. Kein Domain-Service als Dep.
pub struct PdfExportConfigServiceDependencies;
impl service_impl::pdf_export_config::PdfExportConfigServiceDeps
    for PdfExportConfigServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type PdfExportConfigDao = PdfExportConfigDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type PdfExportConfigService = service_impl::pdf_export_config::PdfExportConfigServiceImpl<
    PdfExportConfigServiceDependencies,
>;

// Phase 48 Plan 04 (EXP-01/EXP-03): PdfExportSchedulerImpl ist BL-Tier —
// konsumiert PdfExportConfigService (Basic) + ShiftplanViewService +
// ShiftplanService (catalog) + SalesPersonService + PermissionService +
// ClockService + TransactionDao + eine `WebDavUploadFactory` (Prod: baut echte
// WebDavClient pro Lauf; Tests injizieren einen Mock).
pub struct PdfExportSchedulerDependencies;
impl PdfExportSchedulerDeps for PdfExportSchedulerDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PdfExportConfigService = PdfExportConfigService;
    // Phase 49 Plan 03 (Wave 1 landed): der Scheduler konsumiert jetzt den
    // PdfShiftplanService (BL-Kern), NICHT mehr ShiftplanViewService +
    // SalesPersonService direkt — dedupe mit dem REST-Handler-Pfad.
    type PdfShiftplanService = PdfShiftplanService;
    type ShiftplanService = ShiftplanCatalogService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type TransactionDao = TransactionDao;
}
type PdfExportSchedulerService = PdfExportSchedulerImpl<PdfExportSchedulerDependencies>;

// Phase 49 (PDF-03/PDF-04/PDF-05): PdfShiftplanServiceImpl ist BL-Tier —
// konsumiert ShiftplanViewService (Read-Aggregat) + SalesPersonService
// (Basic) + WeekStatusService (Basic) + PermissionService + TransactionDao.
// Wird sowohl vom REST-Handler (`GET /shiftplan/{id}/{y}/{w}/pdf`, Wave 2)
// als auch vom Scheduler-Refactor (Plan 03) über den DRY-Kern
// `render_week_pdf` konsumiert.
pub struct PdfShiftplanServiceDependencies;
impl service_impl::pdf_shiftplan::PdfShiftplanServiceDeps for PdfShiftplanServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanViewService = ShiftplanViewServiceImpl<ShiftplanViewServiceDependencies>;
    type SalesPersonService = SalesPersonService;
    type WeekStatusService = WeekStatusService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type PdfShiftplanService =
    service_impl::pdf_shiftplan::PdfShiftplanServiceImpl<PdfShiftplanServiceDependencies>;

// Phase 8 (D-04, 08-02-PLAN.md): VacationBalanceServiceImpl ist BL-Tier.
// Konsumiert AbsenceService + EmployeeWorkDetails (alias WorkingHoursService)
// + CarryoverService + SalesPersonService + VacationEntitlementOffsetService
// (Phase 28, Basic) + PermissionService + ClockService + TransactionDao.
// Type-Alias-Namen (WorkingHoursService statt EmployeeWorkDetailsService)
// folgen der bestehenden main.rs-Konvention.
pub struct VacationBalanceServiceDependencies;
impl service_impl::vacation_balance::VacationBalanceServiceDeps
    for VacationBalanceServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type AbsenceService = AbsenceService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type CarryoverService = CarryoverService;
    type SalesPersonService = SalesPersonService;
    type VacationEntitlementOffsetService = VacationEntitlementOffsetService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type TransactionDao = TransactionDao;
}
type VacationBalanceService = service_impl::vacation_balance::VacationBalanceServiceImpl<
    VacationBalanceServiceDependencies,
>;

pub struct ExtraHoursServiceDependencies;
impl service_impl::extra_hours::ExtraHoursServiceDeps for ExtraHoursServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = ExtraHoursDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type CustomExtraHoursService = CustomExtraHoursService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ExtraHoursService =
    service_impl::extra_hours::ExtraHoursServiceImpl<ExtraHoursServiceDependencies>;

pub struct CarryoverServiceDependencies;
impl CarryoverServiceDeps for CarryoverServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type CarryoverDao = CarryoverDao;
    type TransactionDao = TransactionDao;
}

type CarryoverService = service_impl::carryover::CarryoverServiceImpl<CarryoverServiceDependencies>;

type IcalService = service_impl::ical::IcalServiceImpl;

pub struct ShiftplanViewServiceDependencies;
impl ShiftplanViewServiceDeps for ShiftplanViewServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type SpecialDayService = SpecialDayService;
    type ShiftplanService = ShiftplanCatalogService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
    // NEU für Phase 3 (D-Phase3-09):
    type AbsenceService = AbsenceService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    // NEU für Phase 51 (D-51-07): Stichtag-Gate für ShortDay-Slot-Kürzung.
    type ToggleService = ToggleService;
}

pub struct BlockServiceDependencies;
impl service_impl::block::BlockServiceDeps for BlockServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type SalesPersonService = SalesPersonService;
    type ClockService = ClockService;
    type IcalService = IcalService;
    type TransactionDao = TransactionDao;
    type ShiftplanViewService = ShiftplanViewServiceImpl<ShiftplanViewServiceDependencies>;
    type ConfigService = ConfigService;
}
type BlockService = service_impl::block::BlockServiceImpl<BlockServiceDependencies>;

pub struct ReportingServiceDependencies;
impl service_impl::reporting::ReportingServiceDeps for ReportingServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursService = ExtraHoursService;
    type ShiftplanReportService = ShiftplanReportService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type SalesPersonService = SalesPersonService;
    type CarryoverService = CarryoverService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    // Phase 8.4: ReportingService summiert beide Quellen additiv
    // (AbsenceService-derived hours + lebende extra_hours). Kein
    // Feature-Flag-Switch mehr — FeatureFlagService-Dep entfernt (M-03).
    type AbsenceService = AbsenceService;
    type TransactionDao = TransactionDao;
    // Phase 25: holiday derive-on-read deps.
    type SpecialDayService = SpecialDayService;
    type ToggleService = ToggleService;
}
type ReportingService = service_impl::reporting::ReportingServiceImpl<ReportingServiceDependencies>;

pub struct WorkingHoursServiceDependencies;
impl service_impl::employee_work_details::EmployeeWorkDetailsServiceDeps
    for WorkingHoursServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type EmployeeWorkDetailsDao = EmployeeWorkDetailsDao;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type WorkingHoursService = service_impl::employee_work_details::EmployeeWorkDetailsServiceImpl<
    WorkingHoursServiceDependencies,
>;

pub struct WeekMessageServiceDependencies;
impl service_impl::week_message::WeekMessageServiceDeps for WeekMessageServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type WeekMessageDao = WeekMessageDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type WeekMessageService =
    service_impl::week_message::WeekMessageServiceImpl<WeekMessageServiceDependencies>;

// Basic-tier KW status service (D-39-12): DAO + Permission + Clock + Uuid +
// Transaction only, no domain-service dependency. Wired next to week_message,
// before the business-logic layer.
pub struct WeekStatusServiceDependencies;
impl service_impl::week_status::WeekStatusServiceDeps for WeekStatusServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type WeekStatusDao = WeekStatusDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type WeekStatusService =
    service_impl::week_status::WeekStatusServiceImpl<WeekStatusServiceDependencies>;

pub struct ShiftplanEditServiceDependencies;
impl service_impl::shiftplan_edit::ShiftplanEditServiceDeps for ShiftplanEditServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PermissionService = PermissionService;
    type SlotService = SlotService;
    type BookingService = BookingService;
    type CarryoverService = CarryoverService;
    type ReportingService = ReportingService;
    type SalesPersonService = SalesPersonService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type ExtraHoursService = ExtraHoursService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    // NEU für Phase 3 (D-Phase3-06):
    type AbsenceService = AbsenceService;
    // D-24-08: ToggleService für paid_limit_hard_enforcement-Prüfung
    type ToggleService = ToggleService;
    // NEU für Phase 40 (D-40-01): Wochen-Sperre-Gate liest den Lock-Status.
    type WeekStatusService = WeekStatusService;
}
type ShiftplanEditService =
    service_impl::shiftplan_edit::ShiftplanEditServiceImpl<ShiftplanEditServiceDependencies>;

pub struct SchedulerServiceDependencies;
impl service_impl::scheduler::SchedulerServiceDeps for SchedulerServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanEditService = ShiftplanEditService;
}
type SchedulerServiceImpl =
    service_impl::scheduler::SchedulerServiceImpl<SchedulerServiceDependencies>;

pub struct BillingPeriodServiceDependencies;
impl service_impl::billing_period::BillingPeriodServiceDeps for BillingPeriodServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type BillingPeriodDao = BillingPeriodDao;
    type BillingPeriodSalesPersonDao = BillingPeriodSalesPersonDao;
    type SalesPersonService = SalesPersonService;
    type PermissionService = PermissionService;
    type UuidService = UuidService;
    type ClockService = ClockService;
    type TransactionDao = TransactionDao;
}
type BillingPeriodService =
    service_impl::billing_period::BillingPeriodServiceImpl<BillingPeriodServiceDependencies>;

pub struct BillingPeriodReportServiceDependencies;
impl service_impl::billing_period_report::BillingPeriodReportServiceDeps
    for BillingPeriodReportServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type BillingPeriodService = BillingPeriodService;
    type ReportingService = ReportingService;
    type SalesPersonService = SalesPersonService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type TextTemplateService = TextTemplateService;
    type PermissionService = PermissionService;
    type UuidService = UuidService;
    type ClockService = ClockService;
    type TransactionDao = TransactionDao;
}
type BillingPeriodReportService =
    service_impl::billing_period_report::BillingPeriodReportServiceImpl<
        BillingPeriodReportServiceDependencies,
    >;

pub struct BlockReportServiceDependencies;
impl service_impl::block_report::BlockReportServiceDeps for BlockReportServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type BlockService = BlockService;
    type TextTemplateService = TextTemplateService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type TransactionDao = TransactionDao;
}
type BlockReportService = service_impl::block_report::BlockReportServiceImpl<BlockReportServiceDependencies>;

pub struct TextTemplateServiceDependencies;
impl service_impl::text_template::TextTemplateServiceDeps for TextTemplateServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type TextTemplateDao = TextTemplateDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type TextTemplateService = service_impl::text_template::TextTemplateServiceImpl<TextTemplateServiceDependencies>;

pub struct ToggleServiceDependencies;
impl service_impl::toggle::ToggleServiceDeps for ToggleServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ToggleDao = ToggleDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type ToggleService = service_impl::toggle::ToggleServiceImpl<ToggleServiceDependencies>;

pub struct FeatureFlagServiceDependencies;
impl service_impl::feature_flag::FeatureFlagServiceDeps for FeatureFlagServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type FeatureFlagDao = FeatureFlagDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type FeatureFlagService =
    service_impl::feature_flag::FeatureFlagServiceImpl<FeatureFlagServiceDependencies>;

// Phase 8.5 (Plan 03) — AbsenceConversionService DI.
// BL-Tier (D-03): 6 Deps (extra_hours_dao + absence_dao + migration_source_dao
// + extra_hours_service + permission_service + transaction_dao).
// MUSS nach Basic-Services + MigrationSourceDao konstruiert werden (Service-Tier-Konvention).
pub struct AbsenceConversionServiceDependencies;
impl service_impl::absence_conversion::AbsenceConversionServiceDeps
    for AbsenceConversionServiceDependencies
{
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = ExtraHoursDao;
    type AbsenceDao = AbsenceDao;
    type MigrationSourceDao = MigrationSourceDao;
    type ExtraHoursService = ExtraHoursService;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type AbsenceConversionService = service_impl::absence_conversion::AbsenceConversionServiceImpl<
    AbsenceConversionServiceDependencies,
>;

pub struct ShiftplanCatalogServiceDependencies;
impl ShiftplanServiceDeps for ShiftplanCatalogServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ShiftplanDao = ShiftplanDao;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ShiftplanCatalogService = ShiftplanServiceImpl<ShiftplanCatalogServiceDependencies>;

#[derive(Clone)]
pub struct RestStateImpl {
    user_service: Arc<UserService>,
    session_service: Arc<SessionService>,
    permission_service: Arc<PermissionService>,
    slot_service: Arc<SlotService>,
    sales_person_service: Arc<SalesPersonService>,
    special_day_service: Arc<SpecialDayService>,
    sales_person_unavailable_service: Arc<SalesPersonUnavailableService>,
    booking_service: Arc<BookingService>,
    custom_extra_hours_service: Arc<CustomExtraHoursService>,
    booking_information_service: Arc<BookingInformationService>,
    booking_log_service: Arc<BookingLogService>,
    reporting_service: Arc<ReportingService>,
    working_hours_service: Arc<WorkingHoursService>,
    absence_service: Arc<AbsenceService>,
    vacation_balance_service: Arc<VacationBalanceService>,
    extra_hours_service: Arc<ExtraHoursService>,
    shiftplan_edit_service: Arc<ShiftplanEditService>,
    block_service: Arc<BlockService>,
    shiftplan_service: Arc<ShiftplanCatalogService>,
    shiftplan_view_service: Arc<ShiftplanViewServiceImpl<ShiftplanViewServiceDependencies>>,
    week_message_service: Arc<WeekMessageService>,
    week_status_service: Arc<WeekStatusService>,
    billing_period_service: Arc<BillingPeriodService>,
    billing_period_report_service: Arc<BillingPeriodReportService>,
    block_report_service: Arc<BlockReportService>,
    text_template_service: Arc<TextTemplateService>,
    user_invitation_service: Arc<UserInvitationService>,
    toggle_service: Arc<ToggleService>,
    sales_person_shiftplan_service: Arc<SalesPersonShiftplanService>,
    // Phase 8 Plan 08-07 Gap-Closure (Task 2): exposed for REST-Layer
    // (`GET /feature-flag/{key}`). Intern genutzt von ReportingService +
    // ExtraHoursService; jetzt auch via `RestStateDef::feature_flag_service`.
    feature_flag_service: Arc<FeatureFlagService>,
    // Phase 8.5 (Plan 03): HR-gated per-Row-Convert extra_hours -> absence_period.
    absence_conversion_service: Arc<AbsenceConversionService>,
    // Phase 28 (VAC-OFFSET-01): HR-gated REST-CRUD für den Urlaubsanspruch-Offset.
    vacation_entitlement_offset_service: Arc<VacationEntitlementOffsetService>,
    // Phase 48 (EXP-02/EXP-03): admin-gated REST-CRUD für die PDF-Export-Konfig.
    pdf_export_config_service: Arc<PdfExportConfigService>,
    // Phase 48 Plan 04 (EXP-01/EXP-03): Cron-getriebener Nextcloud-Push.
    pdf_export_scheduler: Arc<PdfExportSchedulerService>,
    // Phase 49 (PDF-03/PDF-04/PDF-05): BL-Tier PDF-Shiftplan-Assembler;
    // konsumiert vom On-Demand-Download-Endpoint (Wave 2) und vom
    // Scheduler-Refactor (Plan 03).
    pdf_shiftplan_service: Arc<PdfShiftplanService>,
    basic_dao: Arc<BasicDaoImpl>,
}
impl rest::RestStateDef for RestStateImpl {
    type UserService = UserService;
    type SessionService = SessionService;
    type PermissionService = PermissionService;
    type SlotService = SlotService;
    type SalesPersonService = SalesPersonService;
    type SpecialDayService = SpecialDayService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type BookingService = BookingService;
    type CustomExtraHoursService = CustomExtraHoursService;
    type BookingInformationService = BookingInformationService;
    type BookingLogService = BookingLogService;
    type ReportingService = ReportingService;
    type WorkingHoursService = WorkingHoursService;
    type AbsenceService = AbsenceService;
    type VacationBalanceService = VacationBalanceService;
    type ExtraHoursService = ExtraHoursService;
    type ShiftplanEditService = ShiftplanEditService;
    type BlockService = BlockService;
    type ShiftplanService = ShiftplanCatalogService;
    type ShiftplanViewService = ShiftplanViewServiceImpl<ShiftplanViewServiceDependencies>;
    type WeekMessageService = WeekMessageService;
    type WeekStatusService = WeekStatusService;
    type BillingPeriodService = BillingPeriodService;
    type BillingPeriodReportService = BillingPeriodReportService;
    type BlockReportService = BlockReportService;
    type TextTemplateService = TextTemplateService;
    type UserInvitationService = UserInvitationService;
    type ToggleService = ToggleService;
    type SalesPersonShiftplanService = SalesPersonShiftplanService;
    type FeatureFlagService = FeatureFlagService;
    type AbsenceConversionService = AbsenceConversionService;
    type VacationEntitlementOffsetService = VacationEntitlementOffsetService;
    type PdfExportConfigService = PdfExportConfigService;
    type PdfExportScheduler = PdfExportSchedulerService;
    type PdfShiftplanService = PdfShiftplanService;
    type BasicDao = BasicDaoImpl;

    fn backend_version(&self) -> Arc<str> {
        Arc::from(env!("CARGO_PKG_VERSION"))
    }

    fn user_service(&self) -> Arc<Self::UserService> {
        self.user_service.clone()
    }
    fn session_service(&self) -> Arc<Self::SessionService> {
        self.session_service.clone()
    }
    fn permission_service(&self) -> Arc<Self::PermissionService> {
        self.permission_service.clone()
    }
    fn slot_service(&self) -> Arc<Self::SlotService> {
        self.slot_service.clone()
    }
    fn sales_person_service(&self) -> Arc<Self::SalesPersonService> {
        self.sales_person_service.clone()
    }
    fn special_day_service(&self) -> Arc<Self::SpecialDayService> {
        self.special_day_service.clone()
    }
    fn sales_person_unavailable_service(&self) -> Arc<Self::SalesPersonUnavailableService> {
        self.sales_person_unavailable_service.clone()
    }
    fn booking_service(&self) -> Arc<Self::BookingService> {
        self.booking_service.clone()
    }
    fn custom_extra_hours_service(&self) -> Arc<Self::CustomExtraHoursService> {
        self.custom_extra_hours_service.clone()
    }
    fn booking_information_service(&self) -> Arc<Self::BookingInformationService> {
        self.booking_information_service.clone()
    }
    fn booking_log_service(&self) -> Arc<Self::BookingLogService> {
        self.booking_log_service.clone()
    }
    fn reporting_service(&self) -> Arc<Self::ReportingService> {
        self.reporting_service.clone()
    }
    fn working_hours_service(&self) -> Arc<Self::WorkingHoursService> {
        self.working_hours_service.clone()
    }
    fn absence_service(&self) -> Arc<Self::AbsenceService> {
        self.absence_service.clone()
    }
    fn vacation_balance_service(&self) -> Arc<Self::VacationBalanceService> {
        self.vacation_balance_service.clone()
    }
    fn extra_hours_service(&self) -> Arc<Self::ExtraHoursService> {
        self.extra_hours_service.clone()
    }
    fn shiftplan_edit_service(&self) -> Arc<Self::ShiftplanEditService> {
        self.shiftplan_edit_service.clone()
    }
    fn block_service(&self) -> Arc<Self::BlockService> {
        self.block_service.clone()
    }

    fn shiftplan_service(&self) -> Arc<Self::ShiftplanService> {
        self.shiftplan_service.clone()
    }
    fn shiftplan_view_service(&self) -> Arc<Self::ShiftplanViewService> {
        self.shiftplan_view_service.clone()
    }
    fn week_message_service(&self) -> Arc<Self::WeekMessageService> {
        self.week_message_service.clone()
    }
    fn week_status_service(&self) -> Arc<Self::WeekStatusService> {
        self.week_status_service.clone()
    }
    fn billing_period_service(&self) -> Arc<Self::BillingPeriodService> {
        self.billing_period_service.clone()
    }
    fn billing_period_report_service(&self) -> Arc<Self::BillingPeriodReportService> {
        self.billing_period_report_service.clone()
    }
    fn block_report_service(&self) -> Arc<Self::BlockReportService> {
        self.block_report_service.clone()
    }
    fn text_template_service(&self) -> Arc<Self::TextTemplateService> {
        self.text_template_service.clone()
    }
    fn user_invitation_service(&self) -> Arc<Self::UserInvitationService> {
        self.user_invitation_service.clone()
    }
    fn toggle_service(&self) -> Arc<Self::ToggleService> {
        self.toggle_service.clone()
    }
    fn sales_person_shiftplan_service(&self) -> Arc<Self::SalesPersonShiftplanService> {
        self.sales_person_shiftplan_service.clone()
    }
    fn feature_flag_service(&self) -> Arc<Self::FeatureFlagService> {
        self.feature_flag_service.clone()
    }
    fn absence_conversion_service(&self) -> Arc<Self::AbsenceConversionService> {
        self.absence_conversion_service.clone()
    }
    fn vacation_entitlement_offset_service(&self) -> Arc<Self::VacationEntitlementOffsetService> {
        self.vacation_entitlement_offset_service.clone()
    }
    fn pdf_export_config_service(&self) -> Arc<Self::PdfExportConfigService> {
        self.pdf_export_config_service.clone()
    }
    fn pdf_export_scheduler(&self) -> Arc<Self::PdfExportScheduler> {
        self.pdf_export_scheduler.clone()
    }
    fn pdf_shiftplan_service(&self) -> Arc<Self::PdfShiftplanService> {
        self.pdf_shiftplan_service.clone()
    }
    fn basic_dao(&self) -> Arc<Self::BasicDao> {
        self.basic_dao.clone()
    }
}
impl RestStateImpl {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        let transaction_dao = Arc::new(TransactionDao::new(pool.clone()));
        let permission_dao = Arc::new(PermissionDao::new(pool.clone()));
        let slot_dao = SlotDao::new(pool.clone());
        let carryover_dao = Arc::new(CarryoverDao::new(pool.clone()));
        let vacation_entitlement_offset_dao =
            Arc::new(VacationEntitlementOffsetDao::new(pool.clone()));
        let pdf_export_config_dao = Arc::new(PdfExportConfigDao::new(pool.clone()));
        let sales_person_dao = SalesPersonDao::new(pool.clone());
        let booking_dao = BookingDao::new(pool.clone());
        let booking_log_dao = Arc::new(dao_impl_sqlite::booking_log::BookingLogDaoImpl);
        let absence_dao = Arc::new(AbsenceDao::new(pool.clone()));
        let extra_hours_dao = Arc::new(ExtraHoursDao::new(pool.clone()));
        let shiftplan_report_dao = Arc::new(ShiftplanReportDao::new(pool.clone()));
        let working_hours_dao = Arc::new(EmployeeWorkDetailsDao::new(pool.clone()));
        let special_day_dao = SpecialDayDao::new(pool.clone());
        let session_dao = SessionDao::new(pool.clone());
        let custom_extra_hours_dao =
            Arc::new(dao_impl_sqlite::custom_extra_hours::CustomExtraHoursDaoImpl);
        let text_template_dao = Arc::new(TextTemplateDao::new(pool.clone()));
        let user_invitation_dao = Arc::new(UserInvitationDao::new(pool.clone()));

        // Always authenticate with DEVUSER during development.
        // This is used to test the permission service locally without a login service.
        //
        let user_service = service_impl::UserServiceImpl;
        let user_service = Arc::new(user_service);
        let permission_service = Arc::new(service_impl::PermissionServiceImpl {
            permission_dao: permission_dao.clone(),
            user_service: user_service.clone(),
        });
        let clock_service = Arc::new(service_impl::clock::ClockServiceImpl);
        let uuid_service = Arc::new(service_impl::uuid_service::UuidServiceImpl);
        let session_service = Arc::new(service_impl::session::SessionServiceImpl {
            session_dao: Arc::new(session_dao),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
        });
        let config_service = Arc::new(service_impl::config::ConfigServiceImpl);
        let slot_service = Arc::new(service_impl::slot::SlotServiceImpl::new(
            slot_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
            transaction_dao.clone(),
        ));
        let sales_person_service = Arc::new(service_impl::sales_person::SalesPersonServiceImpl {
            sales_person_dao: sales_person_dao.into(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let special_day_service = Arc::new(service_impl::special_days::SpecialDayServiceImpl::new(
            special_day_dao.into(),
            permission_service.clone(),
            clock_service.clone(),
            uuid_service.clone(),
        ));
        let sales_person_unavailable_service = Arc::new(
            service_impl::sales_person_unavailable::SalesPersonUnavailableServiceImpl {
                sales_person_unavailable_dao: Arc::new(SalesPersonUnavailableDao::new(
                    pool.clone(),
                )),
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let sales_person_shiftplan_dao = Arc::new(SalesPersonShiftplanDao::new(pool.clone()));
        let sales_person_shiftplan_service = Arc::new(
            service_impl::sales_person_shiftplan::SalesPersonShiftplanServiceImpl {
                sales_person_shiftplan_dao,
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let booking_service = Arc::new(service_impl::booking::BookingServiceImpl {
            transaction_dao: transaction_dao.clone(),
            booking_dao: booking_dao.into(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            sales_person_service: sales_person_service.clone(),
            slot_service: slot_service.clone(),
            sales_person_shiftplan_service: sales_person_shiftplan_service.clone(),
        });
        let booking_log_service = Arc::new(service_impl::booking_log::BookingLogServiceImpl {
            booking_log_dao,
            permission_service: permission_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let custom_extra_hours_service = Arc::new(
            service_impl::custom_extra_hours::CustomExtraHoursServiceImpl {
                custom_extra_hours_dao,
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        // working_hours_service muss VOR absence_service gebaut werden,
        // weil AbsenceServiceImpl seit Plan 02-02 employee_work_details_service
        // als Dependency haelt (derive_hours_for_range Per-Tag-Vertrags-Lookup).
        let working_hours_service = Arc::new(
            service_impl::employee_work_details::EmployeeWorkDetailsServiceImpl {
                employee_work_details_dao: working_hours_dao,
                sales_person_service: sales_person_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        let absence_service = Arc::new(service_impl::absence::AbsenceServiceImpl {
            absence_dao: absence_dao.clone(),
            permission_service: permission_service.clone(),
            sales_person_service: sales_person_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            special_day_service: special_day_service.clone(),
            employee_work_details_service: working_hours_service.clone(),
            transaction_dao: transaction_dao.clone(),
            // Phase 3 plan 03-03: Forward-Warning-Loop-Deps (D-Phase3-08).
            // Konstruktionsreihenfolge: booking_service (Z. 699),
            // sales_person_unavailable_service (Z. 678), slot_service
            // (Z. 658) sind alle VOR absence_service (hier) gebaut —
            // Tier-Konform (Basic vor Business-Logic).
            booking_service: booking_service.clone(),
            sales_person_unavailable_service: sales_person_unavailable_service.clone(),
            slot_service: slot_service.clone(),
        });
        let feature_flag_dao = Arc::new(FeatureFlagDao::new(pool.clone()));
        let feature_flag_service: Arc<FeatureFlagService> =
            Arc::new(service_impl::feature_flag::FeatureFlagServiceImpl {
                feature_flag_dao: feature_flag_dao.clone(),
                permission_service: permission_service.clone(),
                transaction_dao: transaction_dao.clone(),
            });
        let extra_hours_service = Arc::new(service_impl::extra_hours::ExtraHoursServiceImpl {
            extra_hours_dao: extra_hours_dao.clone(),
            permission_service: permission_service.clone(),
            sales_person_service: sales_person_service.clone(),
            custom_extra_hours_service: custom_extra_hours_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let shiftplan_report_service = Arc::new(ShiftplanReportService {
            shiftplan_report_dao: shiftplan_report_dao.clone(),
            transaction_dao: transaction_dao.clone(),
        });
        let carryover_service = Arc::new(service_impl::carryover::CarryoverServiceImpl {
            carryover_dao,
            transaction_dao: transaction_dao.clone(),
        });
        // Phase 28 (VAC-OFFSET-01, D-28-06): Basic-Offset-Service NACH
        // carryover_service und VOR vacation_balance_service konstruiert —
        // Business-Logic konsumiert Basic, kein Forward-Reference, kein Cycle.
        let vacation_entitlement_offset_service = Arc::new(
            service_impl::vacation_entitlement_offset::VacationEntitlementOffsetServiceImpl::<
                VacationEntitlementOffsetServiceDependencies,
            > {
                vacation_entitlement_offset_dao,
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        // Phase 48 (EXP-02/EXP-03, D-48-BASIC): Basic-Tier — konstruiert
        // parallel zu den anderen Basic-Services (kein Domain-Service als Dep).
        let pdf_export_config_service = Arc::new(
            service_impl::pdf_export_config::PdfExportConfigServiceImpl::<
                PdfExportConfigServiceDependencies,
            > {
                pdf_export_config_dao,
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        // Phase 8 (D-04, Pitfall 3): VacationBalanceServiceImpl ist BL-Tier
        // und MUSS NACH absence_service (Z. ~798), working_hours_service
        // (Z. ~788) und carryover_service (oben) konstruiert werden — sonst
        // sind die Variablen nicht im Scope. Konsumiert keine Services, die
        // ihrerseits VacationBalance konsumieren — kein Cycle.
        let vacation_balance_service = Arc::new(
            service_impl::vacation_balance::VacationBalanceServiceImpl::<
                VacationBalanceServiceDependencies,
            > {
                absence_service: absence_service.clone(),
                employee_work_details_service: working_hours_service.clone(),
                carryover_service: carryover_service.clone(),
                sales_person_service: sales_person_service.clone(),
                vacation_entitlement_offset_service: vacation_entitlement_offset_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );
        // D-24-08: ToggleService ist Basic-Tier (nur DAO + Permission + Transaction).
        // Muss VOR ShiftplanEditService (Business-Tier) und VOR ReportingService
        // konstruiert werden, da beide ToggleService als Dependency haben.
        // Phase 25: ReportingService liest den holiday_auto_credit-Stichtag via ToggleService.
        let toggle_dao = Arc::new(ToggleDao::new(pool.clone()));
        let toggle_service = Arc::new(service_impl::toggle::ToggleServiceImpl {
            toggle_dao,
            permission_service: permission_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl {
            extra_hours_service: extra_hours_service.clone(),
            shiftplan_report_service: shiftplan_report_service.clone(),
            employee_work_details_service: working_hours_service.clone(),
            sales_person_service: sales_person_service.clone(),
            carryover_service: carryover_service.clone(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            absence_service: absence_service.clone(),
            transaction_dao: transaction_dao.clone(),
            // Phase 25: holiday derive-on-read deps (special_day_service already
            // constructed at ~line 753; toggle_service constructed just above).
            special_day_service: special_day_service.clone(),
            toggle_service: toggle_service.clone(),
        });

        let booking_information_service = Arc::new(
            service_impl::booking_information::BookingInformationServiceImpl {
                shiftplan_report_service: shiftplan_report_service.clone(),
                slot_service: slot_service.clone(),
                booking_service: booking_service.clone(),
                sales_person_service: sales_person_service.clone(),
                sales_person_unavailable_service: sales_person_unavailable_service.clone(),
                reporting_service: reporting_service.clone(),
                special_day_service: special_day_service.clone(),
                employee_work_details_service: working_hours_service.clone(),
                // VFA-01: absence_service already in scope (built at line ~821, before this point).
                absence_service: absence_service.clone(),
                permission_service: permission_service.clone(),
                clock_service: clock_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );

        // Phase 40 (D-40-01): WeekStatusService (Basic-Tier) muss VOR dem
        // ShiftplanEditService (Business-Logic-Tier) konstruiert werden, da
        // Letzterer es als Dep für das Wochen-Sperre-Gate konsumiert
        // (Service-Tier-Konvention: erst Basic, dann Business-Logic).
        let week_status_service = Arc::new(WeekStatusService {
            week_status_dao: Arc::new(WeekStatusDao::new(pool.clone())),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let shiftplan_edit_service =
            Arc::new(service_impl::shiftplan_edit::ShiftplanEditServiceImpl {
                permission_service: permission_service.clone(),
                slot_service: slot_service.clone(),
                booking_service: booking_service.clone(),
                sales_person_service: sales_person_service.clone(),
                employee_work_details_service: working_hours_service.clone(),
                carryover_service: carryover_service.clone(),
                reporting_service: reporting_service.clone(),
                uuid_service: uuid_service.clone(),
                transaction_dao: transaction_dao.clone(),
                extra_hours_service: extra_hours_service.clone(),
                sales_person_unavailable_service: sales_person_unavailable_service.clone(),
                // NEU für Phase 3 (D-Phase3-06): Reverse-Warning konsumiert AbsenceService.
                absence_service: absence_service.clone(),
                // D-24-08: ToggleService für paid_limit_hard_enforcement-Prüfung.
                toggle_service: toggle_service.clone(),
                // NEU für Phase 40 (D-40-01): Wochen-Sperre-Gate.
                week_status_service: week_status_service.clone(),
            });
        let shiftplan_dao = Arc::new(ShiftplanDao::new(pool.clone()));
        let shiftplan_service = Arc::new(service_impl::shiftplan_catalog::ShiftplanServiceImpl {
            shiftplan_dao: shiftplan_dao.clone(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let shiftplan_view_service = Arc::new(service_impl::shiftplan::ShiftplanViewServiceImpl {
            slot_service: slot_service.clone(),
            booking_service: booking_service.clone(),
            sales_person_service: sales_person_service.clone(),
            special_day_service: special_day_service.clone(),
            shiftplan_service: shiftplan_service.clone(),
            permission_service: permission_service.clone(),
            transaction_dao: transaction_dao.clone(),
            // NEU für Phase 3 (D-Phase3-09): per-sales-person-Pfade konsumieren
            // AbsenceService + SalesPersonUnavailableService.
            absence_service: absence_service.clone(),
            sales_person_unavailable_service: sales_person_unavailable_service.clone(),
            // NEU für Phase 51 (D-51-07): Stichtag-Gate für ShortDay-Slot-Kürzung.
            toggle_service: toggle_service.clone(),
        });

        let block_service = Arc::new(service_impl::block::BlockServiceImpl {
            slot_service: slot_service.clone(),
            booking_service: booking_service.clone(),
            sales_person_service: sales_person_service.clone(),
            clock_service: clock_service.clone(),
            ical_service: Arc::new(service_impl::ical::IcalServiceImpl),
            transaction_dao: transaction_dao.clone(),
            shiftplan_service: shiftplan_view_service.clone(),
            config_service: config_service.clone(),
        });

        let week_message_service = Arc::new(WeekMessageService {
            week_message_dao: Arc::new(WeekMessageDao::new(pool.clone())),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let billing_period_service = Arc::new(BillingPeriodService {
            sales_person_service: sales_person_service.clone(),
            permission_service: permission_service.clone(),
            billing_period_dao: Arc::new(BillingPeriodDao::new(pool.clone())),
            billing_period_sales_person_dao: Arc::new(BillingPeriodSalesPersonDao::new(
                pool.clone(),
            )),
            uuid_service: uuid_service.clone(),
            clock_service: clock_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let text_template_service = Arc::new(TextTemplateService {
            text_template_dao: text_template_dao.clone(),
            permission_service: permission_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let billing_period_report_service = Arc::new(BillingPeriodReportService {
            billing_period_service: billing_period_service.clone(),
            reporting_service: reporting_service.clone(),
            sales_person_service: sales_person_service.clone(),
            employee_work_details_service: working_hours_service.clone(),
            text_template_service: text_template_service.clone(),
            permission_service: permission_service.clone(),
            uuid_service: uuid_service.clone(),
            clock_service: clock_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let block_report_service = Arc::new(BlockReportService {
            block_service: block_service.clone(),
            text_template_service: text_template_service.clone(),
            permission_service: permission_service.clone(),
            clock_service: clock_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        let user_invitation_service = Arc::new(service_impl::user_invitation::UserInvitationServiceImpl {
            user_invitation_dao,
            permission_dao: permission_dao.clone(),
            permission_service: permission_service.clone(),
            session_service: session_service.clone(),
            uuid_service: uuid_service.clone(),
            transaction_dao: transaction_dao.clone(),
        });

        // Phase-2 Plan-04: FeatureFlagService wird oben (vor reporting_service)
        // konstruiert und in den ReportingService eingespeist. Die Plan-03-DI
        // ist damit vollstaendig live (kein #[allow(unused_variables)] mehr).
        // D-24-08: toggle_dao + toggle_service wurden nach oben (vor shiftplan_edit_service) verschoben.

        // Phase 8.5 (Plan 03) — migration_source_dao wird von AbsenceConversionService benötigt.
        let migration_source_dao = Arc::new(MigrationSourceDao::new(pool.clone()));

        // Phase 48 Plan 04 (EXP-01/EXP-03): PdfExportScheduler (BL-Tier).
        // Konstruiert nach shiftplan_service (Basic) + shiftplan_view_service
        // (Read-Aggregat) + sales_person_service (Basic) + pdf_export_config_service
        // (Basic) + permission_service + clock_service + transaction_dao —
        // alle bereits im Scope. Die WebDavUpload-Factory ist die Produktions-
        // Variante, die pro Lauf einen echten WebDavClient baut.
        // Phase 49 (PDF-03/PDF-04/PDF-05): BL-Tier PdfShiftplanService.
        // Konsumiert bereits konstruierte Basic-/Read-Aggregat-Services:
        // shiftplan_view_service, sales_person_service, week_status_service.
        // Muss VOR pdf_export_scheduler stehen, weil der Scheduler ihn seit
        // Wave 1 als Dep konsumiert (Phase 49 Plan 03 landed).
        let pdf_shiftplan_service = Arc::new(
            service_impl::pdf_shiftplan::PdfShiftplanServiceImpl::<
                PdfShiftplanServiceDependencies,
            >::new(
                shiftplan_view_service.clone(),
                sales_person_service.clone(),
                week_status_service.clone(),
                permission_service.clone(),
                transaction_dao.clone(),
            ),
        );

        let pdf_export_scheduler = Arc::new(PdfExportSchedulerService::new(
            pdf_export_config_service.clone(),
            pdf_shiftplan_service.clone(),
            shiftplan_service.clone(),
            permission_service.clone(),
            clock_service.clone(),
            transaction_dao.clone(),
            Arc::new(ProductionWebDavUploadFactory) as Arc<dyn WebDavUploadFactory>,
        ));

        // Phase 8.5 (Plan 03) — AbsenceConversionService (BL-Tier nach Basic-Services).
        // Konsumiert: extra_hours_dao (Basic-DAO), absence_dao (Basic-DAO),
        // migration_source_dao (Basic-DAO), extra_hours_service (Basic),
        // permission_service (Basic), transaction_dao.
        // Kein OnceLock / Forward-Decl — alle Deps sind bereits konstruiert.
        let absence_conversion_service = Arc::new(
            service_impl::absence_conversion::AbsenceConversionServiceImpl::<
                AbsenceConversionServiceDependencies,
            > {
                extra_hours_dao: extra_hours_dao.clone(),
                absence_dao: absence_dao.clone(),
                migration_source_dao: migration_source_dao.clone(),
                extra_hours_service: extra_hours_service.clone(),
                permission_service: permission_service.clone(),
                transaction_dao: transaction_dao.clone(),
            },
        );

        Self {
            user_service,
            session_service,
            permission_service,
            slot_service,
            sales_person_service,
            special_day_service,
            sales_person_unavailable_service,
            booking_service,
            custom_extra_hours_service,
            booking_information_service,
            booking_log_service,
            reporting_service,
            working_hours_service,
            absence_service,
            vacation_balance_service,
            extra_hours_service,
            shiftplan_edit_service,
            block_service,
            shiftplan_service,
            shiftplan_view_service,
            week_message_service,
            week_status_service,
            billing_period_service,
            billing_period_report_service,
            block_report_service,
            text_template_service,
            user_invitation_service,
            toggle_service,
            sales_person_shiftplan_service,
            feature_flag_service,
            absence_conversion_service,
            vacation_entitlement_offset_service,
            pdf_export_config_service,
            pdf_export_scheduler,
            pdf_shiftplan_service,
            basic_dao: Arc::new(BasicDaoImpl::new(pool)),
        }
    }
}

async fn create_admin_user(pool: Arc<SqlitePool>, username: &str) {
    use dao::PermissionDao;
    // On development create the DEVUSER and give it admin permissions.
    let permission_dao = PermissionDaoImpl::new(pool.clone());

    let users = permission_dao.all_users().await.expect("Expected users");
    let contains_admin_user = users.iter().any(|user| user.name.as_ref() == username);
    if !contains_admin_user {
        permission_dao
            .create_user(
                &dao::UserEntity {
                    name: username.into(),
                },
                "dev-first-start",
            )
            .await
            .unwrap_or_else(|_| panic!("Expected being able to create the {}", username));
        permission_dao
            .add_user_role(username, "admin", "dev-first-start")
            .await
            .unwrap_or_else(|_| panic!(
                "Expected being able to make {} an admin",
                username
            ));
    }
}

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");

    #[cfg(feature = "local_logging")]
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .with_file(true)
        .finish();

    #[cfg(feature = "json_logging")]
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .json()
        .with_span_events(FmtSpan::CLOSE)
        .with_span_list(true)
        .with_file(true)
        .finish();

    #[cfg(not(any(feature = "local_logging", feature = "json_logging")))]
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::info!("Shifty backend version: {}", version);
    dotenvy::dotenv().ok();
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./localdb.sqlite3")
            .await
            .expect("Could not connect to database"),
    );

    // Apply SQLite-specific migrations
    sqlx::migrate!("../migrations/sqlite")
        .run(pool.as_ref())
        .await
        .expect("Failed to run migrations");

    let rest_state = RestStateImpl::new(pool.clone());
    create_admin_user(pool.clone(), "DEVUSER").await;
    create_admin_user(pool.clone(), "admin").await;

    let scheduler_service = SchedulerServiceImpl::new(rest_state.shiftplan_edit_service.clone());
    scheduler_service
        .start()
        .await
        .expect("Expected the scheduler to start");

    // Phase 48 Plan 04: PDF-Export-Scheduler starten. Bei enabled=false in
    // der DB registriert start() den Cron-Job dormant; erst ein PUT auf
    // `/pdf-export-config` mit enabled=true triggert einen Reload und macht
    // den Job aktiv.
    rest_state
        .pdf_export_scheduler
        .start()
        .await
        .expect("Expected the pdf-export scheduler to start");

    rest::start_server(rest_state).await
}
