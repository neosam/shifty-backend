//! Einmaliger Rücklauf: stellt `booking.created_by` wieder her, wo ein
//! Slot-Umbau es durch `"system"` ersetzt hat.
//!
//! # Hintergrund
//!
//! Die `*`-Markierung im Schichtplan zeigt dem Schichtplaner, wer sich selbst
//! eingetragen hat. Sie hängt an `ShiftplanBooking::self_added`, und das wird
//! nicht gespeichert, sondern in `service_impl/src/shiftplan.rs` abgeleitet aus
//! `booking.created_by == <Benutzer dieser Sales Person>`.
//!
//! Bis zum Fix in `shiftplan_edit.rs` haben beide Umbau-Pfade (`modify_slot`,
//! `modify_slot_single_week`) die Buchungen eines Slots gelöscht und neu
//! angelegt und dabei `created_by` zurückgesetzt. Die Fallback-Kette in
//! `BookingService::create` stempelte dann `"system"`, weil `current_user` bei
//! `Authentication::Full` `None` ist. Die Urheberschaft ist dadurch nicht
//! verloren — sie steht weiterhin in der soft-gelöschten Vorgänger-Zeile.
//!
//! # Verfahren
//!
//! Es gibt **keinen** Fremdschlüssel von der neuen auf die alte Zeile: die neue
//! Buchung bekommt eine frische `id`, eine frische `update_version` und zeigt auf
//! den neuen Slot. Rekonstruiert wird über die Zeit:
//!
//! 1. Gruppieren nach (`sales_person_id`, `year`, `calendar_week`,
//!    `slot.day_of_week`). Der `day_of_week` bleibt beim Umbau erhalten, weil
//!    `modify_slot` den neuen Slot als Kopie des alten anlegt — damit bleiben
//!    mehrere Buchungen derselben Person in derselben Woche unterscheidbar.
//! 2. Innerhalb der Gruppe nach `created` aufsteigend ordnen. Die Zeitstempel
//!    tragen Nanosekunden (`Iso8601::DATE_TIME` erbt
//!    `TimePrecision::Second { decimal_digits: 9 }`), sind also eindeutig. Der
//!    Zeitstempel dient zum **Ordnen**, nicht zum Verbinden — ein
//!    Gleichheits-Join auf `Vorgänger.deleted == Nachfolger.created` würde
//!    fehlschlagen, weil `delete()` und `create()` die Uhr getrennt abfragen.
//! 3. Von der lebenden Zeile rückwärts laufen, **nur** über Glieder, deren
//!    `deleted_by = 'system'` ist. Steht dort ein echter Benutzername, hat ein
//!    Mensch gelöscht — dann sind es zwei getrennte Episoden und die Kette endet.
//!
//! # Was ausdrücklich NICHT angefasst wird
//!
//! Nicht jedes `"system"` ist Schaden. Der Auto-Rebooking-Cron
//! (`voluntary_rebooking_scheduler`) und andere Systempfade buchen legitim als
//! `"system"`. Solche Zeilen haben keine system-gelöschte Vorgängerkette, die zu
//! einem echten Namen führt — sie landen in `Unresolved` und bleiben unverändert.
//! Mehrdeutige Fälle werden gemeldet, nicht geraten.
//!
//! # Warum Laufzeit-Queries statt `query!`
//!
//! Dieses Modul liest soft-gelöschte Zeilen, die kein DAO herausgibt, und läuft
//! genau einmal. Compile-time-geprüfte Makros würden den geteilten
//! `.sqlx`-Cache (197 Einträge) um Abfragen erweitern, die danach wieder
//! verschwinden. Die Abfragen hier sind deshalb bewusst laufzeitgeprüft; das ist
//! auf einen Wartungspfad begrenzt und berührt die Produktionspfade nicht.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// Prozess-Kennung, die der Rücklauf in `booking.update_process` hinterlässt,
/// damit die Änderung im Audit-Trail als solche erkennbar bleibt.
const BACKFILL_PROCESS: &str = "backfill-booking-authorship";

/// Der Sentinel, den die Fallback-Kette in `BookingService::create` setzt.
const SYSTEM_SENTINEL: &str = "system";

#[derive(Debug, Clone)]
struct BookingRow {
    id: Uuid,
    sales_person_id: Uuid,
    year: i64,
    calendar_week: i64,
    day_of_week: i64,
    created: String,
    deleted: Option<String>,
    created_by: Option<String>,
    deleted_by: Option<String>,
}

