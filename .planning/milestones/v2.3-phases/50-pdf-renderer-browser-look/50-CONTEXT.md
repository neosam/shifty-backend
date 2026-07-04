# Phase 50: pdf-renderer-browser-look - Context

**Gathered:** 2026-07-03
**Status:** Ready for planning
**Mode:** discuss (Textform, 6 Gray Areas: G1-c / G2-a / G3-a / G4-Sonntag-dyn + Header/Timestamp Claude's Discretion / G5-Boxen / G6-a + Rückfrage)

<domain>
## Phase Boundary

Kompletter Rewrite von `service_impl/src/pdf_render.rs`. Der neue Renderer produziert
PDFs, die visuell der Browser-`WeekView` entsprechen: **Slots als eigene sichtbare Boxen
innerhalb der Tages-Spalte**, jede Box zeigt Zeit-Label „HH:MM – HH:MM" + die gebuchten
Sales-Person-Namen als Plain-Text-Liste. Landscape A4. Header „Schichtplan KW NN (JJJJ)".
Renderzeitpunkt „Erstellt am DD.MM.YYYY HH:MM Uhr" auf jeder Seite. Bewusst wird der
Byte-Determinismus-Vertrag aus v2.2 aufgehoben (Timestamp bricht ihn ohnehin).

**Kein Snapshot-Bump, keine Migration, keine neue Cargo-Dep.** WebDAV-Scheduler und der
On-Demand-Download-Button aus Phase 49 nutzen den neuen Renderer automatisch, weil beide
über `PdfShiftplanService::render_week_pdf` (D-49-05..09) gehen.

</domain>

<spec_lock>
## Locked Requirements (SPEC.md-Äquivalent aus REQUIREMENTS.md v2.3)

**MUST READ vor Planning:** `.planning/REQUIREMENTS.md` §"PDF-01" + §"PDF-02".

- **PDF-01 (Layout):** Landscape A4, sieben Wochentag-Spalten (Mo–So), Slots als
  eigene Zellen/Kästen innerhalb einer Spalte sortiert nach Startzeit, mit
  Uhrzeit-Label pro Zelle im Format „08:00 – 12:00", Sales-Person-Namen in der
  Slot-Zelle, Kopfzeile „Schichtplan KW {NN} ({JJJJ})". Erfolgskriterium:
  „Ausdruck ohne Digital-Referenz nutzbar".
- **PDF-02 (Timestamp):** „Erstellt am DD.MM.YYYY HH:MM Uhr" auf jeder Seite,
  lokale Zeit des Backend-Servers, Renderer nimmt Timestamp als Argument
  (Pure-Funktion bleibt testbar).
- **Nicht-Ziel:** i18n der PDF-Texte (bleibt Deutsch), Byte-Determinismus
  (bewusst aufgehoben), neue Cargo-Deps.

**ROADMAP-Success-Criteria (Phase 50):**
1. Rendering entspricht sichtbar der Browser-Wochenansicht (siehe PDF-01).
2. Renderzeitpunkt sichtbar auf jeder Seite; Renderer nimmt Timestamp als Argument.
3. Byte-Determinismus-Vertrag aus v2.2 aufgehoben, WebDAV-Scheduler nutzt Renderer
   transparent, alle Backend-Tests + `cargo clippy --workspace -- -D warnings` grün.

</spec_lock>

<decisions>
## Implementation Decisions

### G1 — Slot-Layout innerhalb der Tages-Spalte

- **D-50-01 (G1-c: Hybrid Stack mit Dauer-Skalierung):** Slots werden pro Tages-Spalte
  vertikal gestapelt in Reihenfolge der Slot-Startzeit. Jede Slot-Box hat eine Höhe,
  die grob mit ihrer Dauer skaliert (z.B. `cell_height_mm = base_mm + duration_hours *
  step_mm`, konkrete Konstanten legt der Planner fest). Kein echtes Time-Grid, keine
  linke Zeitachsen-Spalte, keine absoluten Positions-Berechnungen. Sub-Column-Logik
  für überlappende Slots ist explizit nicht nötig — Slots werden einfach
  untereinander gestapelt, auch wenn ihre Zeitfenster sich überschneiden.
- **D-50-02:** Sortierung: primär `slot.from` (Aufsteigend), sekundär `slot.to`
  (Aufsteigend) als deterministischer Tie-Breaker. Terziär `slot.id` als
  Falls-alle-Zeiten-gleich-Fallback.

### G2 — Multi-Page-Handling

- **D-50-03 (G2-a: Best-Effort 1 Seite, Overflow abschneiden):** Es wird immer genau
  **eine A4-Landscape-Seite pro Woche** gerendert. Wenn die Slots pro Tages-Spalte
  nicht in die verfügbare Höhe passen, wird die betroffene Spalte visuell
  abgeschnitten und ein „+ N weitere"-Suffix am unteren Rand der letzten
  darstellbaren Slot-Box eingesetzt (analog dem Browser-Verhalten mit
  overflow-hidden). Kein Auto-Shrink der Font-Size, kein automatisches Umbrechen
  auf Seite 2. **Trotzdem ist der Renderer multi-page-fähig** — der Header
  „Erstellt am …" wird bereits so implementiert, dass er auf **jeder** Seite
  gerendert würde, falls das Layout in Zukunft doch multi-page geht (PDF-02
  fordert das explizit als Vertrag, auch wenn Phase 50 aktuell 1-Seite bleibt).
- **D-50-04 (Namen-Overflow innerhalb einer Slot-Box):** Analog — passen die
  Sales-Person-Namen nicht in die Box-Höhe, wird abgeschnitten mit „+ N weitere"
  in der letzten sichtbaren Namen-Zeile. Kein Wrap in mehrere Boxen.

### G3 — Namen-Rendering in der Slot-Zelle

- **D-50-05 (G3-a: Plain-Text-Liste, ein Name pro Zeile):** Innerhalb der Slot-Box
  werden die gebuchten Sales-Persons als reine Text-Zeilen untereinander
  gerendert. Kein Chip-Rahmen, keine Hintergrundfarbe, kein `background_color`
  aus `SalesPerson`. Font-Size analog der aktuellen Zellen-Konstante (~10pt).
- **D-50-06 (Sortierung der Namen innerhalb einer Slot-Box):** Namen werden
  alphabetisch nach `sales_person.name` (case-insensitive) sortiert.
  Deterministisch, verhindert dass Booking-Insertion-Reihenfolge die Ausgabe
  beeinflusst.
- **D-50-07 (Unpaid-Marker):** Freiwillige (`sales_person.is_paid == Some(false)`)
  bekommen ein Suffix „ (freiwillig)" hinter ihrem Namen. Grund: In Plain-Text
  ohne Chip-Farbe geht die visuelle Unterscheidung Angestellte/Freiwillige aus
  dem Browser sonst verloren. Trivial rückgängig zu machen falls User später
  „Marker weglassen" bevorzugt. Verifikation im Backend-Test mit gemischtem
  Fixture (paid + unpaid Sales-Person auf demselben Slot).

### G4 — Sonntag-Spalte, Header/Footer-Layout

- **D-50-08 (Dynamische Wochentag-Spalten):** Analog zur Browser-`WeekView`-Logik
  (`week_view.rs` §1199–1229): wenn kein Slot der Woche auf Sonntag liegt, wird
  die Sonntag-Spalte **nicht gerendert** — es werden nur 6 Spalten (Mo–Sa)
  gezeichnet. Wenn mindestens ein Slot auf Sonntag liegt, werden alle 7 Spalten
  gerendert. **Konsequenz:** Die verfügbare Spaltenbreite hängt von der
  Wochenzusammensetzung ab; das ist explizit gewollt (Konsistenz mit dem
  Browser-Look).
- **D-50-09 (Header + Timestamp — Claude's Discretion):**
  Konkreter Vorschlag für den Planner (kann bei UX-Bedarf verändert werden):
  - Titel „Schichtplan KW {NN} ({JJJJ})" **oben links**, Bold, Header-Font.
  - Timestamp „Erstellt am DD.MM.YYYY HH:MM Uhr" **oben rechts**, kleinere
    Font-Size (~9pt), normal-Font. **Auf derselben oberen Header-Zeile** wie
    der Titel, um vertikalen Platz für das Grid zu maximieren. Wenn Phase 50
    doch multi-page geht (Zukunft), erscheint diese Header-Zeile auf jeder
    Seite.
  - Kein Footer, keine Seitennummer (Phase 50 rendert 1 Seite; Seitennummerierung
    Zukunfts-Feature falls multi-page erweitert wird).

### G5 — Slot-Rahmen (sichtbare Boxen)

- **D-50-10 (Sichtbare Slot-Boxen):** Jede Slot-Box wird mit einer sichtbaren
  rechteckigen Umrandung gerendert (`printpdf::Op::DrawLine` oder
  `Op::DrawPolygon` mit outline-only style, konkrete API-Wahl vom Planner). Line
  weight ~0.3–0.5pt, Farbe Schwarz. Keine Hintergrund-Füllung (weiß bleibt weiß).
  Das gilt sowohl für die Slot-Boxen innerhalb der Tages-Spalten als auch für die
  Tages-Spalten-Rahmen (analog zum Grid-Look im Browser). Header-Bereich braucht
  keinen Rahmen.

### G6 — Renderer-Signatur mit Timestamp-Parameter

- **D-50-11 (G6-a: Renderer nimmt bereits-konvertierten `OffsetDateTime`):** Neue
  Renderer-Signatur:
  ```rust
  pub fn render_shiftplan_week_pdf(
      week: &ShiftplanWeek,
      sales_persons: &[SalesPerson],
      header_year: u32,
      header_week: u8,
      render_timestamp: time::OffsetDateTime,
  ) -> Result<Vec<u8>, ServiceError>
  ```
  Der Timestamp ist bereits vom Aufrufer in „lokale Zeit des Backend-Servers"
  konvertiert (PDF-02). Der Renderer formatiert ihn schlicht mit
  `format!("{:02}.{:02}.{} {:02}:{:02}", …)`. Keine TZ-Konvertierung im
  Renderer, keine neue Cargo-Dep.
- **D-50-12 (Aufrufer-Verantwortung):** Alle Aufrufer beschaffen den Timestamp
  via `time::OffsetDateTime::now_local()` (nutzt OS-TZ des Backend-Servers).
  Konkret betroffen:
  - `PdfShiftplanService::render_week_pdf` (aus Phase 49, D-49-05..09) — muss um
    `now_local()`-Aufruf ergänzt werden vor `pdf_render::render_shiftplan_week_pdf`.
  - Der Renderer wird nur noch durch `PdfShiftplanService` konsumiert
    (D-49-08 hat den Scheduler auf denselben Service refactored). Kein
    Direktaufruf aus dem Scheduler mehr → **eine einzige Injection-Stelle**.
  - Fallback: Wenn `now_local()` fehlschlägt (auf manchen Multi-Thread-Setups
    unsound → `IndeterminateOffset`-Error von `time` crate), fällt der Aufrufer
    auf `now_utc()` zurück und logged eine Warnung. Der PDF-Text wäre dann UTC
    statt Local — akzeptable Graceful-Degradation. Klarer nicht-panic-Pfad.
- **D-50-13 (Byte-Determinismus aufgehoben):** Da der Timestamp jede Sekunde
  variiert, ist der byte-deterministische-Vertrag aus v2.2 (D-48-PDF-DETERMINISM)
  **explizit aufgehoben**. `FIXED_METADATA_TIMESTAMP` bleibt drin für die PDF-
  Metadata-Felder (`CreationDate`, `ModDate`, `Producer`) — das nur die
  Trailer-Bytes stabilisiert, nicht den sichtbaren Timestamp im Rendering.
  Die bestehenden `deterministic_bytes_for_same_input` + `sales_persons_sorted_by_id`
  Tests (`pdf_render.rs` §430–493) müssen dementsprechend angepasst werden:
  entweder normalisiert um den Timestamp-Bereich, oder Fixed-Timestamp injizieren
  (bevorzugt, siehe D-50-14). Der v2.2-Determinismus-Guardrail-Test wird
  **entfernt**, weil er semantisch obsolet ist.

### Tests

- **D-50-14 (Renderer-Test-Strategie):** Fixed-Timestamp-Fixture wird als
  Test-Konstante definiert (z.B. `time::macros::datetime!(2026-07-03 17:15 UTC)`).
  Alle Renderer-Unit-Tests injizieren diese Konstante. So bleiben Tests
  deterministisch UND testbar für „Timestamp im Textstream vorhanden".
- **D-50-15 (Zu portierende Renderer-Tests aus v2.2):**
  - `empty_week_yields_valid_pdf_signature` → bleibt (jetzt mit Fixed-Timestamp-
    Argument).
  - `header_contains_year_and_week` → bleibt, jetzt mit „Schichtplan KW 27 (2026)".
  - `all_active_sales_persons_appear` → bleibt, muss aber prüfen, dass die
    Namen jetzt **in den Slot-Boxen** (nicht mehr in Namen-Spalte links) im
    Textstream vorkommen. Hex-Encoding-Suche bleibt derselbe Ansatz.
  - `deterministic_bytes_for_same_input` → **entfällt** (Determinismus aufgehoben).
  - `sales_persons_sorted_by_id` → **entfällt** (Sortier-Logik ist jetzt in D-50-06:
    Namen alphabetisch sortiert **innerhalb einer Slot-Box**, nicht global). Ersetzt
    durch neuen Test `names_within_slot_alphabetical`.
  - `build_page_header_produces_expected_text` → bleibt.
  - `build_day_column_headers_yields_seven_short_labels` → bleibt.
  - `build_sales_person_row_lists_bookings_time_ranges` → **entfällt** (Row-Layout
    weg). Ersetzt durch `build_slot_cell_contents_lists_names_and_time_range`.
  - `normalize_pdf_id_removes_variable_id_array`, `find_all_subsequences_locates_multiple_occurrences`
    → bleiben (Test-Helper).
- **D-50-16 (Neue Renderer-Tests, mindestens):**
  - `render_includes_timestamp_string` — mit Fixed-Timestamp verifiziert, dass
    „Erstellt am 03.07.2026 17:15 Uhr" im Textstream steht (hex-encoded).
  - `slot_boxes_sorted_by_start_time` — zwei Slots am selben Tag (12:00 und
    08:00 Startzeit); prüft, dass „08:00" **vor** „12:00" im Textstream steht.
  - `names_within_slot_alphabetical` — drei Bookings auf demselben Slot
    (Charlie/Alice/Bob); prüft alphabetische Reihenfolge im Textstream.
  - `unpaid_marker_suffix` — Sales-Person mit `is_paid=Some(false)` bekommt
    „ (freiwillig)"-Suffix.
  - `sunday_column_hidden_when_no_sunday_slots` — Woche ohne Sonntag-Slots:
    Prüft, dass „So" im Textstream **nicht** vorkommt (bzw. dass die 7. Spalte
    nicht gerendert wird — konkrete Assertion vom Planner).
  - `sunday_column_shown_when_at_least_one_sunday_slot` — Gegen-Test.
  - `now_local_fallback_to_utc_on_indeterminate_offset` — Service-Level-Test
    (nicht Renderer): mock `IndeterminateOffset`, prüft dass `now_utc()`
    genutzt wird + Warning geloggt.
- **D-50-17 (Integrations-/UAT-Verifikation):** Zusätzlich zum Backend-Test
  liefert der Phase-49-Button ein echtes PDF gegen ein reales Wochen-Fixture.
  Vor dem Milestone-Close klickt der User den Button und prüft visuell:
  Layout entspricht Browser-Look, Timestamp vorhanden, Slots als Boxen sichtbar,
  Namen darin, Sonntag korrekt gedynt.

### Claude's Discretion (Planner darf ohne Rückfrage entscheiden)

- Konkrete `printpdf`-API-Wahl für Line-Draws (D-50-10): `Op::DrawLine` vs.
  `Op::DrawPolygon` vs. `Op::AddLineToPath`. Whatever printpdf 0.7 sauber
  unterstützt.
- Exakte Maß-Konstanten: Slot-Box-Base-Height, Duration-Step, Header-Height,
  Padding, Font-Sizes. Aktuelle Konstanten (`pdf_render.rs:52–77`) als
  Ausgangspunkt.
- Struktur des Renderer-Codes: Pure-Fn-Zerlegung (Layout-Berechnung,
  Zellen-Rendering, Header-Rendering als separate `fn`s), Refactor-freundlich.
- Ob `pdf_render.rs` in Submodule zerlegt wird (`layout.rs`, `text.rs`) oder
  in einer Datei bleibt.
- ROADMAP-Update: nach Phase-50-Verifikation wird die Phase-50-Checkbox in
  `.planning/ROADMAP.md` gesetzt (STATE.md + Progress-Tabelle).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` §"PDF-01" + §"PDF-02" — Layout- und
  Timestamp-Requirements (**LOCKED**).
- `.planning/ROADMAP.md` §"Phase 50" — Goal + Success Criteria.

### Phase 49 (Vorgänger — Service, der den Renderer aufruft)
- `.planning/phases/49-pdf-download-button/49-CONTEXT.md` — Insb. D-49-05..09
  (`PdfShiftplanService::render_week_pdf`); D-49-08 (Scheduler-Refactor).
  Phase 50 fügt Timestamp-Injection in diesen Service-Aufrufpfad ein.
- `service_impl/src/pdf_shiftplan.rs` (aus Phase 49) — hier wird die
  `now_local()`-Zeile ergänzt (D-50-12).
- `service_impl/src/pdf_export_scheduler.rs` — Scheduler-Refactor aus 49-03 hat
  bereits vollständig auf `PdfShiftplanService` umgestellt; kein direkter
  Renderer-Aufruf mehr. Muss nur validiert werden, dass es so bleibt.

### Renderer & Test-Referenz (v2.2)
- `service_impl/src/pdf_render.rs` — **wird komplett neu geschrieben**. Aktuelle
  Datei als API-Referenz (Konstanten, Helpers, Test-Helpers) — was übernommen
  wird siehe D-50-15.
- `.planning/milestones/v2.2-phases/48-nextcloud-pdf-webdav/48-CONTEXT.md`
  §"D-48-PDF" — Ursprungs-Design-Entscheidungen für den Renderer, aufgehoben
  bzgl. Byte-Determinismus (D-48-PDF-DETERMINISM → obsolet, D-50-13).

### Browser-Referenz (visueller Look)
- `shifty-dioxus/src/component/week_view.rs` — die visuelle Vorlage, die
  reproduziert werden soll. Insb. §1189–1375 (`WeekView` Body: 7 Day-Columns,
  time-column, Slot-Positionierung).
- `shifty-dioxus/src/component/week_view.rs` §1199–1229 — Sonntag-Ausblendungs-
  Logik (`has_sunday`), Blaupause für D-50-08.
- `shifty-dioxus/src/component/week_view.rs` §1435–1470 — SSR-Test der
  `WeekCellSlot`-Chip-Anordnung (nur als Referenz für Slot-Inhalts-Struktur;
  PDF nutzt Plain-Text statt Chips).

### Backend-Konventionen
- `CLAUDE.md` (Backend-Root) §"Testing" — Renderer-Tests bleiben Unit-Tests im
  gleichen Modul (`#[cfg(test)] mod test`).
- `CLAUDE.md` (Backend-Root) §"Clippy is a hard gate" — `cargo clippy --workspace
  -- -D warnings` MUSS grün sein.

### Config / Timezone (nicht direkt Renderer, aber Kontext für D-50-12)
- `service/src/config.rs` + `service_impl/src/config.rs` — `TIMEZONE`-ENV,
  Default UTC. **Wird von Phase 50 NICHT direkt konsumiert** — `now_local()`
  nutzt OS-TZ, nicht die ENV. Deployment-Voraussetzung: OS-TZ und `TIMEZONE`-ENV
  sind auf demselben Wert konfiguriert (Standard auf NixOS-Deploys).
- `service_impl/src/ical.rs` — iCal-Muster zur Info; Timezone dort ist TZID-
  Attribut, nicht Konvertierung.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Datentypen unverändert:** `ShiftplanWeek`, `ShiftplanDay`, `ShiftplanSlot`,
  `ShiftplanBooking`, `SalesPerson`, `Slot`, `DayOfWeek` — alle bleiben genau
  wie sie sind. Renderer konsumiert dieselbe Struktur wie v2.2.
- **`printpdf` 0.7:** bereits im Backend-Root-`Cargo.toml`, keine neue Dep. APIs:
  `PdfDocument::new`, `add_builtin_font(BuiltinFont::Helvetica|HelveticaBold)`,
  `use_text(text, size, Mm(x), Mm(y), &font)`, `save_to_bytes`, Metadata-Setter.
- **Konstanten-Set (Ausgangspunkt):** `pdf_render.rs:52–77` — `PAGE_WIDTH_MM`,
  `PAGE_HEIGHT_MM`, `HEADER_Y_MM` etc. Werden für neues Layout überarbeitet,
  aber die A4-Landscape-Konstanten bleiben.
- **`day_of_week_order()` + `build_day_column_headers()`:** `pdf_render.rs:178–193`
  — Mo/Di/Mi/Do/Fr/Sa/So-Labels + `DayOfWeek`-Enum-Ordering, direkt
  wiederverwendbar. Bei dynamischen Wochenspalten (D-50-08) muss `slots_by_day`
  vorab die Sonntag-Präsenz prüfen (analog `week_view.rs` `has_sunday`).
- **`normalize_pdf_id` + `find_subsequence` + `encode_ascii_to_pdf_hex`:**
  `pdf_render.rs` §309–383 — Test-Helper. `encode_ascii_to_pdf_hex` bleibt
  wichtig für Textstream-Assertions. `normalize_pdf_id` entfällt (Determinismus
  aufgehoben).

### Established Patterns
- **Renderer bleibt pure Modul:** kein I/O, keine DAO/Service-Aufrufe (D-48-PDF
  bleibt gültig). Timestamp kommt von außen als Argument.
- **Bookings-Filter:** Caller (Phase-49-`PdfShiftplanService`) hat bereits den
  Filter `sales_persons.filter(deleted.is_none())` etabliert (D-49-05,
  D-48-PDF-ACTIVE-ONLY). Renderer prüft nicht selbst.
- **Test-Struktur:** `#[cfg(test)] mod test` mit Fixture-Buildern
  (`make_slot`, `make_booking`, `make_sales_person`) — bleibt und wird ergänzt.
- **Encoding im Textstream:** `printpdf`-`use_text` schreibt Text als uppercase-
  Hex-Sequenz in den Content-Stream. Alle Text-Assertions müssen die
  Hex-Encoding-Wandlung nutzen (`encode_ascii_to_pdf_hex`).

### Integration Points
- **Renderer-Datei:** `service_impl/src/pdf_render.rs` — kompletter Rewrite
  (Signatur + Body + Tests).
- **`PdfShiftplanService::render_week_pdf`:** `service_impl/src/pdf_shiftplan.rs`
  — muss um `now_local()`-Aufruf + Übergabe an den Renderer erweitert werden.
  Neue Zeile ~1: TZ-Abholung; ~2: Renderer-Call mit erweitertem Argument.
- **`PdfExportScheduler`:** Wenn er nach Phase-49-Refactor noch direkte
  Renderer-Aufrufe hätte, muss er entfernt werden. Erwartung: geht bereits
  über `PdfShiftplanService` (D-49-08). Planner sanity-checkt.
- **Backend-Integrationstests** (z.B. `rest/tests/pdf_shiftplan.rs` aus Phase
  49): der Handler-Test prüft nur Status/Header — Content-Bytes werden nur
  smoke-geprüft (Magic-Bytes + Länge). Timestamp-Test läuft auf Renderer-
  Unit-Level.

### Anti-Patterns / Landmines
- **`OffsetDateTime::now_local()` kann `IndeterminateOffset` liefern** — auf
  Multi-Thread-Deploys ohne `unsafe { set_local_offset }` liefert das `time` crate
  einen Error statt der aktuellen lokalen TZ. D-50-12 sagt: `unwrap_or_else(|_|
  now_utc())` + Warning. **KEIN `expect()`** oder `unwrap()` auf `now_local()`.
- **Fixed-Timestamp im Test:** `time::macros::datetime!` liefert `OffsetDateTime`
  mit UTC. Wenn der Renderer-Format-Code auf lokaler Zeit basiert, muss der
  Test die konvertierte Fixed-Zeit prüfen — nicht die UTC-Rohzeit. Planner:
  einfachster Weg = Renderer nimmt den `OffsetDateTime` und formatiert ihn
  einfach direkt via `.hour()`/`.minute()`/`.day()`/`.month()`/`.year()` —
  ohne implizite Konvertierung. Der Test-Wert ist damit die Fixture-Zahl.
- **Determinismus-Test-Entfernung:** Wenn ein Verifier oder ein anderer Nutzer
  außerhalb des Renderer-Moduls den `deterministic_bytes_for_same_input`-Test
  konsumiert, würde das brechen. Search: `grep -rn deterministic_bytes .` —
  Erwartung: kein externer Konsument, Test ist rein modul-intern. Planner
  bestätigt vor Löschung.
- **Sonntag-Ausblendung ändert die Spaltenbreite:** 6 vs. 7 Spalten ändert
  `DAY_COL_WIDTH_MM`-Berechnung. Muss dynamisch berechnet werden, nicht
  hart-kodiert. Auswirkung: alle Slot-Box-X-Positionen sind eine Funktion
  von `n_days` + `col_width`.
- **`+ N weitere"-Overflow-Indicator (D-50-03/04):** braucht Höhen-/Zeilen-
  Buchhaltung im Renderer — nicht bloß absolute Positionen. Layout-Berechnung
  wird komplexer als der v2.2-Renderer (der überhaupt keinen Overflow-Fall
  hatte). Planner budgetiert diese Komplexität explizit ein.

</code_context>

<specifics>
## Specific Ideas

- **Font-Auswahl:** bleibt Helvetica (Header bold) — v2.2-Wahl war korrekt.
  Kein Grund für Font-Wechsel.
- **Farben:** Alles Schwarz-auf-Weiß. Slot-Rahmen 0.3–0.5pt Line, kein Fill.
  Kein Grün/Rot/Farbe für „Freiwillige" oder „paid overage" — Plain-Text-Modus.
- **„08:00 – 12:00"-Format** mit En-Dash oder Hyphen? Requirement schreibt es
  mit einfachem Hyphen; Planner nutzt Hyphen (`08:00 - 12:00`) für ASCII-
  Kompatibilität in der PDF-Encoding-Kette.
- **Verifikation im Milestone-Close:** User klickt Phase-49-Download-Button,
  öffnet PDF, checkt visuell: Boxen, Namen, Zeiten, Timestamp, Sonntag-Dyn.
  Kein automatisierbarer Screenshot-Test (headless-Chrome für PDF-Rendering
  wäre Overkill für v2.3).

</specifics>

<deferred>
## Deferred Ideas

- **Multi-Page-Rendering** — falls große Wochen 1 Seite sprengen. Aktuell
  Overflow abgeschnitten. Zukünftige Phase kann das als „Splittet auf 2
  Seiten wenn Slot-Count > N" implementieren. Der Header-Renderer ist bereits
  so implementiert, dass er auf jeder Seite auftaucht (D-50-03), also
  Vorbereitung ist da.
- **Chip-Look mit Farbe** (G3-b) — verworfen zugunsten Plain-Text. Falls
  User später sagt „ich will die Farben aus dem Browser doch", trivial
  nachrüstbar (`sales_person.background_color` existiert).
- **Auto-Shrink Font-Size** (G2-c) — verworfen. Falls Overflow ein echtes
  Problem wird, wäre das eine Erweiterung.
- **Time-Grid analog Browser** (G1-a) — verworfen zugunsten Hybrid-Stack.
  Wäre eine spätere „PDF-Layout v2".
- **TZ-Konvertierung via `time-tz`** (G6-b) — verworfen, würde v2.3-„keine
  neuen Deps" brechen. Falls OS-TZ ≠ ENV `TIMEZONE` in einem Deployment,
  müssten wir das erst wirklich beobachten, bevor wir eine Dep hinzunehmen.
- **TZ-Suffix im Timestamp** (G6-c) — verworfen. Wenn nötig, trivialer 1-Line-
  Nachtrag.
- **Seitennummerierung** — nicht in Scope. Sinnvoll erst wenn multi-page
  eingeführt wird.
- **PDF-Preview im Browser** — nicht diskutiert, kein Scope (schon in Phase
  49 als deferred markiert).

</deferred>

---

*Phase: 50-pdf-renderer-browser-look*
*Context gathered: 2026-07-03*
