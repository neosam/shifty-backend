//! Phase 62 Plan 05 (PADM-01 / PADM-08, D-62-08) — HTTP-Integrationstest fuer
//! den Aggregat-Endpoint `GET /person-directory`.
//!
//! **Warum die HR-Seite hier ausdruecklich geprueft wird (RESEARCH.md Pitfall
//! 4).** Der Basic-Tier `PersonService` aus Phase 61 ist HR-gegated. Der
//! Aggregat-Endpoint dieser Phase darf dieses Gate NICHT erben: das Verzeichnis
//! gibt Benutzernamen und Rollenzuordnungen preis (T-62-05) und ist damit
//! Admin-Sache (D-62-08). Der wahrscheinlichste Fehler dieser Phase ist ein
//! kopiertes HR-Gate — genau den faengt `hr_ohne_admin_bekommt_403`.
//!
//! **Warum die Rolle `hr` der saubere Negativfall ist.** Die Migration
//! `20240426150045_user-roles.sql` bindet an die Rolle `hr` genau das Privileg
//! `hr` (`role_privilege('hr','hr')`) und **kein** `admin`. Ein Benutzer mit
//! ausschliesslich dieser Rolle ist also einerseits ein vollwertiger
//! HR-Anwender, andererseits garantiert kein Admin. Die Rolle `admin` traegt
//! dagegen `admin`, `sales` und `hr` gemeinsam.
//!
//! **Erwartet wird 403, nicht 200-mit-leerer-Liste.** Der Endpoint unterscheidet
//! sich hier bewusst vom Freiwilligen-Statistik-Endpoint (`voluntary_stats.rs`),
//! der auf `null` redigiert. Eine redigierte oder leere Antwort waere von einem
//! tatsaechlich leeren Verzeichnis nicht zu unterscheiden (62-03-SUMMARY,
//! T-62-05).
//!
//! **Hinweis zur Zeilenzahl.** `TestSetup::new()` legt via `create_admin_user`
//! den Entwicklungs-Admin `DEVUSER` an, und `create_user` vergibt seit D-61-01
//! jedem Benutzer sofort eine eigene `person`-Identitaet. `DEVUSER` ist damit
//! selbst eine legitime Verzeichniszeile der Auspraegung `UserOnly`. Die
//! Gesamtliste traegt also die drei geseedeten Personen **plus** `DEVUSER`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use dao::PermissionDao;
use dao_impl_sqlite::PermissionDaoImpl;
use http_body_util::BodyExt;
use rest::person_directory::generate_route;
use rest::Context as RestContext;
use rest_types::{PersonEntryTO, PersonKindTO};
use tower::ServiceExt;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Der Pfad steht genau einmal hier und ist zeichengleich mit dem
/// `.nest(...)`-Praefix in `rest/src/lib.rs`, der URL in
/// `shifty-dioxus/src/api.rs` und dem `[[web.proxy]]`-Eintrag in
/// `shifty-dioxus/Dioxus.toml`.
const DIRECTORY_PATH: &str = "/person-directory";

/// Anzeigename der verknuepften Person (Sales Person **und** Benutzerkonto).
const LINKED_DISPLAY_NAME: &str = "Anna Verknuepft";
/// Benutzername der verknuepften Person.
const LINKED_USERNAME: &str = "anna-linked";
/// Anzeigename des reinen Benutzerkontos — hier ist er der Benutzername.
const USER_ONLY_NAME: &str = "bertram-useronly";
/// Anzeigename des reinen Verkaeufers (ohne Benutzerkonto).
const SALES_ONLY_DISPLAY_NAME: &str = "Clara Nurverkauf";

/// Die drei Person-Identitaeten des Szenarios, in der Reihenfolge
/// verknuepft / nur-Benutzer / nur-Verkaeufer.
struct SeededPersons {
    linked: Uuid,
    user_only: Uuid,
    sales_only: Uuid,
}