/// Gruppenschlüssel: Person, Woche und Wochentag. Der Wochentag hält mehrere
/// Buchungen derselben Person in derselben Woche auseinander.
type GroupKey = (Uuid, i64, i64, i64);

/// Ergebnis der Rückverfolgung für **eine** lebende Buchung.
#[derive(Debug)]
enum Verdict {
    /// Eindeutig auf einen echten Benutzernamen zurückgeführt.
    Resolved { to: String, steps: usize },
    /// Kette endet an einer Zeile ohne `created_by` — Altbestand vor der
    /// Audit-Migration (`20250115000000_add-user-tracking-to-booking.sql`).
    UnresolvedNullAuthor,
    /// Kette endet an einem echten `"system"` ohne Vorgänger — vermutlich eine
    /// legitime System-Buchung (Auto-Rebooking-Cron), kein Umbau-Schaden.
    UnresolvedNoChain,
    /// Kette bricht an einer von einem Menschen gelöschten Zeile ab — zwei
    /// getrennte Episoden, keine Umbau-Kette.
    UnresolvedHumanDeletion { stopped_at: String },
    /// Mehrere Vorgänger auf derselben Stufe mit **verschiedenen** Urhebern.
    Ambiguous { candidates: Vec<String> },
}

/// Ein echter Urheber ist ein nicht-leerer Name, der nicht der System-Sentinel ist.
fn is_real_author(created_by: Option<&str>) -> bool {
    matches!(created_by, Some(name) if !name.is_empty() && name != SYSTEM_SENTINEL)
}

/// Verfolgt die Urheberschaft einer lebenden Buchung über ihre Vorgängerkette
/// zurück. `group` ist nach `created` aufsteigend sortiert.
fn trace_author(group: &[BookingRow], live_index: usize) -> Verdict {
    let mut steps = 0usize;
    let mut cursor = live_index;

    loop {
        // Alle unmittelbaren Vorgänger sind die Zeilen vor `cursor`. Relevant ist
        // die jeweils jüngste — sie ist die, aus der `cursor` entstanden ist.
        if cursor == 0 {
            return if steps == 0 {
                Verdict::UnresolvedNoChain
            } else {
                Verdict::UnresolvedNullAuthor
            };
        }

        // Kandidaten auf dieser Stufe: Zeilen mit identischem `created` wären
        // nicht unterscheidbar. Nanosekunden machen das praktisch unmöglich,
        // aber wenn es auftritt, wird es gemeldet statt geraten.
        let predecessor = &group[cursor - 1];
        let same_instant: Vec<&BookingRow> = group[..cursor]
            .iter()
            .filter(|r| r.created == predecessor.created)
            .collect();
        if same_instant.len() > 1 {
            let mut candidates: Vec<String> = same_instant
                .iter()
                .map(|r| r.created_by.clone().unwrap_or_default())
                .collect();
            candidates.sort();
            candidates.dedup();
            if candidates.len() > 1 {
                return Verdict::Ambiguous { candidates };
            }
        }

        // Nur über Umbau-Glieder zurücklaufen. Hat ein Mensch gelöscht, endet
        // die Kette hier — was davor liegt, gehört zu einer anderen Episode.
        match predecessor.deleted_by.as_deref() {
            Some(SYSTEM_SENTINEL) => {}
            Some(other) if !other.is_empty() => {
                return Verdict::UnresolvedHumanDeletion {
                    stopped_at: other.to_string(),
                }
            }
            // `deleted_by` NULL: Altbestand vor der Audit-Migration. Keine
            // belastbare Aussage, ob Umbau oder Mensch — nicht weiterlaufen.
            _ => return Verdict::UnresolvedNullAuthor,
        }

        steps += 1;

        if is_real_author(predecessor.created_by.as_deref()) {
            return Verdict::Resolved {
                to: predecessor
                    .created_by
                    .clone()
                    .expect("is_real_author garantiert Some"),
                steps,
            };
        }

        cursor -= 1;
    }
}

/// Ein geplanter Schreibvorgang.
#[derive(Debug)]
pub struct PlannedUpdate {
    pub booking_id: Uuid,
    pub new_created_by: String,
    pub steps: usize,
}

/// Was die Analyse insgesamt ergeben hat.
#[derive(Debug, Default)]
pub struct Analysis {
    pub total_bookings: usize,
    pub live_bookings: usize,
    pub live_system_bookings: usize,
    pub planned: Vec<PlannedUpdate>,
    pub unresolved_null_author: Vec<Uuid>,
    pub unresolved_no_chain: Vec<Uuid>,
    pub unresolved_human_deletion: Vec<(Uuid, String)>,
    pub ambiguous: Vec<(Uuid, Vec<String>)>,
}

