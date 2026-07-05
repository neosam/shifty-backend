# API-Konventionen

Diese Datei sammelt Konventionen, die über die einzelnen Endpoints
hinweg gelten. Für konkrete Endpoint-Definitionen siehe die
Feature-Dokus.

## Request/Response-Format

- **Content-Type:** `application/json` in beide Richtungen.
- **Character-Encoding:** UTF-8.

## HTTP-Methoden

| Methode | Semantik | Idempotent? |
| --- | --- | --- |
| GET | Lesen | Ja |
| POST | Anlegen oder Command | Nein |
| PUT | Full-Update / Upsert | Ja |
| PATCH | Teil-Update | Nein |
| DELETE | Soft-Delete (setzt `deleted`-Spalte) | Ja |

**Wichtig zu DELETE:** Shifty macht Soft-Delete, kein Hard-Delete.
Der Reader filtert `deleted IS NULL`. Ein `DELETE`-Aufruf setzt die
Spalte, entfernt die Zeile aber nicht physisch.

## URL-Struktur

- **Kebab-Case:** `/sales-person`, `/billing-period`, nicht
  `/salesPerson`.
- **Ressourcen im Plural:** `/bookings`, `/absences`.
- **Sub-Ressourcen:** `/sales-person/{id}/employee-work-details`.

**[Zu prüfen]** — Uniformität ist im Repo nicht 100% (manche Routen
sind Singular, manche Plural). Bei neuen Routen: konsistent bleiben.

## UUIDs

Alle Entity-IDs sind UUIDs, wire als Hyphenated-String:

```
"550e8400-e29b-41d4-a716-446655440000"
```

Nicht: comma-separiert, ohne Bindestriche, oder als Byte-Array.

## Zeit & Datum

- **Date-only:** ISO 8601, `YYYY-MM-DD`.
- **Datetime:** ISO 8601, `YYYY-MM-DDTHH:MM:SSZ` (UTC — **[Zu prüfen]**).

Kein Unix-Timestamp. Keine Locale-abhängige Formatierung im API.

## Enums

Wire-Format: String mit exaktem Rust-Variant-Namen.

```json
{ "category": "SickLeave" }
```

Nicht: Lowercase, Snake-Case, Numeric-ID.

## Optionale Felder

Rust `Option<T>` wird zu `null` oder komplett weggelassen —
**[Zu prüfen]** wie serde in Shifty konfiguriert ist (`skip_serializing_if`
oder `null`?).

## Fehler-Format

Fehler kommen als JSON zurück:

```json
{
  "error": "ValidationError",
  "message": "sales_person_id must not be empty",
  "details": null
}
```

**[Zu prüfen]** — genaues Feld-Set aus `error_handler`.

## Auth-Header

- **`mock_auth`:** Kein Header nötig.
- **`oidc`:** Bearer-Token:
  ```
  Authorization: Bearer <token>
  ```

## Transaktionen aus Client-Sicht

Ein API-Call = eine Backend-Transaktion. Der Client sieht **atomar**:
entweder Erfolg mit vollem Effekt oder Fehler ohne teilweisen Effekt.

Es gibt keine expliziten Client-Transaktionen ("BEGIN"/"COMMIT" über
mehrere Requests). Wenn du eine Composite-Op brauchst, ist das ein
Fall für einen dedizierten Backend-Endpoint, der die Ops intern
zusammenfasst.

## Pagination

**[Zu prüfen]** — aktueller Stand. Viele Endpoints scheinen Full-Liste
zu liefern. Für große Datensätze wäre offset-basierte Pagination der
nächste Schritt.

## Rate-Limiting

**[Zu prüfen]** — ob Rate-Limits gesetzt sind. Aktuell vermutlich
nicht.

## Versionierung des API

- **Kein URL-Prefix.** Aktuell ist `/booking`, nicht `/v1/booking`.
- **Breaking Changes** werden über die SemVer-Backend-Version
  kommuniziert. Ein Zweit-Client sollte die Backend-Version im UI oder
  Log anzeigen.
- **Für DTO-Änderungen:** Bei additiven Änderungen (neues Feld) kompatibel.
  Bei entfernten Feldern: Major-Bump.

## Idempotenz-Keys

**[Zu prüfen]** — ob Idempotency-Header-Support existiert. Aktuell
vermutlich nicht — Retry-Verhalten muss vom Client sicher gestaltet
werden (POST kann bei Retry zu Duplikaten führen).

## Long-Polling / WebSockets

**Nein.** Shifty ist Request-Response. Kein WebSocket, kein SSE. Für
Live-Updates muss der Client periodisch pollen.
