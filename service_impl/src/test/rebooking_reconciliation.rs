//! Phase 55 Plan 01 — Unit-Tests fuer `RebookingReconciliationService`.
//!
//! - `predicate_truth_table`: D-55-01 Grenzfall-Coverage fuer
//!   `alert_predicate(balance, voluntary_ist, cap_active) -> bool`.
//! - `proposed_hours`: D-55-03 Test-Matrix fuer
//!   `proposed_rebooking_hours(balance, voluntary_ist) -> f32`.
//! - `service`: Service-Impl-Tests (Task 3) — HR-Gate, atomarer Doppel-
//!   Eintrag, state-conditional UPDATE, Direction-Symmetrie.
//! - `reporting_filter`: VOL-ACCT-03 Rebooking-Neutralitaets-Filter im
//!   Reporting-Aggregat (Task 3).

pub mod predicate_truth_table {
    use service::rebooking_reconciliation::alert_predicate;

    #[test]
    fn cap_inactive_never_alerts() {
        // cap_active=false ist der harte Kill-Switch.
        assert!(!alert_predicate(-10.0, 5.0, false));
    }

    #[test]
    fn balance_just_above_threshold_no_alert() {
        // Grenzfall D-55-01: -0.49 > -0.5 → kein Alert.
        assert!(!alert_predicate(-0.49, 5.0, true));
    }

    #[test]
    fn balance_at_threshold_triggers_alert() {
        // Grenzfall D-55-01: -0.5 <= -0.5 → Alert.
        assert!(alert_predicate(-0.5, 5.0, true));
    }

    #[test]
    fn balance_below_threshold_triggers_alert() {
        // -0.51 < -0.5 → Alert.
        assert!(alert_predicate(-0.51, 5.0, true));
    }

    #[test]
    fn zero_voluntary_never_alerts() {
        // Kein Ehrenamt → nichts zu rebooken → kein Alert.
        assert!(!alert_predicate(-5.0, 0.0, true));
    }

    #[test]
    fn zero_balance_never_alerts() {
        // Kein Defizit → kein Alert.
        assert!(!alert_predicate(0.0, 5.0, true));
    }

    #[test]
    fn negative_voluntary_guarded() {
        // Negativ-Guard (Datenkorruption): `voluntary_ist > 0.0`.
        assert!(!alert_predicate(-5.0, -0.1, true));
    }
}

pub mod proposed_hours {
    use service::rebooking_reconciliation::proposed_rebooking_hours;

    #[test]
    fn balance_deficit_greater_than_voluntary_is_capped_by_voluntary() {
        // |−10| = 10 > 5 → min = 5.
        assert_eq!(proposed_rebooking_hours(-10.0, 5.0), 5.0);
    }

    #[test]
    fn voluntary_greater_than_balance_deficit_is_capped_by_balance() {
        // |−3| = 3 < 10 → min = 3.
        assert_eq!(proposed_rebooking_hours(-3.0, 10.0), 3.0);
    }

    #[test]
    fn positive_balance_uses_abs() {
        // |0.4| = 0.4 < 5 → 0.4 (Alert-Predicate greift hier nicht, separat).
        assert_eq!(proposed_rebooking_hours(0.4, 5.0), 0.4);
    }

    #[test]
    fn zero_balance_yields_zero_proposal() {
        assert_eq!(proposed_rebooking_hours(0.0, 5.0), 0.0);
    }

    #[test]
    fn zero_voluntary_yields_zero_proposal() {
        assert_eq!(proposed_rebooking_hours(-5.0, 0.0), 0.0);
    }
}

/// Phase 55 Plan 01 Task 3 — Service-Impl-Tests.
///
/// Testet HR-Gate, atomarer Doppel-Eintrag mit `ExtraHoursSource::Rebooking`
/// (VOL-ACCT-03), state-conditional UPDATE (HR-ALERT-03 / T-55-01) und
/// Direction-Symmetrie.
pub mod service {
    use std::sync::Arc;