async fn load_rows(pool: &SqlitePool) -> Result<Vec<BookingRow>, sqlx::Error> {
    // Soft-gelöschte Zeilen sind hier ausdrücklich GEWOLLT — sie tragen die
    // gesuchte Urheberschaft. Der Slot-Join liefert den `day_of_week`; auch
    // soft-gelöschte Slots bleiben dabei drin, weil ein Umbau den alten Slot
    // nur verkürzt oder (im Randfall) löscht.
    let rows = sqlx::query(
        "SELECT b.id, b.sales_person_id, b.year, b.calendar_week, \
                b.created, b.deleted, b.created_by, b.deleted_by, \
                s.day_of_week \
         FROM booking b JOIN slot s ON s.id = b.slot_id \
         ORDER BY b.created ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id_blob: Vec<u8> = row.try_get("id")?;
        let sp_blob: Vec<u8> = row.try_get("sales_person_id")?;
        let id = Uuid::from_slice(&id_blob).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let sales_person_id =
            Uuid::from_slice(&sp_blob).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        out.push(BookingRow {
            id,
            sales_person_id,
            year: row.try_get("year")?,
            calendar_week: row.try_get("calendar_week")?,
            day_of_week: row.try_get("day_of_week")?,
            created: row.try_get("created")?,
            deleted: row.try_get("deleted")?,
            created_by: row.try_get("created_by")?,
            deleted_by: row.try_get("deleted_by")?,
        });
    }
    Ok(out)
}

/// Liest den Bestand und entscheidet für jede lebende `"system"`-Buchung, ob und
/// worauf sie zurückgeführt werden kann. Schreibt nichts.
pub async fn analyze(pool: &SqlitePool) -> Result<Analysis, sqlx::Error> {
    let rows = load_rows(pool).await?;

    let mut groups: HashMap<GroupKey, Vec<BookingRow>> = HashMap::new();
    for row in &rows {
        groups
            .entry((
                row.sales_person_id,
                row.year,
                row.calendar_week,
                row.day_of_week,
            ))
            .or_default()
            .push(row.clone());
    }

    let mut analysis = Analysis {
        total_bookings: rows.len(),
        live_bookings: rows.iter().filter(|r| r.deleted.is_none()).count(),
        ..Default::default()
    };

    for group in groups.values_mut() {
        // `load_rows` sortiert global; innerhalb der Gruppe bleibt die Ordnung
        // erhalten. Explizit nachsortieren, damit die Invariante lokal gilt.
        group.sort_by(|a, b| a.created.cmp(&b.created));

        for (index, row) in group.iter().enumerate() {
            if row.deleted.is_some() {
                continue;
            }
            if is_real_author(row.created_by.as_deref()) {
                continue;
            }
            if row.created_by.is_none() {
                // Altbestand ohne Urheber — kein Umbau-Schaden, nichts zu tun.
                continue;
            }
            analysis.live_system_bookings += 1;

            match trace_author(group, index) {
                Verdict::Resolved { to, steps } => analysis.planned.push(PlannedUpdate {
                    booking_id: row.id,
                    new_created_by: to,
                    steps,
                }),
                Verdict::UnresolvedNullAuthor => {
                    analysis.unresolved_null_author.push(row.id);
                }
                Verdict::UnresolvedNoChain => analysis.unresolved_no_chain.push(row.id),
                Verdict::UnresolvedHumanDeletion { stopped_at } => {
                    analysis.unresolved_human_deletion.push((row.id, stopped_at));
                }
                Verdict::Ambiguous { candidates } => {
                    analysis.ambiguous.push((row.id, candidates));
                }
            }
        }
    }

    analysis.planned.sort_by_key(|p| p.booking_id);
    Ok(analysis)
}

/// Schreibt die geplanten Änderungen. `update_process` wird auf
/// [`BACKFILL_PROCESS`] gesetzt, damit die Änderung im Audit-Trail als Rücklauf
/// erkennbar ist, und `update_version` auf eine neue UUID.
async fn apply(pool: &SqlitePool, analysis: &Analysis) -> Result<usize, sqlx::Error> {
    let mut written = 0usize;
    for planned in &analysis.planned {
        let id_blob = planned.booking_id.as_bytes().to_vec();
        let version_blob = Uuid::new_v4().as_bytes().to_vec();
        // Bedingung auf `created_by = 'system'` macht den Lauf idempotent: ein
        // zweiter Durchgang findet nichts mehr.
        let result = sqlx::query(
            "UPDATE booking SET created_by = ?, update_version = ?, update_process = ? \
             WHERE id = ? AND created_by = ? AND deleted IS NULL",
        )
        .bind(&planned.new_created_by)
        .bind(version_blob)
        .bind(BACKFILL_PROCESS)
        .bind(id_blob)
        .bind(SYSTEM_SENTINEL)
        .execute(pool)
        .await?;
        written += result.rows_affected() as usize;
    }
    Ok(written)
}

