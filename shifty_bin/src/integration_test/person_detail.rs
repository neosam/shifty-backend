//! Phase 63 Plan 06 (PADM-02 … PADM-05, PADM-07, PADM-08) — HTTP-Integrationstests
//! fuer die sieben `/person`-Operationen der Personen-Detailansicht.
//!
//! **Warum die HR-Seite hier ausdruecklich geprueft wird (T-63-06-A).** Der
//! Basic-Tier `PersonService` aus Phase 61 ist HR-gegated, und das Detail-
//! Aggregat konsumiert ihn. Es darf dieses Gate NICHT erben: die Detailansicht
//! gibt Benutzernamen, Rollen und Verkaeuferdaten preis und ist damit
//! Admin-Sache (D-62-08). Der wahrscheinlichste Fehler dieser Phase ist ein
//! kopiertes oder vergessenes Gate auf genau einer der sieben Operationen —
//! `hr_ohne_admin_bekommt_403_auf_jeder_route` faehrt deshalb alle sieben ab
//! und nicht nur eine Stichprobe.
//!
//! **Warum die Rolle `hr` der saubere Negativfall ist.** Die Migration
//! `20240426150045_user-roles.sql` bindet an die Rolle `hr` genau das Privileg
//! `hr` (`role_privilege('hr','hr')`) und **kein** `admin`. Ein Benutzer mit
//! ausschliesslich dieser Rolle ist also einerseits ein vollwertiger
//! HR-Anwender, andererseits garantiert kein Admin. Die Rolle `admin` traegt
//! dagegen `admin`, `sales` und `hr` gemeinsam.
//!
//! **Warum der 409-Test gegen `/sales-person/{id}/user` faehrt.** Success
//! Criterion 4 verlangt, dass ein bereits verknuepftes Ziel abgelehnt wird —
//! auch dann, wenn es in der Kandidatenliste gar nicht erst erscheint. Die
//! Ausblendung in der Liste ist Bequemlichkeit, die serverseitige Ablehnung ist
//! die Grenze (D-61-04). Der Test befaehrt deshalb bewusst den
//! **unveraenderten** Phase-61-Endpoint und nicht einen neuen Schreibpfad —
//! Phase 63 hat keinen gebaut.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use dao::PermissionDao;
use dao_impl_sqlite::PermissionDaoImpl;
use http_body_util::BodyExt;
use rest::person_detail::generate_route;
use rest::Context as RestContext;
use rest_types::{
    AddPersonFacetTO, CreatePersonEntryTO, CreatedPersonEntryTO, PersonEntryTO, PersonFacetTO,
    PersonKindTO, SalesPersonTO, UserTO,
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::integration_test::TestSetup;

/// Der Pfad steht genau einmal hier und ist zeichengleich mit dem
/// `.nest(...)`-Praefix in `rest/src/lib.rs`, der URL in
/// `shifty-dioxus/src/api.rs` und dem `[[web.proxy]]`-Eintrag in
/// `shifty-dioxus/Dioxus.toml`. Weicht eine der vier Stellen ab, laeuft der
/// Dev-Server in einen 404 — eine Fehlerklasse, die dieses Repo bereits
/// dreimal ausgeliefert hat.
const PERSON_PATH: &str = "/person";

/// Praefix des unveraenderten Phase-61-Verknuepfungs-Endpoints, gegen den der
/// 409-Regressionsschutz faehrt.
const SALES_PERSON_PATH: &str = "/sales-person";

/// Anzeigename der verknuepften Person (Sales Person **und** Benutzerkonto).
const LINKED_DISPLAY_NAME: &str = "Anna Verknuepft";
/// Benutzername der verknuepften Person.
const LINKED_USERNAME: &str = "anna-linked";
/// Anzeigename des reinen Benutzerkontos — hier ist er der Benutzername.
const USER_ONLY_NAME: &str = "bertram-useronly";
/// Anzeigename des reinen Verkaeufers (ohne Benutzerkonto).
const SALES_ONLY_DISPLAY_NAME: &str = "Clara Nurverkauf";

/// Die drei Person-Identitaeten des Szenarios samt der beiden
/// Sales-Person-Identitaeten, die der 409-Test und die Kandidatenlisten
/// brauchen.
struct SeededPersons {
    linked: Uuid,
    linked_sales_person: Uuid,
    user_only: Uuid,
    sales_only: Uuid,
    sales_only_sales_person: Uuid,
}

/// Legt die drei Auspraegungen per direktem `sqlx`-Insert an.
///
/// Direkte Inserts statt der neuen Service-Schreibpfade: die Tests sollen den
/// Lesepfad und die Mutationen gegen einen **unabhaengig** hergestellten
/// Zustand pruefen. Wuerde der Aufbau ueber `POST /person` laufen, koennte ein
/// gemeinsamer Fehler in Anlage und Lesung sich gegenseitig verdecken. Muster
/// uebernommen aus `integration_test/person_directory.rs`.
///
/// Laeuft **nach** `TestSetup::new()`, weil `TestSetup` intern
/// `BasicDaoImpl::clear_all()` aufruft und dabei alle `sales_person`-Zeilen
/// sowie jede traegerlos gewordene `person`-Zeile loescht.
async fn seed_detail_szenario(test_setup: &TestSetup) -> SeededPersons {
    let pool = test_setup.pool.as_ref();

    let linked = Uuid::new_v4();
    let user_only = Uuid::new_v4();
    let sales_only = Uuid::new_v4();
    let linked_sales_person = Uuid::new_v4();
    let sales_only_sales_person = Uuid::new_v4();

    for person_id in [linked, user_only, sales_only] {
        sqlx::query(
            "INSERT INTO person (id, created, update_process, update_version) \
             VALUES (?1, '2026-01-01T00:00:00.000000000Z', 'seed-63-06', ?2)",
        )
        .bind(person_id.as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed person");
    }

    for (sales_person_id, person_id, name) in [
        (linked_sales_person, linked, LINKED_DISPLAY_NAME),
        (
            sales_only_sales_person,
            sales_only,
            SALES_ONLY_DISPLAY_NAME,
        ),
    ] {
        sqlx::query(
            "INSERT INTO sales_person \
             (id, name, background_color, is_paid, inactive, deleted, person_id, \
              update_process, update_version) \
             VALUES (?1, ?2, '#FFF', true, false, NULL, ?3, 'seed-63-06', ?4)",
        )
        .bind(sales_person_id.as_bytes().to_vec())
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .bind(Uuid::new_v4().as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed sales_person");
    }

    for (person_id, name) in [(linked, LINKED_USERNAME), (user_only, USER_ONLY_NAME)] {
        sqlx::query(
            "INSERT INTO user (name, person_id, update_process) VALUES (?1, ?2, 'seed-63-06')",
        )
        .bind(name)
        .bind(person_id.as_bytes().to_vec())
        .execute(pool)
        .await
        .expect("seed user");
    }

    SeededPersons {
        linked,
        linked_sales_person,
        user_only,
        sales_only,
        sales_only_sales_person,
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
            "seed-63-06",
            tx.clone(),
        )
        .await
        .expect("create hr user");
    transaction_dao.commit(tx).await.expect("commit hr user");
    permission_dao
        .add_user_role(username, "hr", "seed-63-06")
        .await
        .expect("bind hr role");
}

/// Baut den Router genau so, wie ihn `rest::start_server` nestet — beide
/// Praefixe nebeneinander, damit derselbe Helfer sowohl die neuen
/// `/person`-Operationen als auch den unveraenderten Phase-61-Endpoint
/// erreicht — und schickt die Anfrage im Namen von `username`.
async fn request_as(
    test_setup: &TestSetup,
    username: &str,
    method: &str,
    uri: &str,
    body: Option<String>,
) -> (StatusCode, Vec<u8>) {
    let router = axum::Router::new()
        .nest(PERSON_PATH, generate_route::<crate::RestStateImpl>())
        .nest(
            SALES_PERSON_PATH,
            rest::sales_person::generate_route::<crate::RestStateImpl>(),
        )
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(Some(Arc::<str>::from(username)) as RestContext));

    let builder = Request::builder().method(method).uri(uri);
    let req = match body {
        Some(json) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(json))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };

    let resp = router.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
    (status, body)
}

/// Liest die Detailzeile einer Person als Admin und erwartet 200.
async fn detail_of(test_setup: &TestSetup, person_id: Uuid) -> PersonEntryTO {
    let (status, body) = request_as(
        test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/{person_id}"),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET {PERSON_PATH}/{person_id} muss 200 liefern; body={}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).unwrap_or_else(|e| {
        panic!(
            "Antwort muss nach PersonEntryTO deserialisieren: {e}; body={}",
            String::from_utf8_lossy(&body)
        )
    })
}