/// Legt die drei Auspraegungen per direktem `sqlx`-Insert an.
///
/// Direkte Inserts sind hier die richtige Wahl, weil Phase 62 keinen
/// Schreibpfad fuer Personen ueber die Services kennt — das Anlegen mit
/// Auspraegungen ist Gegenstand von Phase 63. Muster uebernommen aus den
/// Seed-Helfern in `service_impl/src/test/person_directory.rs`.
///
/// Laeuft **nach** `TestSetup::new()`, weil `TestSetup` intern
/// `BasicDaoImpl::clear_all()` aufruft und dabei alle `sales_person`-Zeilen
/// sowie jede traegerlos gewordene `person`-Zeile loescht.
async fn seed_directory_szenario(test_setup: &TestSetup) -> SeededPersons {
    let pool = test_setup.pool.as_ref();

    let linked = Uuid::new_v4();
    let user_only = Uuid::new_v4();
    let sales_only = Uuid::new_v4();

    for person_id in [linked, user_only, sales_only] {
        sqlx::query(
            "INSERT INTO person (id, created, update_process, update_version) \
             VALUES (?1, '2026-01-01T00:00:00.000000000Z', 'seed-62-05', ?2)",
        )
        .bind(person_id.as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed person");
    }

    for (person_id, name) in [
        (linked, LINKED_DISPLAY_NAME),
        (sales_only, SALES_ONLY_DISPLAY_NAME),
    ] {
        sqlx::query(
            "INSERT INTO sales_person \
             (id, name, background_color, is_paid, inactive, deleted, person_id, \
              update_process, update_version) \
             VALUES (?1, ?2, '#FFF', true, false, NULL, ?3, 'seed-62-05', ?4)",
        )
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed sales_person");
    }

    for (person_id, name) in [(linked, LINKED_USERNAME), (user_only, USER_ONLY_NAME)] {
        sqlx::query(
            "INSERT INTO user (name, person_id, update_process) VALUES (?1, ?2, 'seed-62-05')",
        )
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed user");
    }

    SeededPersons {
        linked,
        user_only,
        sales_only,
    }
}

/// Legt einen Benutzer an, der ausschliesslich die Rolle `hr` traegt.
///
/// Aufrufmuster wie `create_admin_user` in `shifty_bin/src/main.rs`, nur mit
/// der anderen Rolle. Muss **nach** `TestSetup::new()` laufen.
async fn seed_hr_only_user(test_setup: &TestSetup, username: &str) {
    use dao::TransactionDao as _;
    let permission_dao = PermissionDaoImpl::new(test_setup.pool.clone());
    // Phase 63 Plan 01: `create_user` nimmt die Transaktion vom Aufrufer.
    let transaction_dao = dao_impl_sqlite::TransactionDaoImpl::new(test_setup.pool.clone());
    let tx = transaction_dao
        .use_transaction(None)
        .await
        .expect("transaction");
    permission_dao
        .create_user(
            &dao::UserEntity {
                name: username.into(),
            },
            None,
            "seed-62-05",
            tx.clone(),
        )
        .await
        .expect("create hr user");
    transaction_dao.commit(tx).await.expect("commit hr user");
    permission_dao.add_user_role(username, "hr", "seed-62-05").await.expect("bind hr role");
}

/// Baut den Router genau so, wie ihn `rest::start_server` nestet, und schickt
/// ein `GET` auf den Verzeichnis-Pfad im Namen von `username`.
async fn get_directory_as(test_setup: &TestSetup, username: &str) -> (StatusCode, Vec<u8>) {
    let router = axum::Router::new()
        .nest(DIRECTORY_PATH, generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(Some(Arc::<str>::from(username)) as RestContext));

    let req = Request::builder()
        .method("GET")
        .uri(DIRECTORY_PATH)
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
    (status, body)
}

fn entry_of(entries: &[PersonEntryTO], person_id: Uuid) -> &PersonEntryTO {
    entries
        .iter()
        .find(|entry| entry.person_id == person_id)
        .unwrap_or_else(|| panic!("Person {person_id} fehlt im Verzeichnis: {entries:?}"))
}

/// PADM-01 / PADM-08 (positiv): Der Admin bekommt 200 und alle drei
/// Auspraegungen nebeneinander, jede mit nicht-leerem Anzeigenamen.
#[tokio::test]
async fn admin_bekommt_200() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_directory_szenario(&test_setup).await;

    let (status, body) = get_directory_as(&test_setup, "DEVUSER").await;
    assert_eq!(status, StatusCode::OK, "Admin muss 200 bekommen");

    let entries: Vec<PersonEntryTO> = serde_json::from_slice(&body).unwrap_or_else(|e| {
        panic!(
            "Antwort muss nach Vec<PersonEntryTO> deserialisieren: {e}; body={}",
            String::from_utf8_lossy(&body)
        )
    });

    let linked = entry_of(&entries, seeded.linked);
    assert_eq!(linked.kind, PersonKindTO::Linked);
    assert_eq!(linked.display_name.as_ref(), LINKED_DISPLAY_NAME);
    assert_eq!(linked.username.as_deref(), Some(LINKED_USERNAME));

    let user_only = entry_of(&entries, seeded.user_only);
    assert_eq!(user_only.kind, PersonKindTO::UserOnly);
    assert_eq!(user_only.display_name.as_ref(), USER_ONLY_NAME);

    let sales_only = entry_of(&entries, seeded.sales_only);
    assert_eq!(sales_only.kind, PersonKindTO::SalesPersonOnly);
    assert_eq!(sales_only.display_name.as_ref(), SALES_ONLY_DISPLAY_NAME);
    assert!(
        sales_only.username.is_none(),
        "Ein reiner Verkaeufer hat kein Benutzerkonto: {sales_only:?}"
    );

    for entry in [linked, user_only, sales_only] {
        assert!(
            !entry.display_name.trim().is_empty(),
            "Anzeigename darf nie leer sein: {entry:?}"
        );
    }

    // Genau drei geseedete Zeilen, je eine Auspraegung — plus DEVUSER, den
    // `TestSetup::new()` als `UserOnly` anlegt (siehe Modulkopf). Die
    // Gesamtzahl wird mitgeprueft, damit der Endpoint weder Zeilen
    // unterschlaegt noch dupliziert.
    assert_eq!(
        entries.len(),
        4,
        "erwartet: 3 geseedete Personen + DEVUSER; bekommen: {entries:?}"
    );
}

/// PADM-08 (negativ, D-62-08, T-62-01/T-62-05): Ein HR-Anwender ohne
/// Admin-Rolle bekommt 403 — **kein** 200 mit leerer Liste und keine Redaktion.
#[tokio::test]
async fn hr_ohne_admin_bekommt_403() {
    let test_setup = TestSetup::new().await;
    seed_directory_szenario(&test_setup).await;
    seed_hr_only_user(&test_setup, "hilde-hr").await;

    let (status, body) = get_directory_as(&test_setup, "hilde-hr").await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "HR ohne Admin muss 403 bekommen, nicht 200 mit leerer Liste; body={}",
        String::from_utf8_lossy(&body)
    );

    let body_str = String::from_utf8_lossy(&body).to_string();
    for name in [
        LINKED_DISPLAY_NAME,
        LINKED_USERNAME,
        USER_ONLY_NAME,
        SALES_ONLY_DISPLAY_NAME,
    ] {
        assert!(
            !body_str.contains(name),
            "Die 403-Antwort darf keinen geseedeten Namen preisgeben ({name}): {body_str}"
        );
    }
}
