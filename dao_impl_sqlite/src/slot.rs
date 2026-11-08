use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use dao::{slot::SlotEntity, DaoError};
use shifty_utils::DayOfWeek;
use sqlx::{query, SqlitePool};
use time::{
    format_description::well_known::Iso8601, macros::format_description, Date, PrimitiveDateTime,
    Time,
};
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

pub struct SlotDaoImpl {
    _pool: Arc<SqlitePool>,
}
/// Parst den `deleted`-Zeitstempel eines Slots.
///
/// **Der Defekt.** `create_slot`/`update_slot` haben den Wert mit
/// `PrimitiveDateTime::to_string()` geschrieben. Das ist `time`s
/// `Display`-Format und **kein ISO8601**: Leerzeichen statt `T`, Stunde ohne
/// führende Null, variabel viele Nachkommastellen — `2026-11-08 7:17:18.220225594`.
/// Gelesen wird mit einem ISO8601-Parser, der dieses Format nicht kennt: er
/// verbraucht den Datumsteil und meldet am Rest `UnexpectedTrailingCharacters`.
/// Jeder Lesezugriff auf einen soft-gelöschten Slot endete deshalb in
/// `DaoError` → `ServiceError::DatabaseQueryError` → HTTP 500.
///
/// Im Feld trat das beim zweiten „Slot ab KW ändern" auf: der erste Umbau
/// soft-löscht den alten Slot (Erste-KW-Randfall in
/// `ShiftplanEditService::modify_slot`), und das Frontend schickt danach die
/// alte `id` weiter — `edit_slot` gibt den neuen Slot zwar zurück, aber
/// `api::update_slot` verwirft die Antwort. Verdeckt wurde der Fehler dadurch,
/// dass drei der vier Lesepfade `WHERE deleted IS NULL` haben: dort ist die
/// Spalte immer `NULL` und `map` läuft nie. Nur `get_slot WHERE id = ?` hat
/// diesen Filter nicht.
///
/// **Was NICHT die Ursache war.** Die Lesestellen standen auf `Iso8601::DATE`,
/// einem Format ohne Zeitanteil — das sieht nach der Ursache aus, ist aber
/// keine: `time`s well-known-`Iso8601`-Parser ignoriert die Konfiguration beim
/// Parsen und akzeptiert jedes gültige ISO8601, Zeitanteil inklusive. Auf
/// `DATE_TIME` umgestellt ist es trotzdem, weil es die Absicht ausspricht und
/// zur Schreibseite passt — verhaltensgleich ist beides.
///
/// **Der Fallback bleibt dauerhaft.** Geschrieben wird jetzt
/// `Iso8601::DATE_TIME` wie bei `booking`, aber der Altbestand liegt weiter im
/// `Display`-Format in der Datenbank. Eine SQL-Normalisierung ist dafür kein
/// Weg: SQLites Datumsfunktionen liefern für eine einstellige Stunde `NULL`,
/// das Format ist für sie nicht verarbeitbar.
fn parse_slot_deleted(value: &str) -> Result<PrimitiveDateTime, DaoError> {
    match PrimitiveDateTime::parse(value, &Iso8601::DATE_TIME) {
        Ok(parsed) => Ok(parsed),
        Err(iso_error) => {
            let legacy = format_description!(
                "[year]-[month]-[day] [hour padding:none]:[minute]:[second].[subsecond]"
            );
            // Schlaegt auch der Fallback fehl, ist der ISO-Fehler die
            // nuetzlichere Meldung -- ISO8601 ist das Soll-Format.
            PrimitiveDateTime::parse(value, legacy).map_err(|_| DaoError::from(iso_error))
        }
    }
}

impl SlotDaoImpl {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl dao::slot::SlotDao for SlotDaoImpl {
    type Transaction = TransactionImpl;