/// PADM-05 / PADM-06 (positiv): Der Admin bekommt zu jeder der drei
/// Auspraegungen 200 und eine fertig klassifizierte Zeile.
#[tokio::test]
async fn admin_bekommt_detail_200() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;

    let linked = detail_of(&test_setup, seeded.linked).await;
    assert_eq!(linked.kind, PersonKindTO::Linked);
    assert_eq!(linked.display_name.as_ref(), LINKED_DISPLAY_NAME);
    assert_eq!(linked.username.as_deref(), Some(LINKED_USERNAME));
    assert_eq!(linked.sales_person_id, Some(seeded.linked_sales_person));

    let user_only = detail_of(&test_setup, seeded.user_only).await;
    assert_eq!(user_only.kind, PersonKindTO::UserOnly);
    assert_eq!(user_only.display_name.as_ref(), USER_ONLY_NAME);
    assert_eq!(user_only.username.as_deref(), Some(USER_ONLY_NAME));
    assert!(
        user_only.sales_person_id.is_none(),
        "Ein reines Benutzerkonto hat keine Sales Person: {user_only:?}"
    );

    let sales_only = detail_of(&test_setup, seeded.sales_only).await;
    assert_eq!(sales_only.kind, PersonKindTO::SalesPersonOnly);
    assert_eq!(sales_only.display_name.as_ref(), SALES_ONLY_DISPLAY_NAME);
    assert!(
        sales_only.username.is_none(),
        "Ein reiner Verkaeufer hat kein Benutzerkonto: {sales_only:?}"
    );
    assert_eq!(
        sales_only.sales_person_id,
        Some(seeded.sales_only_sales_person)
    );

    // Der Rueckweg ueber die `sales_person_id` liefert dieselbe Zeile (PADM-04,
    // SC-4) — er ist die einzige Moeglichkeit, nach dem Verknuepfen eines
    // bestehenden Verkaeufers auf die Detailansicht zurueckzufinden.
    let (status, body) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/by-sales-person/{}", seeded.linked_sales_person),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let by_sales_person: PersonEntryTO = serde_json::from_slice(&body).unwrap();
    assert_eq!(by_sales_person, linked);
}