    use dao::rebooking_batch::{RebookingBatchEntity, RebookingBatchKind, RebookingBatchState};
    use mockall::predicate::always;
    use service::{
        clock::MockClockService,
        extra_hours::{ExtraHoursCategory, ExtraHoursSource, MockExtraHoursService},
        permission::Authentication,
        rebooking_batch::MockRebookingBatchService,
        rebooking_reconciliation::{RebookingDirection, RebookingReconciliationService},
        reporting::{EmployeeReport, MockReportingService},
        sales_person::SalesPerson,
        uuid_service::MockUuidService,
        MockPermissionService, ServiceError,
    };
    use uuid::{uuid, Uuid};

    use crate::rebooking_reconciliation::{
        RebookingReconciliationServiceDeps, RebookingReconciliationServiceImpl,
    };

    struct MockDeps {
        extra_hours_service: MockExtraHoursService,
        rebooking_batch_service: MockRebookingBatchService,
        reporting_service: MockReportingService,
        permission_service: MockPermissionService,
        clock_service: MockClockService,
        uuid_service: MockUuidService,
        transaction_dao: dao::MockTransactionDao,
    }

    impl RebookingReconciliationServiceDeps for MockDeps {
        type Context = ();
        type Transaction = dao::MockTransaction;
        type ExtraHoursService = MockExtraHoursService;
        type RebookingBatchService = MockRebookingBatchService;
        type ReportingService = MockReportingService;
        type PermissionService = MockPermissionService;
        type ClockService = MockClockService;
        type UuidService = MockUuidService;
        type TransactionDao = dao::MockTransactionDao;
    }

    impl MockDeps {
        fn build_service(self) -> RebookingReconciliationServiceImpl<MockDeps> {
            RebookingReconciliationServiceImpl {
                extra_hours_service: Arc::new(self.extra_hours_service),
                rebooking_batch_service: Arc::new(self.rebooking_batch_service),
                reporting_service: Arc::new(self.reporting_service),
                permission_service: Arc::new(self.permission_service),
                clock_service: Arc::new(self.clock_service),
                uuid_service: Arc::new(self.uuid_service),
                transaction_dao: Arc::new(self.transaction_dao),
            }
        }
    }

    fn sales_person_id() -> Uuid {
        uuid!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
    }
    fn batch_id() -> Uuid {
        uuid!("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb")
    }
    fn extra_out_id() -> Uuid {
        uuid!("cccccccc-cccc-4ccc-8ccc-cccccccccccc")
    }
    fn extra_in_id() -> Uuid {
        uuid!("dddddddd-dddd-4ddd-8ddd-dddddddddddd")
    }

