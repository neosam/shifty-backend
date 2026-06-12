# Quick Task 260612-svs: Absences-Seite — Krankheitstage vorerst ausblenden - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning

<domain>
## Task Boundary

Im Frontend (Absences-Seite) fürs Erste ausschließlich Urlaubstage und unbezahlte Tage
anzeigen — noch keine Krankheitstage, da dort fachlich noch etwas fehlt.

**Scope: NUR die Absences-Seite** (`shifty-dioxus/src/page/absences.rs`).
Reports (Employee-Report, Billing etc.) bleiben vollständig unverändert —
dort müssen Krankheitstage weiterhin erscheinen.

</domain>

<decisions>
## Implementation Decisions

### Liste (bestehende Krankheits-Einträge)
- Komplett ausblenden: Krankheits-Absences UND stundenbasierte Krankheits-Marker
  erscheinen nicht in der Liste.

### Anlegen/Bearbeiten (AbsenceModal)
- Kategorie-Option „Krankheit" aus dem Dropdown entfernen — neue Krankheitstage
  können über diese Seite nicht angelegt werden.

### Stats-Karte + Kategorie-Filter
- Stats-Kachel „Krankheitstage" auf der Absences-Seite ausblenden.
- Kategorie-Filter-Option „Krankheit" in der FilterBar entfernen.
- **Reports bleiben wie sie sind** — keine Änderungen außerhalb der Absences-Seite.

### Umsetzung
- Zentrale Konstante (z.B. `SICK_LEAVE_ENABLED: bool = false`) in `absences.rs`;
  alle Ausblendungs-Stellen prüfen dagegen. Reaktivierung später per One-Liner.

### Claude's Discretion
- Exakter Name/Platzierung der Konstante.
- Verhalten, falls der aktive Kategorie-Filter beim Laden auf „sick_leave" steht
  (z.B. aus URL/State): sinnvoll auf „alle" zurückfallen.
- Test-Abdeckung gemäß bestehender Konvention (Pure-Function- + SSR-Snapshot-Tests).

</decisions>

<specifics>
## Specific Ideas

- Krankheitstage sollen später wiederkommen — die Abschaltung ist explizit temporär
  („fürs Erste"), daher Konstante statt hartem Entfernen.

</specifics>

<canonical_refs>
## Canonical References

No external specs — requirements fully captured in decisions above.

</canonical_refs>
