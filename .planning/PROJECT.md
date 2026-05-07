---
type: project_charter
last_updated: 2026-05-07
---

# Shifty — Project Charter

## Was ist Shifty

Employee shift planning + HR-Management mit zwei gekoppelten Subprojekten:

- **`shifty-backend/`** (dieses Repo): Rust-Backend (Axum, SQLite, layered architecture).
  Authoritative source für Domain-Logik, REST-API, Persistenz, Reporting.
- **`../shifty-dioxus/`** (Schwester-Repo): Dioxus-Frontend (WASM). Konsumiert das
  Backend ausschließlich über REST.

Geteilte Crates:

- **`rest-types`**: API-DTOs. Heute in *beiden* Repos dupliziert — siehe
  [Bekannte Constraints](#bekannte-constraints).

## GSD-Scope-Regel

**Phasen umfassen Backend UND Frontend.**

Jede Phase, die ein neues TO einführt oder ein bestehendes ändert, hat per
Default Frontend-Anteil. Jeder Plan muss in seinem Header explizit benennen,
welche Pfade in beiden Subprojekten betroffen sind:

```
**Backend-Pfade:**
- `service/src/...`
- `rest/src/...`
- `rest-types/src/...`

**Frontend-Pfade:**
- `../shifty-dioxus/src/...`
- `../shifty-dioxus/rest-types/src/...`   (bis Konsolidierung)
```

"Frontend out of scope" ist erlaubt, **braucht aber eine begründete Notiz im
DISCUSS** (z. B. „rein interne Refactor-Phase ohne API-Wirkung", „Frontend
folgt in Folge-Phase v1.X+1, getrackt in Backlog"). Eine Phase ohne sichtbare
API-Wirkung darf still ohne Frontend-Anteil laufen.

**Konsequenz für `verify-work`:** UAT muss Frontend-Pfad mitprüfen, wenn
Frontend-Anteil im Plan stand. „Backend-Tests grün" ist nicht ausreichend
für Phasen mit Frontend-Anteil.

## Quellen-Hierarchie

| Zweck | Quelle |
|---|---|
| Backend-Konventionen | `shifty-backend/CLAUDE.md` |
| Frontend-Konventionen | `../shifty-dioxus/CLAUDE.md` (Executor lädt automatisch beim Lesen von Frontend-Files) |
| Frontend-Codebase-Map | `.planning/codebase/frontend/` (manuell verortet — `gsd:map-codebase` läuft nicht out-of-the-box auf Sister-Verzeichnis) |
| Backend-Codebase-Map | (`CLAUDE.md` ist detailliert genug; bewusst keine `.planning/codebase/`-Map) |
| Roadmap & Phasen | `.planning/ROADMAP.md`, `.planning/phases/` |
| Lokale Dev-Conventions | `shifty-backend/CLAUDE.local.md` (jj-only, NixOS-Spezifika) |

## Bekannte Constraints

### `rest-types`-Drift (Hochrisiko für Cross-Subprojekt-Konsistenz)

- **Backend**: `shifty-backend/rest-types/` v1.13.0-dev, 2041 Zeilen — single
  source of truth für Backend
- **Frontend**: `shifty-dioxus/rest-types/` v1.0.5-dev, 1468 Zeilen —
  gedrifteter Fork

Frontend kompiliert *nicht* gegen den Backend-Stand. Heißt: ein neuer Match-Arm
oder Feldname im Backend-`rest-types` schlägt sich nicht automatisch im
Frontend-Compile nieder. **Plan-Disziplin muss diese Lücke schließen**, bis die
Konsolidierung läuft (geplant in einer Folge-Phase: Frontend-Drift schließen +
Backend-`rest-types` als einzige Dependency setzen, mit
`default-features = false` für WASM-Kompatibilität).

Bekannte v1.1-Drift-Schulden im Frontend-Backlog:

- `current_paid_count` / `max_paid_employees` werden aktuell vom Frontend
  nicht gerendert (Phase 5 explizit Backend-only ausgeliefert)
- `VolunteerWork` / `UnpaidLeave` Extra-Hours-Kategorien fehlen in
  Frontend-Match-Armen
- `cap_planned_hours_to_expected` nicht im Frontend-Settings-UI

### Co-Location offen

Frontend liegt aktuell als Schwester-Verzeichnis. Plan-Pfade müssen relative
`../shifty-dioxus/...`-Pfade benutzen. Read/Edit-Tools funktionieren
verzeichnisagnostisch. Eine spätere Co-Location-Phase (`cp -r` plus
`exclude = ["shifty-dioxus"]` in Backend-Workspace) kann das vereinfachen,
ist aber nicht Voraussetzung für gemeinsame GSD-Phasen.

### Versionsabgleich

Beide Repos haben heute zufällig identische Versionsstände (`1.13.0-dev`).
Bei Releases müssen Backend- und Frontend-Versionen bewusst synchron gehalten
werden — Update via `cli-update-version.sh` in beiden Repos.

## Aktive Milestone

Siehe `.planning/ROADMAP.md`. Vorletzte Milestone (v1.1 — Slot Capacity)
abgeschlossen am 2026-05-04. Nächste Milestone noch nicht definiert —
`/gsd:new-milestone` als nächster Schritt.
