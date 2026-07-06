use std::sync::Arc;

use async_trait::async_trait;
use dao::special_day::SpecialDayEntity;
use service::{
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    special_days::{SpecialDay, SpecialDayType},
    ServiceError, ValidationFailureItem,
};
use uuid::Uuid;

pub struct SpecialDayServiceImpl<
    SpecialDayDao: dao::special_day::SpecialDayDao,
    PermissionService: service::PermissionService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    special_day_dao: Arc<SpecialDayDao>,
    permission_service: Arc<PermissionService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<SpecialDayDao, PermissionService, ClockService, UuidService>
    SpecialDayServiceImpl<SpecialDayDao, PermissionService, ClockService, UuidService>
where
    SpecialDayDao: dao::special_day::SpecialDayDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        special_day_dao: Arc<SpecialDayDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            special_day_dao,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        SpecialDayDao: dao::special_day::SpecialDayDao + Sync + Send,
        PermissionService: service::PermissionService + Sync + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::special_days::SpecialDayService
    for SpecialDayServiceImpl<SpecialDayDao, PermissionService, ClockService, UuidService>
{
    type Context = PermissionService::Context;

    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError> {
        Ok(self
            .special_day_dao
            .find_by_week(year, calendar_week)
            .await?
            .iter()
            .map(SpecialDay::from)
            .collect())
    }
    /// Liefert alle Special Days, deren tatsächliches **Kalender-Datum** ins
    /// Kalenderjahr `year` fällt. Die DB speichert `year` als ISO-Wochenjahr —
    /// ein Eintrag am 01.01.2027 landet z.B. als `(year=2026, week=53, day=Fri)`.
    /// Daher werden hier `year` und `year - 1` geladen und dann per
    /// `ShiftyDate::to_date().year()` gefiltert (SDF-03 post-ship).
    async fn get_by_year(
        &self,
        year: u32,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError> {
        use shifty_utils::ShiftyDate;

        let target_year = year as i32;
        let mut results: Vec<SpecialDay> = Vec::new();

        let mut push_matches = |entities: Arc<[SpecialDayEntity]>| {
            for entity in entities.iter() {
                let sd = SpecialDay::from(entity);
                let Ok(shifty_date) =
                    ShiftyDate::new(sd.year, sd.calendar_week, sd.day_of_week)
                else {
                    continue;
                };
                if shifty_date.to_date().year() == target_year {
                    results.push(sd);
                }
            }
        };

        push_matches(self.special_day_dao.find_by_year(year).await?);
        if year > 0 {
            push_matches(self.special_day_dao.find_by_year(year - 1).await?);
        }

        // v2.2 post-ship: nach Kalenderdatum aufsteigend sortieren, damit ein
        // 01.01.YYYY-Eintrag (der aus ISO-Wochenjahr YYYY-1 stammt) am Anfang der
        // Liste steht und nicht ans Ende der year=YYYY-Einträge angehängt bleibt.
        results.sort_by_key(|sd| {
            ShiftyDate::new(sd.year, sd.calendar_week, sd.day_of_week)
                .map(|d| d.to_date())
                .unwrap_or_else(|_| time::Date::MIN)
        });

        Ok(results.into())
    }
    /// Phase 52 Follow-up #3 — direkter ISO-Wochenjahr-Batch für Konsumenten,
    /// die per ISO-Wochenjahr bucketen (z.B. `get_weekly_summary`).
    ///
    /// Delegiert direkt an `SpecialDayDao::find_by_iso_year`, weil die DB-Spalte
    /// `year` bereits das ISO-Wochenjahr speichert. Anders als `get_by_year`
    /// (das für die REST-`/special-days/year/{y}`-Route Kalender-Jahr-basiert
    /// filtert) macht diese Variante KEIN Union(y, y-1) und KEINEN
    /// Kalender-Datum-Post-Filter — d.h. eine Row `(year=2026, week=53,
    /// day=Fri)` (Kalender-Datum 2027-01-01) landet korrekt im
    /// `iso_year=2026`-Bucket.
    async fn get_by_iso_year(
        &self,
        year: u32,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError> {
        Ok(self
            .special_day_dao
            .find_by_iso_year(year)
            .await?
            .iter()
            .map(SpecialDay::from)
            .collect())
    }
    async fn create(
        &self,
        special_day: &SpecialDay,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        // Server-side input validation (D-33-06 / D-33-07). The backend is the
        // real trust boundary for this shiftplanner-gated mutation; the type/time
        // coupling and calendar_week bounds are otherwise only enforced in the
        // two front-ends (WR-03).
        let mut validation: Vec<ValidationFailureItem> = Vec::new();
        match special_day.day_type {
            // A ShortDay requires a time_of_day.
            SpecialDayType::ShortDay if special_day.time_of_day.is_none() => {
                validation.push(ValidationFailureItem::InvalidValue(
                    "time_of_day is required for a ShortDay".into(),
                ));
            }
            _ => {}
        }
        let max_week = time::util::weeks_in_year(special_day.year as i32);
        if special_day.calendar_week < 1 || special_day.calendar_week > max_week {
            validation.push(ValidationFailureItem::InvalidValue(
                format!(
                    "calendar_week {} out of range 1..={} for year {}",
                    special_day.calendar_week, max_week, special_day.year
                )
                .into(),
            ));
        }
        if !validation.is_empty() {
            return Err(ServiceError::ValidationError(validation.into()));
        }

        let mut special_day = special_day.clone();
        // A Holiday never carries a time_of_day — normalize so the persisted row
        // matches the type/time invariant regardless of what the client sent.
        if special_day.day_type == SpecialDayType::Holiday {
            special_day.time_of_day = None;
        }

        // Guard: client must not supply id/version on create.
        if !special_day.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !special_day.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        // Same-date replacement (SDF-01, D-01): if an active (deleted IS NULL) row
        // already exists for this (year, calendar_week, day_of_week), replace it in
        // place via a single atomic UPDATE instead of returning a Duplicate error.
        // This makes switching Holiday ↔ ShortDay work without a frontend delete-then-
        // create dance and without a new PUT endpoint (D-02, D-04).
        // `find_by_week` already filters WHERE deleted IS NULL, so no second check here.
        let existing_week = self
            .special_day_dao
            .find_by_week(special_day.year, special_day.calendar_week)
            .await?;
        if let Some(existing_entry) = existing_week
            .iter()
            .find(|e| e.day_of_week == special_day.day_of_week)
        {
            // Preserve the existing row's id and created timestamp; update only
            // day_type, time_of_day, and version so the replacement is atomic (D-01).
            let mut updated = existing_entry.clone();
            updated.day_type = (&special_day.day_type).into();
            updated.time_of_day = special_day.time_of_day;
            updated.version = self
                .uuid_service
                .new_uuid("special-day-service::replace version");
            self.special_day_dao
                .update(&updated, "special-days-service::replace")
                .await?;
            return Ok(SpecialDay::from(&updated));
        }

        // Create path: stamp `created` from the clock only when actually inserting
        // a new row — the replace path above keeps the existing row's `created`.
        special_day.created = Some(self.clock_service.date_time_now());
        let mut entity: SpecialDayEntity = (&special_day).try_into()?;
        entity.id = self.uuid_service.new_uuid("special-day-service::create id");
        entity.version = self
            .uuid_service
            .new_uuid("special-day-service::create version");

        self.special_day_dao
            .create(&entity, "special-days-service::create")
            .await?;
        Ok(SpecialDay::from(&entity))
    }
    async fn delete(
        &self,
        special_day_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let mut entity = self
            .special_day_dao
            .find_by_id(special_day_id)
            .await?
            .ok_or_else(|| ServiceError::EntityNotFound(special_day_id))?;

        if entity.deleted.is_some() {
            return Err(ServiceError::EntityNotFound(special_day_id));
        }

        entity.deleted = Some(self.clock_service.date_time_now());
        entity.version = self.uuid_service.new_uuid("special-day-service::delete version");

        self.special_day_dao
            .update(&entity, "special-days-service::delete")
            .await?;

        Ok(SpecialDay::from(&entity))
    }
}