fn print_analysis(analysis: &Analysis) {
    println!("Buchungen insgesamt (inkl. soft-geloescht): {}", analysis.total_bookings);
    println!("davon lebend:                               {}", analysis.live_bookings);
    println!(
        "lebend mit created_by = '{}':               {}",
        SYSTEM_SENTINEL, analysis.live_system_bookings
    );
    println!();
    println!("auf einen echten Urheber zurueckgefuehrt:   {}", analysis.planned.len());
    println!("nicht zurueckfuehrbar:");
    println!(
        "  Kette endet ohne Urheber (Altbestand):    {}",
        analysis.unresolved_null_author.len()
    );
    println!(
        "  keine Vorgaengerkette (echte System-Bu.): {}",
        analysis.unresolved_no_chain.len()
    );
    println!(
        "  Kette durch Menschen-Loeschung getrennt:  {}",
        analysis.unresolved_human_deletion.len()
    );
    println!("  mehrdeutig (wird NICHT geraten):          {}", analysis.ambiguous.len());

    if !analysis.ambiguous.is_empty() {
        println!();
        println!("Mehrdeutige Faelle — brauchen eine Entscheidung:");
        for (id, candidates) in &analysis.ambiguous {
            println!("  {id}  Kandidaten: {}", candidates.join(", "));
        }
    }
    if !analysis.unresolved_human_deletion.is_empty() {
        println!();
        println!("Durch Menschen-Loeschung getrennt (Stichprobe, max. 10):");
        for (id, who) in analysis.unresolved_human_deletion.iter().take(10) {
            println!("  {id}  geloescht von: {who}");
        }
    }
}