/// PADM-05 (D-62-04): Eine unbekannte `person_id` ist 404 — nie eine halbleere
/// Antwort, die von einem echten Eintrag ununterscheidbar waere.
#[tokio::test]
async fn unbekannte_person_id_ist_404() {
    let test_setup = TestSetup::new().await;
    seed_detail_szenario(&test_setup).await;

    let (status, _) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/{}", Uuid::new_v4()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Gegenstueck auf dem zweiten Lesepfad: eine nicht aufloesbare Sales Person
    // ist ebenfalls 404 (`EntityNotFoundGeneric`).
    let (status, _) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/by-sales-person/{}", Uuid::new_v4()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// PADM-08 (negativ, D-62-08, T-63-06-A/T-63-06-B): Ein HR-Anwender ohne
/// Admin-Rolle bekommt auf **jeder** der sieben Operationen 403 — nicht auf
/// sechs von sieben.
#[tokio::test]
async fn hr_ohne_admin_bekommt_403_auf_jeder_route() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;
    seed_hr_only_user(&test_setup, "hilde-hr").await;

    let create_body = serde_json::to_string(&CreatePersonEntryTO {
        display_name: "Eva Eindringling".into(),
        wants_user: true,
        wants_sales_person: true,
        username: Some("eva-eindringling".into()),
    })
    .unwrap();
    let facet_body = serde_json::to_string(&AddPersonFacetTO {
        facet: PersonFacetTO::User,
        display_name: None,
        username: Some("eva-eindringling".into()),
    })
    .unwrap();

    let operationen: [(&str, String, Option<String>); 7] = [
        (
            "GET",
            format!("{PERSON_PATH}/candidates/sales-persons"),
            None,
        ),
        ("GET", format!("{PERSON_PATH}/candidates/users"), None),
        (
            "GET",
            format!("{PERSON_PATH}/by-sales-person/{}", seeded.linked_sales_person),
            None,
        ),
        ("POST", PERSON_PATH.to_string(), Some(create_body)),
        ("GET", format!("{PERSON_PATH}/{}", seeded.linked), None),
        (
            "POST",
            format!("{PERSON_PATH}/{}/facet", seeded.sales_only),
            Some(facet_body),
        ),
        (
            "DELETE",
            format!("{PERSON_PATH}/{}/facet/user", seeded.user_only),
            None,
        ),
    ];

    for (method, uri, body) in operationen {
        let (status, response_body) =
            request_as(&test_setup, "hilde-hr", method, &uri, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{method} {uri} muss fuer einen reinen HR-Anwender 403 liefern; body={}",
            String::from_utf8_lossy(&response_body)
        );

        let body_str = String::from_utf8_lossy(&response_body).to_string();
        for name in [
            LINKED_DISPLAY_NAME,
            LINKED_USERNAME,
            USER_ONLY_NAME,
            SALES_ONLY_DISPLAY_NAME,
        ] {
            assert!(
                !body_str.contains(name),
                "Die 403-Antwort von {method} {uri} darf keinen geseedeten Namen preisgeben ({name}): {body_str}"
            );
        }
    }

    // Gegenprobe: der Zustand ist unveraendert — kein Schreibvorgang ist
    // durchgerutscht. `eva-eindringling` darf weder als Benutzer noch als
    // Sales Person existieren.
    let (_, body) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/candidates/users"),
        None,
    )
    .await;
    let users: Vec<UserTO> = serde_json::from_slice(&body).unwrap();
    assert!(
        !users.iter().any(|user| user.name == "eva-eindringling"),
        "Der HR-Versuch darf keinen Benutzer angelegt haben: {users:?}"
    );

    let sales_only_danach = detail_of(&test_setup, seeded.sales_only).await;
    assert_eq!(
        sales_only_danach.kind,
        PersonKindTO::SalesPersonOnly,
        "Das HR-Zuschalten darf die Auspraegung nicht veraendert haben"
    );
    let user_only_danach = detail_of(&test_setup, seeded.user_only).await;
    assert_eq!(
        user_only_danach.kind,
        PersonKindTO::UserOnly,
        "Das HR-Entfernen darf die Auspraegung nicht veraendert haben"
    );
}

