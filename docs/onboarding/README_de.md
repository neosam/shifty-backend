# Onboarding — Neue Entwickler:innen

Diese Sektion bringt dich vom leeren Laptop zum ersten Merge.

## Reihenfolge

1. **[first-week.md](./first-week.md)** — Was du in den ersten Tagen brauchst:
   Repo klonen, Toolchain, Editor-Setup, Backend + Frontend starten, erster
   Fake-Bug-Fix.
2. **Architektur verstehen** — Bevor du Code schreibst, lies:
   - [`../architecture/01-layered.md`](../architecture/01-layered.md) — Warum REST → Service → DAO → SQLite.
   - [`../architecture/02-service-tiers.md`](../architecture/02-service-tiers.md) — Basic vs Business-Logic; wann welcher Service?
   - [`../architecture/05-transactions.md`](../architecture/05-transactions.md) — Das `Option<Transaction>`-Pattern.
3. **Fach kennen lernen** — Ohne Fach ist Code Roulette:
   - [`../domain/glossary.md`](../domain/glossary.md) — Sales Person, Slot, Booking, Absence, Balance, Billing Period.
   - [`../domain/time-accounting.md`](../domain/time-accounting.md) — Wie das Stundenkonto berechnet wird.
   - [`../domain/edge-cases.md`](../domain/edge-cases.md) — Die scharfen Kanten. Bitte lesen, bevor du am Reporting anfasst.
4. **Konventionen respektieren**:
   - [`../architecture/07-testing.md`](../architecture/07-testing.md) — Mockall, In-Mem-SQLite, `cargo sqlx prepare`, Clippy-Gate.
   - [`../architecture/08-i18n.md`](../architecture/08-i18n.md) — Neue Strings brauchen En/De/Cs.

## Wichtige Grundhaltung

- **Kein Fachwissen im Frontend duplizieren.** Rechnest du im UI Stunden aus,
  ist der falsche Fluss. Rechne im Backend, sende das Ergebnis.
- **Kein Hard-Delete.** Alle Löschungen sind Soft-Delete. Reader filtern
  `deleted IS NULL`.
- **Kein direkter `git commit`.** Dieses Repo läuft auf **jj** (co-located mit
  git). Der GSD-Executor committet automatisch via git; manuelle Commits
  gehen ausschließlich über `jj`.
- **Kein Backend-Endpoint ohne `Dioxus.toml`-Proxy-Eintrag**, wenn das Frontend
  ihn ansprechen soll — sonst 404 im `dx serve`-Dev-Modus.

## Hilfe

- `.planning/` — GSD-Planungsartefakte für aktuelle und vergangene Phasen. Das
  ist Kontext, warum ein Feature so aussieht, wie es aussieht.
- `CLAUDE.md` (Repo-Root) — Kurzform der wichtigsten Konventionen.
- Diese Doku ist das nachschlagbare Langformat davon.