/// Einstiegspunkt. `mode` ist `--report`, `--dry-run` oder `--apply`.
pub async fn run(pool: Arc<SqlitePool>, mode: &str) {
    let analysis = match analyze(pool.as_ref()).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("FEHLER bei der Analyse: {e}");
            std::process::exit(1);
        }
    };

    match mode {
        "--report" => {
            println!("== Rücklauf Buchungs-Urheberschaft: BERICHT (schreibt nichts) ==\n");
            print_analysis(&analysis);
        }
        "--dry-run" => {
            println!("== Rücklauf Buchungs-Urheberschaft: TROCKENLAUF (schreibt nichts) ==\n");
            print_analysis(&analysis);
            println!();
            println!("Geplante Aenderungen:");
            for planned in &analysis.planned {
                println!(
                    "  {}  created_by: '{}' -> '{}'  ({} Glied(er) zurueck)",
                    planned.booking_id, SYSTEM_SENTINEL, planned.new_created_by, planned.steps
                );
            }
        }
        "--apply" => {
            println!("== Rücklauf Buchungs-Urheberschaft: SCHREIBEN ==\n");
            print_analysis(&analysis);
            println!();
            match apply(pool.as_ref(), &analysis).await {
                Ok(written) => {
                    println!("Geschrieben: {written} von {} geplanten Zeilen.", analysis.planned.len());
                    if written != analysis.planned.len() {
                        println!(
                            "HINWEIS: Differenz bedeutet, dass Zeilen sich zwischen Analyse und \
                             Schreiben geaendert haben. Lauf wiederholen (er ist idempotent)."
                        );
                    }
                }
                Err(e) => {
                    eprintln!("FEHLER beim Schreiben: {e}");
                    std::process::exit(1);
                }
            }
        }
        other => {
            eprintln!("Unbekannter Modus '{other}'. Erwartet: --report | --dry-run | --apply");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        id: u128,
        created: &str,
        deleted: Option<&str>,
        created_by: Option<&str>,
        deleted_by: Option<&str>,
    ) -> BookingRow {
        BookingRow {
            id: Uuid::from_u128(id),
            sales_person_id: Uuid::from_u128(0xAA),
            year: 2026,
            calendar_week: 20,
            day_of_week: 1,
            created: created.to_string(),
            deleted: deleted.map(str::to_string),
            created_by: created_by.map(str::to_string),
            deleted_by: deleted_by.map(str::to_string),
        }
    }

    /// Der Normalfall: ein Umbau. Die lebende Zeile trägt `"system"`, ihre
    /// system-gelöschte Vorgängerin den echten Namen.
    #[test]
    fn traces_single_restructure_back_to_real_author() {
        let group = vec![
            row(1, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")),
            row(2, "2026-05-02T09:00:00.000000003", None, Some("system"), None),
        ];
        match trace_author(&group, 1) {
            Verdict::Resolved { to, steps } => {
                assert_eq!(to, "alice");
                assert_eq!(steps, 1);
            }
            other => panic!("erwartet Resolved, war {other:?}"),
        }
    }

    /// Zwei Umbauten hintereinander: die Kette muss über das mittlere
    /// `"system"`-Glied hinweg bis zum echten Namen laufen.
    #[test]
    fn traces_chained_restructures() {
        let group = vec![
            row(1, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")),
            row(2, "2026-05-02T09:00:00.000000003", Some("2026-05-03T08:00:00.000000004"), Some("system"), Some("system")),
            row(3, "2026-05-03T08:00:00.000000005", None, Some("system"), None),
        ];
        match trace_author(&group, 2) {
            Verdict::Resolved { to, steps } => {
                assert_eq!(to, "alice");
                assert_eq!(steps, 2, "zwei Glieder zurueck");
            }
            other => panic!("erwartet Resolved, war {other:?}"),
        }
    }

    /// Hat ein Mensch gelöscht, sind es zwei Episoden. Der frühere Name darf
    /// NICHT übernommen werden — sonst schreibt der Rücklauf eine Urheberschaft
    /// an eine Buchung, die gar nicht aus jener Kette stammt.
    #[test]
    fn stops_at_human_deletion_instead_of_attributing() {
        let group = vec![
            row(1, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("bob")),
            row(2, "2026-05-02T09:00:00.000000003", None, Some("system"), None),
        ];
        match trace_author(&group, 1) {
            Verdict::UnresolvedHumanDeletion { stopped_at } => assert_eq!(stopped_at, "bob"),
            other => panic!("erwartet UnresolvedHumanDeletion, war {other:?}"),
        }
    }

    /// Eine lebende `"system"`-Buchung ohne jede Vorgängerin ist keine
    /// Umbau-Folge, sondern eine echte System-Buchung (Auto-Rebooking-Cron).
    /// Sie muss unangetastet bleiben.
    #[test]
    fn leaves_genuine_system_booking_alone() {
        let group = vec![row(1, "2026-05-01T10:00:00.000000001", None, Some("system"), None)];
        match trace_author(&group, 0) {
            Verdict::UnresolvedNoChain => {}
            other => panic!("erwartet UnresolvedNoChain, war {other:?}"),
        }
    }

    /// Altbestand vor der Audit-Migration: die Vorgängerin trägt kein
    /// `deleted_by`. Ob Umbau oder Mensch ist nicht entscheidbar — nicht laufen.
    #[test]
    fn stops_when_predecessor_has_no_deleted_by() {
        let group = vec![
            row(1, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("alice"), None),
            row(2, "2026-05-02T09:00:00.000000003", None, Some("system"), None),
        ];
        match trace_author(&group, 1) {
            Verdict::UnresolvedNullAuthor => {}
            other => panic!("erwartet UnresolvedNullAuthor, war {other:?}"),
        }
    }

    /// Zwei Vorgängerinnen mit identischem `created` und verschiedenen Urhebern:
    /// die Paarung ist nicht entscheidbar und wird gemeldet, nicht geraten.
    #[test]
    fn reports_ambiguity_instead_of_guessing() {
        let group = vec![
            row(1, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("alice"), Some("system")),
            row(2, "2026-05-01T10:00:00.000000001", Some("2026-05-02T09:00:00.000000002"), Some("carol"), Some("system")),
            row(3, "2026-05-02T09:00:00.000000003", None, Some("system"), None),
        ];
        match trace_author(&group, 2) {
            Verdict::Ambiguous { candidates } => {
                assert_eq!(candidates, vec!["alice".to_string(), "carol".to_string()]);
            }
            other => panic!("erwartet Ambiguous, war {other:?}"),
        }
    }

    #[test]
    fn real_author_recognises_system_and_empty_as_not_real() {
        assert!(is_real_author(Some("alice")));
        assert!(!is_real_author(Some("system")));
        assert!(!is_real_author(Some("")));
        assert!(!is_real_author(None));
    }
}