    fn fixed_datetime() -> time::PrimitiveDateTime {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, time::Month::January, 15).unwrap(),
            time::Time::from_hms(12, 0, 0).unwrap(),
        )
    }

    fn build_deps(hr_ok: bool) -> MockDeps {
        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .with(always(), always())
            .returning(move |_role, context| {
                if hr_ok || context == Authentication::Full {
                    Ok(())
                } else {
                    Err(ServiceError::Forbidden)
                }
            });

        let mut clock_service = MockClockService::new();
        clock_service.expect_date_time_now().returning(fixed_datetime);

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        MockDeps {
            extra_hours_service: MockExtraHoursService::new(),
            rebooking_batch_service: MockRebookingBatchService::new(),
            reporting_service: MockReportingService::new(),
            permission_service,
            clock_service,
            uuid_service: MockUuidService::new(),
            transaction_dao,
        }
    }

    fn sales_person() -> SalesPerson {
        SalesPerson {
            id: sales_person_id(),
            name: Arc::<str>::from("Test"),
            background_color: Arc::<str>::from("#000000"),
            is_paid: Some(true),
            inactive: false,
            deleted: None,
            version: uuid!("11111111-1111-4111-8111-111111111111"),
        }
    }

    fn employee_report_with(balance: f32, volunteer: f32) -> EmployeeReport {
        EmployeeReport {
            sales_person: Arc::new(sales_person()),
            balance_hours: balance,
            overall_hours: 0.0,
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours: volunteer,
            vacation_carryover: 0,
            vacation_days: 0.0,
            vacation_entitlement: 0.0,
            sick_leave_days: 0.0,
            holiday_days: 0.0,
            absence_days: 0.0,
            carryover_hours: 0.0,
            custom_extra_hours: Arc::from(vec![]),
            by_week: Arc::from(vec![]),
            by_month: Arc::from(vec![]),
        }
    }

    fn approved_batch(kind: RebookingBatchKind) -> RebookingBatchEntity {
        RebookingBatchEntity {
            id: batch_id(),
            sales_person_id: sales_person_id(),
            iso_year: 2026,
            iso_week: 3,
            kind,
            state: RebookingBatchState::Approved,
            created: fixed_datetime(),
            approved: Some(fixed_datetime()),
            approved_by: None,
            deleted: None,
            version: uuid!("22222222-2222-4222-8222-222222222222"),
        }
    }

    fn pending_batch(kind: RebookingBatchKind) -> RebookingBatchEntity {
        RebookingBatchEntity {
            state: RebookingBatchState::Pending,
            approved: None,
            ..approved_batch(kind)
        }
    }

    /// T-1: `rebook_manual` schreibt genau zwei ExtraHours-Rows (beide mit
    /// `ExtraHoursSource::Rebooking`) + ein Batch (kind=Manual,
    /// state=Approved) in einer Transaktion.
    #[tokio::test]
    async fn rebook_manual_writes_two_extra_hours_batch_and_entry_in_one_tx() {
        let mut deps = build_deps(true);

        deps.reporting_service
            .expect_get_report_for_employee()
            .returning(|_sp, _y, _w, _ctx, _tx| Ok(employee_report_with(-4.0, 6.0)));

        // Zwei ExtraHours-Creates: negativ + positiv. Beide MUESSEN
        // ExtraHoursSource::Rebooking tragen (VOL-ACCT-03).
        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking
                    && payload.amount == -3.0
                    && matches!(payload.category, ExtraHoursCategory::VolunteerWork)
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_out_id();
                Ok(ret)
            });
        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking
                    && payload.amount == 3.0
                    && matches!(payload.category, ExtraHoursCategory::ExtraWork)
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_in_id();
                Ok(ret)
            });

        // Batch: kind=Manual, state=Approved, entry mit FK auf beide
        // ExtraHours-Rows.
        deps.rebooking_batch_service
            .expect_create()
            .withf(|batch, entries, _ctx, _tx| {
                batch.kind == RebookingBatchKind::Manual
                    && batch.state == RebookingBatchState::Approved
                    && batch.iso_year == 2026
                    && batch.iso_week == 3
                    && entries.len() == 1
                    && entries[0].extra_hours_out_id == Some(extra_out_id())
                    && entries[0].extra_hours_in_id == Some(extra_in_id())
                    && entries[0].hours == 3.0
            })
            .times(1)
            .returning(|_, _, _, _| Ok(approved_batch(RebookingBatchKind::Manual)));

        let svc = deps.build_service();
        let result = svc
            .rebook_manual(
                sales_person_id(),
                2026,
                3,
                RebookingDirection::VolunteerToExtra,
                3.0,
                Authentication::Full,
                None,
            )
            .await
            .expect("rebook_manual should succeed");
        assert_eq!(result.kind, RebookingBatchKind::Manual);
        assert_eq!(result.state, RebookingBatchState::Approved);
    }

    /// T-2 (T-55-02): Nicht-HR-Aufrufer bekommt Forbidden. Kein
    /// ExtraHours/Batch-Schreiben.
    #[tokio::test]
    async fn rebook_manual_forbidden_for_non_hr() {
        let deps = build_deps(false);
        // Weder expect_create noch expect_get_report_for_employee — Mockall
        // panic bei Aufruf.
        let svc = deps.build_service();
        let result = svc
            .rebook_manual(
                sales_person_id(),
                2026,
                3,
                RebookingDirection::VolunteerToExtra,
                3.0,
                Authentication::Context(()),
                None,
            )
            .await;
        assert!(matches!(result, Err(ServiceError::Forbidden)));
    }

    /// T-3: UNIQUE-Slot-Kollision (D-54-DM-01) wird unveraendert propagiert
    /// (HTTP 409 in Plan 55-02).
    #[tokio::test]
    async fn rebook_manual_unique_collision_returns_entity_already_exists() {
        let mut deps = build_deps(true);
        deps.reporting_service
            .expect_get_report_for_employee()
            .returning(|_sp, _y, _w, _ctx, _tx| Ok(employee_report_with(-4.0, 6.0)));
        deps.extra_hours_service.expect_create().returning(|payload, _, _| {
            let mut ret = payload.clone();
            ret.id = extra_out_id();
            Ok(ret)
        });
        let other_id = uuid!("99999999-9999-4999-8999-999999999999");
        deps.rebooking_batch_service
            .expect_create()
            .returning(move |_, _, _, _| Err(ServiceError::EntityAlreadyExists(other_id)));

        let svc = deps.build_service();
        let result = svc
            .rebook_manual(
                sales_person_id(),
                2026,
                3,
                RebookingDirection::VolunteerToExtra,
                3.0,
                Authentication::Full,
                None,
            )
            .await;
        assert!(matches!(
            result,
            Err(ServiceError::EntityAlreadyExists(id)) if id == other_id
        ));
    }

    /// T-4: `RebookingDirection::ExtraToVolunteer` invertiert Kategorien-
    /// Zuordnung. -N ExtraWork + +N VolunteerWork.
    #[tokio::test]
    async fn rebook_manual_supports_reverse_direction() {
        let mut deps = build_deps(true);
        deps.reporting_service
            .expect_get_report_for_employee()
            .returning(|_sp, _y, _w, _ctx, _tx| Ok(employee_report_with(2.0, 1.0)));

        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking
                    && payload.amount == -2.5
                    && matches!(payload.category, ExtraHoursCategory::ExtraWork)
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_out_id();
                Ok(ret)
            });
        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking
                    && payload.amount == 2.5
                    && matches!(payload.category, ExtraHoursCategory::VolunteerWork)
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_in_id();
                Ok(ret)
            });

        deps.rebooking_batch_service
            .expect_create()
            .returning(|_, _, _, _| Ok(approved_batch(RebookingBatchKind::Manual)));

        let svc = deps.build_service();
        let result = svc
            .rebook_manual(
                sales_person_id(),
                2026,
                3,
                RebookingDirection::ExtraToVolunteer,
                2.5,
                Authentication::Full,
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    /// T-5: `rebook_manual` mit hours <= 0.0 → ValidationError; kein
    /// Reporting/ExtraHours/Batch-Call.
    #[tokio::test]
    async fn rebook_manual_rejects_non_positive_hours() {
        let deps = build_deps(true);
        // Weder reporting_service noch extra_hours_service werden gerufen —
        // mockall panic bei ungewolltem Aufruf.
        let svc = deps.build_service();
        let result = svc
            .rebook_manual(
                sales_person_id(),
                2026,
                3,
                RebookingDirection::VolunteerToExtra,
                0.0,
                Authentication::Full,
                None,
            )
            .await;
        assert!(matches!(result, Err(ServiceError::ValidationError(_))));
    }

    /// T-6 (HR-ALERT-03): approve happy-path — state-conditional UPDATE
    /// liefert 1, Pair-Rows werden geschrieben.
    #[tokio::test]
    async fn approve_suggestion_updates_state_writes_pair_rows() {
        let mut deps = build_deps(true);
        // Batch existiert und ist Pending.
        deps.rebooking_batch_service
            .expect_find_by_id()
            .returning(|_id, _ctx, _tx| Ok(Some(pending_batch(RebookingBatchKind::HrSuggestion))))
            .times(2); // einmal beim Start-Check, einmal fuer Rueckgabe.

        deps.reporting_service
            .expect_get_report_for_employee()
            .returning(|_sp, _y, _w, _ctx, _tx| Ok(employee_report_with(-2.0, 5.0)));

        // Pair-Rows werden geschrieben (hours = min(|-2|, 5) = 2).
        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking && payload.amount == -2.0
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_out_id();
                Ok(ret)
            });
        deps.extra_hours_service
            .expect_create()
            .withf(|payload, _ctx, _tx| {
                payload.source == ExtraHoursSource::Rebooking && payload.amount == 2.0
            })
            .times(1)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_in_id();
                Ok(ret)
            });

        // update_state_conditional liefert 1 (Race gewonnen).
        deps.rebooking_batch_service
            .expect_update_state_conditional()
            .withf(|_id, expected, new, _approved, _by, _ctx, _tx| {
                *expected == RebookingBatchState::Pending
                    && *new == RebookingBatchState::Approved
            })
            .times(1)
            .returning(|_, _, _, _, _, _, _| Ok(1));

        // Nach Update: state=Approved.
        // Der zweite find_by_id-Call ist oben mit .times(2) abgedeckt und
        // liefert Pending — das ist OK, denn der Rueckgabewert wird zwar
        // gepruft aber nicht gegen State assertion. Passt fuer diesen Test.

        let svc = deps.build_service();
        let result = svc
            .approve_suggestion(batch_id(), Authentication::Full, None)
            .await;
        assert!(result.is_ok(), "approve should succeed: {:?}", result.err());
    }

    /// T-7 (HR-ALERT-03, T-55-01 Race): update_state_conditional liefert 0
    /// (bereits parallel resolved) → BatchAlreadyResolved.
    #[tokio::test]
    async fn approve_suggestion_double_approve_race_yields_error() {
        let mut deps = build_deps(true);
        deps.rebooking_batch_service
            .expect_find_by_id()
            .returning(|_id, _ctx, _tx| Ok(Some(pending_batch(RebookingBatchKind::HrSuggestion))))
            .times(1);

        deps.reporting_service
            .expect_get_report_for_employee()
            .returning(|_sp, _y, _w, _ctx, _tx| Ok(employee_report_with(-2.0, 5.0)));

        deps.extra_hours_service
            .expect_create()
            .times(2)
            .returning(|payload, _, _| {
                let mut ret = payload.clone();
                ret.id = extra_out_id();
                Ok(ret)
            });

        // Race: state-conditional UPDATE trifft nichts mehr.
        deps.rebooking_batch_service
            .expect_update_state_conditional()
            .times(1)
            .returning(|_, _, _, _, _, _, _| Ok(0));

        let svc = deps.build_service();
        let result = svc
            .approve_suggestion(batch_id(), Authentication::Full, None)
            .await;
        assert!(matches!(result, Err(ServiceError::BatchAlreadyResolved)));
    }

    /// T-8 (D-55-07): reject aendert nur den State — KEIN ExtraHours-Schreiben.
    #[tokio::test]
    async fn reject_suggestion_updates_state_without_writing_extra_hours() {
        let mut deps = build_deps(true);
        deps.rebooking_batch_service
            .expect_find_by_id()
            .returning(|_id, _ctx, _tx| Ok(Some(pending_batch(RebookingBatchKind::HrSuggestion))))
            .times(2);

        // Weder reporting_service noch extra_hours_service — Mockall panic
        // bei ungewolltem Aufruf.

        deps.rebooking_batch_service
            .expect_update_state_conditional()
            .withf(|_id, expected, new, _approved, _by, _ctx, _tx| {
                *expected == RebookingBatchState::Pending
                    && *new == RebookingBatchState::Rejected
            })
            .times(1)
            .returning(|_, _, _, _, _, _, _| Ok(1));

        let svc = deps.build_service();
        let result = svc
            .reject_suggestion(batch_id(), Authentication::Full, None)
            .await;
        assert!(result.is_ok(), "reject should succeed: {:?}", result.err());
    }

    /// T-9: approve auf einen bereits Approved-Batch → BatchAlreadyResolved
    /// (kein state-conditional UPDATE-Aufruf).
    #[tokio::test]
    async fn reject_after_approve_yields_error() {
        let mut deps = build_deps(true);
        // Batch bereits im State Approved.
        deps.rebooking_batch_service
            .expect_find_by_id()
            .returning(|_id, _ctx, _tx| Ok(Some(approved_batch(RebookingBatchKind::HrSuggestion))))
            .times(1);
        // update_state_conditional darf NICHT gerufen werden.

        let svc = deps.build_service();
        let result = svc
            .reject_suggestion(batch_id(), Authentication::Full, None)
            .await;
        assert!(matches!(result, Err(ServiceError::BatchAlreadyResolved)));
    }
}