    async fn get_slots(&self, tx: Self::Transaction) -> Result<Arc<[SlotEntity]>, DaoError> {
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, min_resources, max_paid_employees, valid_from, valid_to, deleted, update_version, shiftplan_id FROM slot WHERE deleted IS NULL")
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .iter()
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    min_resources: row.min_resources as u8,
                    max_paid_employees: row.max_paid_employees.map(|n| n as u8),
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_deref()
                        .map(parse_slot_deleted)
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                    shiftplan_id: row.shiftplan_id.as_ref().map(|id| Uuid::from_slice(id)).transpose()?,
                })
            })
            .collect()
    }

    async fn get_slot(
        &self,
        id: &Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SlotEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        let result = query!(r"SELECT id, day_of_week, time_from, time_to, min_resources, max_paid_employees, valid_from, valid_to, deleted, update_version, shiftplan_id FROM slot WHERE id = ?", id_vec)
            .fetch_optional(tx.tx.lock().await.as_mut())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    min_resources: row.min_resources as u8,
                    max_paid_employees: row.max_paid_employees.map(|n| n as u8),
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_deref()
                        .map(parse_slot_deleted)
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                    shiftplan_id: row.shiftplan_id.as_ref().map(|id| Uuid::from_slice(id)).transpose()?,
                })
            })
            .transpose()
    }

    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        shiftplan_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[SlotEntity]>, DaoError> {
        let monday = Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)?;
        let sunday = Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday)?;
        let monday_str = monday.format(&Iso8601::DATE)?;
        let sunday_str = sunday.format(&Iso8601::DATE)?;
        let shiftplan_id_vec = shiftplan_id.as_bytes().to_vec();
        let result = query!(r"
                SELECT id, day_of_week, time_from, time_to, min_resources, max_paid_employees, valid_from, valid_to, deleted, update_version, shiftplan_id
                FROM slot
                WHERE deleted IS NULL
                AND valid_from <= ?
                AND (valid_to IS NULL OR valid_to >= ?)
                AND shiftplan_id = ?", sunday_str, monday_str, shiftplan_id_vec)
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .iter()
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    min_resources: row.min_resources as u8,
                    max_paid_employees: row.max_paid_employees.map(|n| n as u8),
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_deref()
                        .map(parse_slot_deleted)
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                    shiftplan_id: row.shiftplan_id.as_ref().map(|id| Uuid::from_slice(id)).transpose()?,
                })
            })
            .collect()
    }

    async fn get_slots_for_week_all_plans(
        &self,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SlotEntity]>, DaoError> {
        let monday = Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)?;
        let sunday = Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday)?;
        let monday_str = monday.format(&Iso8601::DATE)?;
        let sunday_str = sunday.format(&Iso8601::DATE)?;
        let result = query!(r"
                SELECT slot.id, slot.day_of_week, slot.time_from, slot.time_to, slot.min_resources, slot.max_paid_employees, slot.valid_from, slot.valid_to, slot.deleted, slot.update_version, slot.shiftplan_id
                FROM slot
                LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id
                WHERE slot.deleted IS NULL
                AND slot.valid_from <= ?
                AND (slot.valid_to IS NULL OR slot.valid_to >= ?)
                AND (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)", sunday_str, monday_str)
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_err(|err| DaoError::DatabaseQueryError(Box::new(err)))?;
        result
            .iter()
            .map(|row| {
                Ok(SlotEntity {
                    id: Uuid::from_slice(row.id.as_ref())?,
                    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
                        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
                    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
                    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
                    min_resources: row.min_resources as u8,
                    max_paid_employees: row.max_paid_employees.map(|n| n as u8),
                    valid_from: Date::parse(&row.valid_from, &Iso8601::DATE)?,
                    valid_to: row
                        .valid_to
                        .as_ref()
                        .map(|valid_to| Date::parse(valid_to, &Iso8601::DATE))
                        .transpose()?,
                    deleted: row
                        .deleted
                        .as_deref()
                        .map(parse_slot_deleted)
                        .transpose()?,
                    version: Uuid::from_slice(&row.update_version)?,
                    shiftplan_id: row.shiftplan_id.as_ref().map(|id| Uuid::from_slice(id)).transpose()?,
                })
            })
            .collect()
    }

    async fn create_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let time_format = format_description!("[hour]:[minute]:[second].0");
        let id_vec = slot.id.as_bytes().to_vec();
        let version_vec = slot.version.as_bytes().to_vec();
        let day_of_week = slot.day_of_week.to_number();
        let from = slot.from.format(&time_format)?;
        let to = slot.to.format(&time_format)?;
        let valid_from = slot.valid_from.to_string();
        let valid_to = slot.valid_to.map(|valid_to| valid_to.to_string());
        // ISO8601 mit Zeitanteil -- `to_string()` (time's Display) ist kein
        // ISO8601 und war nicht wieder einlesbar. Siehe `parse_slot_deleted`.
        let deleted = slot
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()?;
        let min_resources = slot.min_resources;
        let max_paid_employees = slot.max_paid_employees;
        let shiftplan_id_vec = slot.shiftplan_id.map(|id| id.as_bytes().to_vec());
        query!("INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version, update_process, min_resources, max_paid_employees, shiftplan_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id_vec,
            day_of_week,
            from,
            to,
            valid_from,
            valid_to,
            deleted,
            version_vec,
            process,
            min_resources,
            max_paid_employees,
            shiftplan_id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = slot.id.as_bytes().to_vec();
        let version_vec = slot.version.as_bytes().to_vec();
        let valid_to = slot.valid_to.map(|valid_to| valid_to.to_string());
        // ISO8601 mit Zeitanteil -- `to_string()` (time's Display) ist kein
        // ISO8601 und war nicht wieder einlesbar. Siehe `parse_slot_deleted`.
        let deleted = slot
            .deleted
            .as_ref()
            .map(|deleted| deleted.format(&Iso8601::DATE_TIME))
            .transpose()?;
        let max_paid_employees = slot.max_paid_employees;
        query!("UPDATE slot SET valid_to = ?, deleted = ?, max_paid_employees = ?, update_version = ?, update_process = ? WHERE id = ?",
            valid_to,
            deleted,
            max_paid_employees,
            version_vec,
            process,
            id_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}