/// PADM-02 / SC-2: Nach dem Speichern existieren genau die gewaehlten Objekte,
/// und beide haengen an derselben Person.
#[tokio::test]
async fn kombiniertes_anlegen_erzeugt_beide_objekte() {
    let test_setup = TestSetup::new().await;
    seed_detail_szenario(&test_setup).await;

    let body = serde_json::to_string(&CreatePersonEntryTO {
        display_name: "Dora Doppelt".into(),
        wants_user: true,
        wants_sales_person: true,
        username: Some("dora-doppelt".into()),
    })
    .unwrap();

    let (status, response_body) = request_as(
        &test_setup,
        "DEVUSER",
        "POST",
        PERSON_PATH,
        Some(body),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Das kombinierte Anlegen muss 201 liefern; body={}",
        String::from_utf8_lossy(&response_body)
    );

    let created: CreatedPersonEntryTO = serde_json::from_slice(&response_body).unwrap();
    let entry = detail_of(&test_setup, created.person_id).await;
    assert_eq!(entry.kind, PersonKindTO::Linked);
    assert_eq!(entry.display_name.as_ref(), "Dora Doppelt");
    assert_eq!(entry.username.as_deref(), Some("dora-doppelt"));
    assert!(
        entry.sales_person_id.is_some(),
        "Beide Seiten muessen an derselben Person haengen: {entry:?}"
    );
}