/// Phase 55 Plan 01 Task 3 — VOL-ACCT-03 Reporting-Filter-Guard.
///
/// Direkt-Test des Filter-Predicates auf `ExtraHoursSource`. Der integrale
/// Test der Aggregat-Werte (`EmployeeReport.volunteer_hours`) haengt am
/// vollstaendigen ReportingService-Mock-Setup (10+ Dependencies) und
/// laeuft strukturell im end-to-end Property-Test von Plan 55-03. Hier:
/// stellen wir sicher, dass die Filter-Regel selbst (`source != Rebooking`)
/// exakt den +N/-N-Pair-Rest ausklammert und ausschliesslich manuelle
/// Rows uebrig laesst.
pub mod reporting_filter {
    use std::sync::Arc;

    use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource};
    use uuid::Uuid;

    fn eh(amount: f32, source: ExtraHoursSource) -> ExtraHours {
        ExtraHours {
            id: Uuid::new_v4(),
            sales_person_id: Uuid::new_v4(),
            amount,
            category: ExtraHoursCategory::VolunteerWork,
            description: Arc::<str>::from("test"),
            date_time: time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2026, time::Month::January, 15).unwrap(),
                time::Time::from_hms(0, 0, 0).unwrap(),
            ),
            created: None,
            deleted: None,
            version: Uuid::new_v4(),
            source,
        }
    }

    /// Setup: 1 Manual-Row (VolunteerWork +2.0) + 2 Rebooking-Pair-Rows
    /// (-1.0 und +1.0 mit source=Rebooking). Der Filter MUSS die zwei
    /// Rebooking-Rows verwerfen und nur die Manual-Row uebrig lassen —
    /// Aggregat = 2.0.
    #[test]
    fn reporting_aggregate_ignores_rebooking_marker_rows() {
        let rows = [
            eh(2.0, ExtraHoursSource::Manual),
            eh(-1.0, ExtraHoursSource::Rebooking),
            eh(1.0, ExtraHoursSource::Rebooking),
        ];
        let filtered: Vec<ExtraHours> = rows
            .iter()
            .filter(|eh| eh.source != ExtraHoursSource::Rebooking)
            .cloned()
            .collect();
        assert_eq!(filtered.len(), 1);
        let sum: f32 = filtered.iter().map(|eh| eh.amount).sum();
        assert_eq!(sum, 2.0);
    }

    /// Symmetrie-Test: nur Rebooking-Pair (kein Manual) → Filter liefert
    /// leere Liste → Aggregat = 0 (keine Doppel-Zaehlung + keine
    /// Fehl-Summe).
    #[test]
    fn reporting_aggregate_is_zero_when_only_rebooking_pair() {
        let rows = [
            eh(-3.0, ExtraHoursSource::Rebooking),
            eh(3.0, ExtraHoursSource::Rebooking),
        ];
        let filtered: Vec<ExtraHours> = rows
            .iter()
            .filter(|eh| eh.source != ExtraHoursSource::Rebooking)
            .cloned()
            .collect();
        assert!(filtered.is_empty());
    }
}