/// D-63-07 / ASVS V5: Die Regel „mindestens ein Umschalter muss an sein" ist
/// eine Server-Pruefung. Der deaktivierte Absende-Knopf im Frontend ist UX,
/// nicht die Grenze — dieser Request umgeht ihn und muss 422 bekommen.
#[tokio::test]
async fn anlegen_ohne_umschalter_ist_422() {
    let test_setup = TestSetup::new().await;

    let body = serde_json::to_string(&CreatePersonEntryTO {
        display_name: "Niemand Nirgends".into(),
        wants_user: false,
        wants_sales_person: false,
        username: None,
    })
    .unwrap();

    let (status, response_body) = request_as(
        &test_setup,
        "DEVUSER",
        "POST",
        PERSON_PATH,
        Some(body),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Ein leerer Auswahl-Satz muss 422 sein; body={}",
        String::from_utf8_lossy(&response_body)
    );
}

/// PADM-03 / SC-3: Auf einem Eintrag mit fehlender Auspraegung entsteht genau
/// das fehlende Objekt und ist verknuepft; das bereits vorhandene Objekt bleibt
/// unveraendert. Beim Zuschalten des **Benutzers** bleibt die `person_id`
/// unveraendert — es gibt nichts zusammenzufuehren.
#[tokio::test]
async fn facet_zuschalten_wechselt_die_auspraegung() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;

    let body = serde_json::to_string(&AddPersonFacetTO {
        facet: PersonFacetTO::User,
        display_name: None,
        username: Some("clara-login".into()),
    })
    .unwrap();

    let (status, response_body) = request_as(
        &test_setup,
        "DEVUSER",
        "POST",
        &format!("{PERSON_PATH}/{}/facet", seeded.sales_only),
        Some(body),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Das Zuschalten muss 200 liefern; body={}",
        String::from_utf8_lossy(&response_body)
    );

    let resulting: CreatedPersonEntryTO = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(
        resulting.person_id, seeded.sales_only,
        "Beim Zuschalten eines Benutzers bleibt die person_id unveraendert"
    );

    let entry = detail_of(&test_setup, seeded.sales_only).await;
    assert_eq!(entry.kind, PersonKindTO::Linked);
    assert_eq!(entry.username.as_deref(), Some("clara-login"));
    // Das vorhandene Objekt bleibt unveraendert: Anzeigename und
    // Sales-Person-Identitaet sind dieselben wie vorher.
    assert_eq!(entry.display_name.as_ref(), SALES_ONLY_DISPLAY_NAME);
    assert_eq!(entry.sales_person_id, Some(seeded.sales_only_sales_person));
}

/// PADM-07 / SC-5 und D-63-01: Verliert die Person durch das Entfernen ihren
/// letzten lebenden Traeger, verschwindet sie aus dem Verzeichnis — die
/// Detailansicht folgt derselben Sichtbarkeitsregel und antwortet danach 404.
#[tokio::test]
async fn facet_entfernen_raeumt_die_person_auf() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;

    let (status, body) = request_as(
        &test_setup,
        "DEVUSER",
        "DELETE",
        &format!("{PERSON_PATH}/{}/facet/user", seeded.user_only),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "Das Entfernen muss 204 liefern; body={}",
        String::from_utf8_lossy(&body)
    );

    let (status, _) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/{}", seeded.user_only),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "Die traegerlos gewordene Person muss aus der Detailansicht verschwinden"
    );

    // Die verknuepfte Person ist davon unberuehrt — es wird genau eine Seite
    // genau eines Eintrags entfernt, kein Sammelknopf (D-63-05).
    let linked = detail_of(&test_setup, seeded.linked).await;
    assert_eq!(linked.kind, PersonKindTO::Linked);
}

/// PADM-04 / SC-4: Die Kandidatenlisten blenden bereits verknuepfte Ziele aus.
/// Das ist die Bequemlichkeits-Seite des Vertrags; die Grenze prueft der Test
/// darunter.
#[tokio::test]
async fn kandidatenlisten_blenden_verknuepfte_ziele_aus() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;

    let (status, body) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/candidates/sales-persons"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let candidates: Vec<SalesPersonTO> = serde_json::from_slice(&body).unwrap();
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.id == seeded.sales_only_sales_person),
        "Der Verkaeufer ohne Login muss als Kandidat erscheinen: {candidates:?}"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.id == seeded.linked_sales_person),
        "Der bereits verknuepfte Verkaeufer darf nicht erscheinen: {candidates:?}"
    );

    let (status, body) = request_as(
        &test_setup,
        "DEVUSER",
        "GET",
        &format!("{PERSON_PATH}/candidates/users"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let candidates: Vec<UserTO> = serde_json::from_slice(&body).unwrap();
    assert!(
        candidates.iter().any(|user| user.name == USER_ONLY_NAME),
        "Der Benutzer ohne Verkaeufer muss als Kandidat erscheinen: {candidates:?}"
    );
    assert!(
        !candidates.iter().any(|user| user.name == LINKED_USERNAME),
        "Der bereits verknuepfte Benutzer darf nicht erscheinen: {candidates:?}"
    );
}

/// Success Criterion 4 / D-61-04 (T-63-06-C): Ein bereits verknuepftes Ziel
/// wird abgelehnt — **obwohl** es in der Kandidatenliste gar nicht erscheint.
///
/// Der Test faehrt bewusst gegen den UNVERAENDERTEN Phase-61-Endpoint
/// `POST /sales-person/{id}/user`. Er belegt damit, dass die Ausblendung in der
/// Kandidatenliste die serverseitige Ablehnung nicht **ersetzt**, sondern
/// ergaenzt: wer die Liste umgeht oder zwischen Laden und Absenden verliert,
/// laeuft in den 409 und nicht in eine stille Ueberschreibung.
#[tokio::test]
async fn bereits_verknuepftes_ziel_wird_mit_409_abgelehnt() {
    let test_setup = TestSetup::new().await;
    let seeded = seed_detail_szenario(&test_setup).await;

    let (status, body) = request_as(
        &test_setup,
        "DEVUSER",
        "POST",
        &format!(
            "{SALES_PERSON_PATH}/{}/user",
            seeded.sales_only_sales_person
        ),
        Some(serde_json::to_string(LINKED_USERNAME).unwrap()),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "Ein bereits verknuepfter Benutzer darf nicht an einen zweiten \
         Verkaeufer gehaengt werden; body={}",
        String::from_utf8_lossy(&body)
    );

    // Und der Zustand ist unveraendert geblieben — kein Teil-Schreibvorgang.
    let linked = detail_of(&test_setup, seeded.linked).await;
    assert_eq!(linked.username.as_deref(), Some(LINKED_USERNAME));
    assert_eq!(linked.sales_person_id, Some(seeded.linked_sales_person));

    let sales_only = detail_of(&test_setup, seeded.sales_only).await;
    assert_eq!(sales_only.kind, PersonKindTO::SalesPersonOnly);
    assert!(sales_only.username.is_none());
}
