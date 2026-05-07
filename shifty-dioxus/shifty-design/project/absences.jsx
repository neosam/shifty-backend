// absences.jsx — Range-based Absence Management UI
// Implements the frontend surface for the new /absence-period endpoints.
// - List page (CRUD)
// - Create/Edit modal with non-blocking warning rendering
// - UnavailabilityMarker chip helper
// - Deprecation banner helper for legacy ExtraHours flow

const { useState: useStateA, useMemo: useMemoA } = React;

// ─── Helpers ────────────────────────────────────────────────
function isoToDe(iso) {
  if (!iso) return '';
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  return m ? `${m[3]}.${m[2]}.${m[1]}` : iso;
}
function deToIso(de) {
  if (!de) return '';
  const m = /^(\d{2})\.(\d{2})\.(\d{4})$/.exec(de);
  return m ? `${m[3]}-${m[2]}-${m[1]}` : de;
}
function daysInRange(fromIso, toIso) {
  if (!fromIso || !toIso) return 0;
  const a = new Date(fromIso), b = new Date(toIso);
  return Math.max(0, Math.round((b - a) / 86_400_000) + 1);
}
function rangeStatus(fromIso, toIso) {
  // "today" pinned to KW17/2026 to match the rest of the demo
  const today = new Date('2026-04-22');
  const a = new Date(fromIso), b = new Date(toIso);
  if (b < today) return { label: 'Beendet', tone: 'neutral' };
  if (a > today) return { label: 'Geplant',  tone: 'plan' };
  return { label: 'Aktiv', tone: 'active' };
}
function categoryMeta(catId) {
  return window.SHIFTY_DATA.ABSENCE_CATEGORIES[catId] || { id: catId, label: catId, short: '?', color: 'var(--ink-muted)', soft: 'var(--surface-2)' };
}

// ─── Reusable bits ──────────────────────────────────────────
function CategoryBadge({ category, size = 'md' }) {
  const meta = categoryMeta(category);
  const fontSize = size === 'sm' ? 11 : 12;
  const pad = size === 'sm' ? '1px 7px' : '2px 9px';
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      padding: pad, borderRadius: 999,
      background: meta.soft, color: meta.color,
      fontSize, fontWeight: 600, whiteSpace: 'nowrap',
    }}>
      <span style={{ width: 7, height: 7, borderRadius: '50%', background: meta.color }} />
      {meta.label}
    </span>
  );
}

function StatusPill({ status }) {
  const colors = {
    active:  { bg: 'var(--accent-soft)', fg: 'var(--accent)' },
    plan:    { bg: 'var(--surface-2)',   fg: 'var(--ink-soft)' },
    neutral: { bg: 'var(--surface-alt)', fg: 'var(--ink-muted)' },
  };
  const c = colors[status.tone] || colors.neutral;
  return (
    <span style={{
      fontSize: 11, padding: '2px 8px', borderRadius: 999,
      background: c.bg, color: c.fg, fontWeight: 600,
    }}>{status.label}</span>
  );
}

// Renders WarningTO[] as a compact non-blocking list. Same shape used after
// POST/PUT /absence-period (forward warnings) and POST /shiftplan-edit/booking
// (reverse warnings).
function WarningList({ warnings, dense }) {
  if (!warnings || warnings.length === 0) return null;
  const explain = (w) => {
    switch (w.kind) {
      case 'absence_overlaps_booking':
        return `Bestehende Buchung am ${isoToDe(w.data.date)} überschneidet sich mit dieser Abwesenheit.`;
      case 'absence_overlaps_manual_unavailable':
        return `Manuell als unverfügbar markierter Tag überschneidet sich. Nach dem Cutover ist dieser Eintrag redundant.`;
      case 'booking_on_absence_day':
        return `Buchung am ${isoToDe(w.data.date)} liegt auf einem ${categoryMeta(w.data.category).label}-Tag.`;
      case 'booking_on_unavailable_day':
        return `Buchung in KW ${w.data.year}/${w.data.week} an Tag ${w.data.day_of_week} fällt auf einen als unverfügbar markierten Tag.`;
      case 'paid_employee_limit_exceeded':
        return `Maximalzahl bezahlter Mitarbeiter (${w.data.max_paid_employees}) im Slot überschritten.`;
      default:
        return JSON.stringify(w);
    }
  };
  return (
    <div style={{
      borderLeft: '3px solid var(--warn)',
      background: 'var(--warn-soft)',
      borderRadius: 'var(--r-md)',
      padding: dense ? '8px 10px' : '10px 12px',
      display: 'flex', flexDirection: 'column', gap: 4,
    }}>
      <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--warn)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
        Hinweis · {warnings.length} {warnings.length === 1 ? 'Konflikt' : 'Konflikte'} (nicht blockierend)
      </div>
      <ul style={{ margin: 0, paddingLeft: 16, fontSize: 13, color: 'var(--ink)', display: 'flex', flexDirection: 'column', gap: 2 }}>
        {warnings.map((w, i) => <li key={i}>{explain(w)}</li>)}
      </ul>
    </div>
  );
}

// Per-day marker chip, used inside the shiftplan grid header row.
function UnavailabilityChip({ marker, compact }) {
  if (!marker) return null;
  if (marker.kind === 'manual_unavailable') {
    return (
      <span title="Manuell als unverfügbar markiert" style={{
        display: 'inline-flex', alignItems: 'center', gap: 4,
        padding: compact ? '0 5px' : '1px 7px',
        height: compact ? 16 : 18,
        borderRadius: 'var(--r-sm)',
        background: 'var(--surface-2)', color: 'var(--ink-muted)',
        fontSize: 10, fontWeight: 600,
        border: '1px dashed var(--border-strong)',
      }}>
        <span style={{ fontSize: 9 }}>✕</span>frei
      </span>
    );
  }
  const meta = categoryMeta(marker.data.category);
  const isBoth = marker.kind === 'both';
  return (
    <span title={isBoth ? `${meta.label} (manueller Eintrag redundant)` : meta.label} style={{
      display: 'inline-flex', alignItems: 'center', gap: 4,
      padding: compact ? '0 5px' : '1px 7px',
      height: compact ? 16 : 18,
      borderRadius: 'var(--r-sm)',
      background: meta.soft, color: meta.color,
      fontSize: 10, fontWeight: 700,
      border: isBoth ? '1px dashed ' + meta.color : 'none',
    }}>
      {meta.short}{isBoth && '·!'}
    </span>
  );
}

// ─── Vacation entitlement card ─────────────────────────────
// Shows the urlaubsanspruch summary. Two scopes:
//  - 'self'  → big remaining-days hero + breakdown (employee view)
//  - 'team'  → aggregated team view + per-person mini list (HR view)
function VacationEntitlementCard({ summary, viewAs }) {
  const { PEOPLE, VACATION_QUOTA } = window.SHIFTY_DATA;
  const isSelf = summary.scope === 'self';
  const goodColor = window.SHIFTY_DATA.ABSENCE_CATEGORIES.Vacation.color;
  const goodSoft  = window.SHIFTY_DATA.ABSENCE_CATEGORIES.Vacation.soft;

  // Progress: used + pending out of entitled
  const usedPct    = summary.entitled === 0 ? 0 : Math.min(100, (summary.used / summary.entitled) * 100);
  const pendingPct = summary.entitled === 0 ? 0 : Math.min(100 - usedPct, (summary.pending / summary.entitled) * 100);

  return (
    <div style={{
      background: 'var(--surface)', border: '1px solid var(--border)',
      borderRadius: 'var(--r-lg)', overflow: 'hidden',
    }}>
      <div style={{
        display: 'grid',
        gridTemplateColumns: isSelf ? 'minmax(180px, 240px) 1fr' : '1fr',
        gap: 0,
      }}>
        {/* Hero column (self only) */}
        {isSelf && (
          <div style={{
            background: goodSoft, padding: '18px 20px',
            display: 'flex', flexDirection: 'column', justifyContent: 'center',
            borderRight: '1px solid var(--border)',
          }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: goodColor, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
              Urlaubsanspruch 2026
            </div>
            <div className="mono" style={{ fontSize: 40, fontWeight: 700, color: goodColor, lineHeight: 1.05, marginTop: 6 }}>
              {summary.remaining}<span style={{ fontSize: 16, fontWeight: 600, marginLeft: 4, opacity: 0.7 }}>/ {summary.entitled}</span>
            </div>
            <div style={{ fontSize: 12, color: 'var(--ink-soft)', marginTop: 2 }}>
              Tage verbleibend
            </div>
          </div>
        )}

        {/* Breakdown column */}
        <div style={{ padding: '14px 18px', display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap' }}>
            <div>
              <div style={{ fontSize: 14, fontWeight: 700 }}>
                {isSelf ? 'Dein Urlaubskonto' : `Urlaubsanspruch Team · ${summary.count} Personen`}
              </div>
              <div style={{ fontSize: 12, color: 'var(--ink-muted)' }}>
                {isSelf
                  ? 'Anspruch aus Vertrag + Übertrag aus dem Vorjahr.'
                  : 'Summe über alle bezahlten Mitarbeiter.'}
              </div>
            </div>
            {!isSelf && (
              <div className="mono" style={{ fontSize: 22, fontWeight: 700, color: goodColor }}>
                {summary.remaining}<span style={{ fontSize: 13, fontWeight: 600, opacity: 0.6 }}> / {summary.entitled} Tage</span>
              </div>
            )}
          </div>

          {/* Progress bar */}
          <div style={{
            height: 8, borderRadius: 999, background: 'var(--surface-alt)',
            overflow: 'hidden', display: 'flex', position: 'relative',
          }}>
            <div style={{ width: usedPct + '%', background: goodColor }} />
            {pendingPct > 0 && (
              <div style={{
                width: pendingPct + '%', background: goodColor, opacity: 0.4,
                backgroundImage: `repeating-linear-gradient(45deg, transparent 0 4px, rgba(255,255,255,0.4) 4px 8px)`,
              }} />
            )}
          </div>

          {/* Breakdown rows */}
          <div style={{
            display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))', gap: 14,
          }}>
            <Stat label="Vertrag" value={summary.total} unit="Tage" />
            <Stat label="Übertrag '25" value={summary.carryover} unit="Tage" muted={summary.carryover === 0} />
            <Stat label="Genommen" value={summary.used} unit="Tage" tone="muted" />
            {summary.pending > 0 && <Stat label="Beantragt" value={summary.pending} unit="Tage" tone="warn" />}
            <Stat label="Verbleibend" value={summary.remaining} unit="Tage" tone="good" strong />
          </div>
        </div>
      </div>

      {/* HR-only: per-person breakdown */}
      {!isSelf && <VacationPerPersonList />}
    </div>
  );
}

function Stat({ label, value, unit, tone, strong, muted }) {
  const color = tone === 'good' ? window.SHIFTY_DATA.ABSENCE_CATEGORIES.Vacation.color
              : tone === 'warn' ? 'var(--warn)'
              : tone === 'muted' ? 'var(--ink-soft)'
              : 'var(--ink)';
  return (
    <div>
      <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--ink-muted)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>{label}</div>
      <div className="mono" style={{ fontSize: 18, fontWeight: strong ? 700 : 600, color, opacity: muted ? 0.5 : 1, marginTop: 2 }}>
        {value}<span style={{ fontSize: 11, fontWeight: 500, marginLeft: 3, opacity: 0.6 }}>{unit}</span>
      </div>
    </div>
  );
}

function VacationPerPersonList() {
  const { PEOPLE, VACATION_QUOTA } = window.SHIFTY_DATA;
  const goodColor = window.SHIFTY_DATA.ABSENCE_CATEGORIES.Vacation.color;
  const [expanded, setExpanded] = useStateA(false);

  const rows = PEOPLE
    .map((p) => {
      const q = VACATION_QUOTA[p.id];
      if (!q || (q.total + q.carryover) === 0) return null;
      const entitled  = q.total + q.carryover;
      const remaining = entitled - q.used - q.pending;
      const pct = entitled === 0 ? 0 : (q.used / entitled) * 100;
      const low = remaining <= 3;
      return { p, q, entitled, remaining, pct, low };
    })
    .filter(Boolean)
    .sort((a, b) => a.remaining - b.remaining); // lowest remaining first

  const visible = expanded ? rows : rows.slice(0, 4);

  return (
    <div style={{ borderTop: '1px solid var(--border)', background: 'var(--surface-alt)' }}>
      <div style={{
        padding: '10px 18px', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        fontSize: 11, fontWeight: 600, color: 'var(--ink-muted)', textTransform: 'uppercase', letterSpacing: '0.04em',
      }}>
        <span>Pro Person · sortiert nach verbleibenden Tagen</span>
        <button onClick={() => setExpanded((v) => !v)} style={{
          background: 'none', border: 'none', color: 'var(--accent)', cursor: 'pointer',
          fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em',
        }}>
          {expanded ? 'Weniger' : `Alle (${rows.length})`}
        </button>
      </div>
      <div style={{ padding: '0 18px 14px', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))', gap: 8 }}>
        {visible.map(({ p, q, entitled, remaining, pct, low }) => (
          <div key={p.id} style={{
            background: 'var(--surface)', border: '1px solid var(--border)',
            borderRadius: 'var(--r-md)', padding: '8px 12px',
            display: 'flex', flexDirection: 'column', gap: 6,
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span style={{
                width: 22, height: 22, borderRadius: '50%',
                background: p.color || 'transparent',
                border: p.color ? 'none' : '1px dashed var(--border-strong)',
                display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 10, fontWeight: 700, flexShrink: 0,
              }}>{p.initials}</span>
              <span style={{ fontSize: 13, fontWeight: 600, flex: 1, minWidth: 0, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{p.name}</span>
              <span className="mono" style={{
                fontSize: 13, fontWeight: 700,
                color: low ? 'var(--warn)' : goodColor,
              }}>{remaining}<span style={{ fontSize: 10, opacity: 0.6, fontWeight: 600 }}>/{entitled}</span></span>
            </div>
            <div style={{ height: 4, borderRadius: 999, background: 'var(--surface-alt)', overflow: 'hidden' }}>
              <div style={{ width: pct + '%', height: '100%', background: low ? 'var(--warn)' : goodColor }} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Absence page ───────────────────────────────────────────
function AbsencePage({ currentUser, modalVariant, viewAs }) {
  const { ABSENCES, PERSON_BY_ID, PEOPLE, VACATION_QUOTA } = window.SHIFTY_DATA;
  const isHr = viewAs !== 'employee';
  const [filterCat, setFilterCat]   = useStateA('all');
  const [filterPerson, setFilterPerson] = useStateA('all');
  const [showPast, setShowPast] = useStateA(true);
  const [editing, setEditing]   = useStateA(null); // null | 'new' | absence-id
  const [absences, setAbsences] = useStateA(ABSENCES);

  const today = new Date('2026-04-22');
  const filtered = absences.filter((a) => {
    if (!isHr && a.sales_person_id !== currentUser.id) return false;
    if (filterCat !== 'all' && a.category !== filterCat) return false;
    if (isHr && filterPerson !== 'all' && a.sales_person_id !== filterPerson) return false;
    if (!showPast && new Date(a.to_date) < today) return false;
    return true;
  }).sort((a, b) => b.from_date.localeCompare(a.from_date));

  // Quick stats: this calendar year (also scoped to viewer)
  const thisYear = '2026';
  const totals = useMemoA(() => {
    const t = { Vacation: 0, SickLeave: 0, UnpaidLeave: 0 };
    absences.forEach((a) => {
      if (!isHr && a.sales_person_id !== currentUser.id) return;
      if (a.from_date.startsWith(thisYear)) {
        t[a.category] = (t[a.category] || 0) + daysInRange(a.from_date, a.to_date);
      }
    });
    return t;
  }, [absences, isHr, currentUser.id]);

  // Vacation entitlement: in employee mode → just current user's quota.
  // In HR mode → aggregate across all paid people (sum total + carryover − used).
  const vacationSummary = useMemoA(() => {
    if (!isHr) {
      const q = VACATION_QUOTA[currentUser.id];
      if (!q) return null;
      const entitled = q.total + q.carryover;
      return {
        scope: 'self',
        total: q.total, carryover: q.carryover, used: q.used,
        entitled, remaining: entitled - q.used - q.pending, pending: q.pending,
      };
    }
    let total = 0, carryover = 0, used = 0, pending = 0, count = 0;
    PEOPLE.forEach((p) => {
      const q = VACATION_QUOTA[p.id];
      if (!q || (q.total + q.carryover) === 0) return;
      total += q.total; carryover += q.carryover; used += q.used; pending += q.pending; count++;
    });
    const entitled = total + carryover;
    return { scope: 'team', total, carryover, used, pending, entitled, remaining: entitled - used - pending, count };
  }, [isHr, currentUser.id]);

  const scopedActive = absences.filter((a) =>
    (isHr || a.sales_person_id === currentUser.id) &&
    new Date(a.from_date) <= today && new Date(a.to_date) >= today
  ).length;

  const scopedTotal = absences.filter((a) => isHr || a.sales_person_id === currentUser.id).length;

  const editingAbs = editing && editing !== 'new' ? absences.find((a) => a.id === editing) : null;

  const onSave = (data) => {
    if (editing === 'new') {
      setAbsences((xs) => [{ ...data, id: 'a-' + Math.random().toString(36).slice(2, 7), created: new Date().toISOString(), deleted: null, version: 'v-1' }, ...xs]);
    } else {
      setAbsences((xs) => xs.map((a) => a.id === editing ? { ...a, ...data } : a));
    }
    setEditing(null);
  };  const onDelete = (id) => {
    if (!confirm('Abwesenheit wirklich löschen? (Soft-Delete — bleibt für Audit-Logs erhalten.)')) return;
    setAbsences((xs) => xs.filter((a) => a.id !== id));
    setEditing(null);
  };

  return (
    <div style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap' }}>
        <div>
          <h1 style={{ fontSize: 22, fontWeight: 600, margin: 0, letterSpacing: '-0.01em' }}>Abwesenheiten</h1>
          <div style={{ fontSize: 13, color: 'var(--ink-muted)', marginTop: 2 }}>
            Urlaub, Krankheit und unbezahlte Freistellung als Zeiträume. Stunden pro Tag werden aus dem gültigen Arbeitsvertrag abgeleitet.
          </div>
        </div>
        <Btn kind="primary" icon="+" onClick={() => setEditing('new')}>Neue Abwesenheit</Btn>
      </div>

      {/* Vacation entitlement — featured card */}
      {vacationSummary && (
        <VacationEntitlementCard summary={vacationSummary} viewAs={viewAs} />
      )}

      {/* Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))', gap: 10 }}>
        {[
          ['Krankheitstage 2026', totals.SickLeave,  'SickLeave'],
          ['Unbezahlt 2026',     totals.UnpaidLeave, 'UnpaidLeave'],
          ['Aktive Abwesenheiten', scopedActive, null],
        ].map(([label, val, cat]) => {
          const meta = cat ? categoryMeta(cat) : null;
          return (
            <div key={label} style={{
              background: 'var(--surface)', border: '1px solid var(--border)',
              borderRadius: 'var(--r-md)', padding: '10px 14px',
            }}>
              <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--ink-muted)', textTransform: 'uppercase', letterSpacing: '0.04em', display: 'flex', alignItems: 'center', gap: 6 }}>
                {meta && <span style={{ width: 7, height: 7, borderRadius: '50%', background: meta.color }} />}
                {label}
              </div>
              <div className="mono" style={{ fontSize: 22, fontWeight: 600, marginTop: 2 }}>
                {val}{cat ? ' Tage' : ''}
              </div>
            </div>
          );
        })}
      </div>

      {/* Filter row */}
      <div style={{ display: 'flex', gap: 10, alignItems: 'center', flexWrap: 'wrap',
        background: 'var(--surface)', border: '1px solid var(--border)',
        borderRadius: 'var(--r-lg)', padding: '10px 14px',
      }}>
        <span style={{ fontSize: 12, color: 'var(--ink-muted)' }}>Kategorie</span>
        <div style={{ display: 'flex', background: 'var(--surface-alt)', padding: 2, borderRadius: 'var(--r-md)' }}>
          {[['all','Alle'], ['Vacation','Urlaub'], ['SickLeave','Krank'], ['UnpaidLeave','Unbezahlt']].map(([k, lbl]) => (
            <button key={k} onClick={() => setFilterCat(k)} style={{
              padding: '4px 10px', borderRadius: 'var(--r-sm)', border: 'none',
              background: filterCat === k ? 'var(--surface)' : 'transparent',
              color: filterCat === k ? 'var(--ink)' : 'var(--ink-muted)',
              fontSize: 12, fontWeight: 600, cursor: 'pointer',
              boxShadow: filterCat === k ? '0 1px 2px rgba(15,18,30,0.06)' : 'none',
            }}>{lbl}</button>
          ))}
        </div>

        <span style={{ width: 1, height: 22, background: 'var(--border)', margin: '0 4px' }} />

        {isHr && (<>
        <span style={{ fontSize: 12, color: 'var(--ink-muted)' }}>Person</span>
        <select value={filterPerson} onChange={(e) => setFilterPerson(e.target.value)} style={{
          padding: '5px 10px', borderRadius: 'var(--r-md)', border: '1px solid var(--border-strong)',
          background: 'var(--surface)', fontSize: 13, color: 'var(--ink)',
        }}>
          <option value="all">Alle Personen</option>
          {PEOPLE.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}
        </select>
        </>)}

        <label style={{ display: 'inline-flex', alignItems: 'center', gap: 6, fontSize: 13, color: 'var(--ink-muted)', cursor: 'pointer', marginLeft: 'auto' }}>
          <input type="checkbox" checked={showPast} onChange={(e) => setShowPast(e.target.checked)} />
          Vergangene anzeigen
        </label>
        <span style={{ fontSize: 12, color: 'var(--ink-muted)' }}>{filtered.length} von {scopedTotal}</span>
      </div>

      {/* List */}
      <div style={{
        background: 'var(--surface)', border: '1px solid var(--border)',
        borderRadius: 'var(--r-lg)', overflow: 'hidden',
      }}>
        {/* Column header */}
        {filtered.length > 0 && (
          <div style={{
            display: 'grid',
            gridTemplateColumns: '1.5fr 170px 140px 90px 70px',
            gap: 14, alignItems: 'center',
            padding: '8px 16px',
            background: 'var(--surface-alt)',
            borderBottom: '1px solid var(--border)',
            fontSize: 11, fontWeight: 600, color: 'var(--ink-muted)',
            textTransform: 'uppercase', letterSpacing: '0.04em',
          }}>
            <div>Mitarbeiter</div>
            <div>Zeitraum</div>
            <div>Kategorie</div>
            <div>Status</div>
            <div style={{ textAlign: 'right' }}>Hinweise</div>
          </div>
        )}
        {filtered.length === 0 ? (
          <div style={{ padding: 40, textAlign: 'center', color: 'var(--ink-muted)', fontSize: 14 }}>
            Keine Abwesenheiten für diesen Filter.
          </div>
        ) : filtered.map((a, idx) => {
          const p = PERSON_BY_ID[a.sales_person_id];
          const days = daysInRange(a.from_date, a.to_date);
          const meta = categoryMeta(a.category);
          const status = rangeStatus(a.from_date, a.to_date);
          const hasWarn = a.warnings && a.warnings.length > 0;
          return (
            <button key={a.id} onClick={() => setEditing(a.id)}
              style={{
                width: '100%', textAlign: 'left',
                display: 'grid',
                gridTemplateColumns: '1.5fr 170px 140px 90px 70px',
                gap: 14, alignItems: 'center',
                padding: '14px 16px',
                background: 'transparent', border: 'none',
                borderTop: idx > 0 ? '1px solid var(--border)' : 'none',
                cursor: 'pointer', color: 'var(--ink)',
                position: 'relative',
              }}
              onMouseEnter={(e) => e.currentTarget.style.background = 'var(--surface-alt)'}
              onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
            >
              {/* Person */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, minWidth: 0 }}>
                <span aria-hidden="true" className={`person-avatar${p?.color ? '' : ' no-color'}`} style={{
                  width: 32, height: 32, borderRadius: '50%',
                  background: p?.color || 'transparent',
                  border: p?.color ? 'none' : '1px dashed var(--border-strong)',
                  display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
                  fontSize: 12, fontWeight: 700, flexShrink: 0,
                }}>{p?.initials}</span>
                <div style={{ minWidth: 0 }}>
                  <div style={{ fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{p?.name}</div>
                  {a.description && <div style={{ fontSize: 12, color: 'var(--ink-muted)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{a.description}</div>}
                </div>
              </div>

              {/* Range */}
              <div className="mono" style={{ fontSize: 13 }}>
                <div>{isoToDe(a.from_date)} – {isoToDe(a.to_date)}</div>
                <div style={{ fontSize: 11, color: 'var(--ink-muted)' }}>{days} {days === 1 ? 'Tag' : 'Tage'}</div>
              </div>

              {/* Category */}
              <div><CategoryBadge category={a.category} /></div>

              {/* Status */}
              <div><StatusPill status={status} /></div>

              {/* Warnings */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--ink-muted)', justifyContent: 'flex-end' }}>
                {hasWarn && (
                  <span style={{
                    display: 'inline-flex', alignItems: 'center', gap: 4,
                    padding: '2px 8px', borderRadius: 999,
                    background: 'var(--warn-soft)', color: 'var(--warn)',
                    fontSize: 11, fontWeight: 700,
                  }} title={a.warnings.length + ' nicht-blockierende Konflikte'}>
                    ⚠ {a.warnings.length}
                  </span>
                )}
                <span style={{ fontSize: 16, color: 'var(--ink-muted)' }}>›</span>
              </div>
            </button>
          );
        })}
      </div>

      <AbsenceModal
        open={editing != null}
        onClose={() => setEditing(null)}
        variant={modalVariant}
        absence={editingAbs}
        isNew={editing === 'new'}
        onSave={onSave}
        onDelete={onDelete}
        lockPerson={!isHr}
        defaultPersonId={!isHr ? currentUser.id : undefined}
      />
    </div>
  );
}

// ─── Modal ──────────────────────────────────────────────────
function AbsenceModal({ open, onClose, variant, absence, isNew, onSave, onDelete, lockPerson, defaultPersonId }) {
  const { PEOPLE } = window.SHIFTY_DATA;
  // Reset state when the modal target changes (mounting once + key dance is overkill here)
  const [salesPersonId, setSP]   = useStateA(absence?.sales_person_id || defaultPersonId || PEOPLE[0].id);
  const [category, setCategory]  = useStateA(absence?.category || 'Vacation');
  const [fromDate, setFromDate]  = useStateA(absence?.from_date || '2026-04-20');
  const [toDate, setToDate]      = useStateA(absence?.to_date || '2026-04-24');
  const [description, setDesc]   = useStateA(absence?.description || '');

  React.useEffect(() => {
    if (!open) return;
    setSP(absence?.sales_person_id || defaultPersonId || PEOPLE[0].id);
    setCategory(absence?.category || 'Vacation');
    setFromDate(absence?.from_date || '2026-04-20');
    setToDate(absence?.to_date || '2026-04-24');
    setDesc(absence?.description || '');
  }, [open, absence?.id]);

  const days = daysInRange(fromDate, toDate);
  const invalidRange = !fromDate || !toDate || new Date(toDate) < new Date(fromDate);
  const warnings = absence?.warnings || [];

  // Naive client-side self-overlap detection (server enforces 422; we mirror UX)
  const selfOverlap = useMemoA(() => {
    if (!salesPersonId || !fromDate || !toDate) return null;
    return window.SHIFTY_DATA.ABSENCES.find((a) =>
      a.id !== absence?.id &&
      a.sales_person_id === salesPersonId &&
      a.category === category &&
      !(a.to_date < fromDate || a.from_date > toDate)
    );
  }, [salesPersonId, category, fromDate, toDate, absence?.id]);

  const handleSubmit = () => {
    if (invalidRange || selfOverlap) return;
    onSave({ sales_person_id: salesPersonId, category, from_date: fromDate, to_date: toDate, description, warnings: warnings });
  };

  const cats = Object.values(window.SHIFTY_DATA.ABSENCE_CATEGORIES);
  const meta = categoryMeta(category);

  return (
    <Modal
      open={open} onClose={onClose} variant={variant}
      title={isNew ? 'Neue Abwesenheit' : 'Abwesenheit bearbeiten'}
      subtitle={isNew ? 'Ganztagiger Zeitraum. Stunden werden aus dem Vertrag abgeleitet.' : 'Änderungen werden mit optimistic-locking gespeichert.'}
      width={520}
      footer={<>
        {!isNew && <Btn kind="danger" onClick={() => onDelete(absence.id)}>Löschen</Btn>}
        <span style={{ flex: 1 }} />
        <Btn kind="ghost" onClick={onClose}>Abbrechen</Btn>
        <Btn kind="primary" onClick={handleSubmit} disabled={invalidRange || !!selfOverlap}>
          {isNew ? 'Anlegen' : 'Speichern'}
        </Btn>
      </>}
    >
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <Field label="Mitarbeiter" span={2}>
          <SelectInput value={salesPersonId} onChange={(e) => setSP(e.target.value)} disabled={lockPerson}>
            {PEOPLE.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}
          </SelectInput>
        </Field>

        <Field label="Kategorie" span={2}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {cats.map((c) => {
              const active = category === c.id;
              return (
                <button key={c.id} type="button" onClick={() => setCategory(c.id)} style={{
                  padding: '6px 12px', borderRadius: 999,
                  border: '1px solid ' + (active ? c.color : 'var(--border-strong)'),
                  background: active ? c.soft : 'var(--surface)',
                  color: active ? c.color : 'var(--ink-soft)',
                  fontSize: 13, fontWeight: active ? 600 : 500, cursor: 'pointer',
                  display: 'inline-flex', alignItems: 'center', gap: 6,
                }}>
                  <span style={{ width: 8, height: 8, borderRadius: '50%', background: c.color }} />
                  {c.label}
                </button>
              );
            })}
          </div>
        </Field>

        <Field label="Von">
          <TextInput type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} />
        </Field>
        <Field label="Bis (inklusiv)" error={invalidRange ? 'Endedatum vor Startdatum' : null}>
          <TextInput type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} />
        </Field>

        <Field label="Beschreibung" span={2} hint="Optional — z.B. Reiseziel oder Anmerkung.">
          <TextareaInput value={description} onChange={(e) => setDesc(e.target.value)} placeholder={category === 'Vacation' ? 'z.B. Sommerurlaub' : category === 'SickLeave' ? 'z.B. AU bis 25.04' : ''} />
        </Field>

        {/* Live preview */}
        <div style={{ gridColumn: 'span 2', padding: 12, background: 'var(--surface-alt)', border: '1px solid var(--border)', borderRadius: 'var(--r-md)' }}>
          <div style={{ fontSize: 11, color: 'var(--ink-muted)', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: 6 }}>Vorschau</div>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <CategoryBadge category={category} />
              <span className="mono" style={{ fontSize: 13, fontWeight: 600 }}>
                {invalidRange ? '—' : `${isoToDe(fromDate)} – ${isoToDe(toDate)}`}
              </span>
            </div>
            <div className="mono" style={{ fontSize: 13, color: 'var(--ink-muted)' }}>
              {invalidRange ? '0 Tage' : `${days} ${days === 1 ? 'Tag' : 'Tage'}`}
            </div>
          </div>
          <div style={{ fontSize: 11, color: 'var(--ink-muted)', marginTop: 6 }}>
            Feiertage im Bereich werden mit 0 h verrechnet. Stunden pro Tag stammen aus dem am jeweiligen Tag gültigen Arbeitsvertrag.
          </div>
        </div>

        {/* Self-overlap (422 from server) */}
        {selfOverlap && (
          <div style={{ gridColumn: 'span 2', padding: '10px 12px', borderRadius: 'var(--r-md)',
            background: 'var(--bad-soft)', borderLeft: '3px solid var(--bad)',
            color: 'var(--ink)', fontSize: 13,
          }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--bad)', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: 2 }}>
              Selbst-Überlappung (422)
            </div>
            Diese Person hat bereits eine {meta.label}-Abwesenheit von {isoToDe(selfOverlap.from_date)} bis {isoToDe(selfOverlap.to_date)}, die sich überschneidet.
          </div>
        )}

        {/* Forward warnings (server-side) */}
        {warnings.length > 0 && (
          <div style={{ gridColumn: 'span 2' }}>
            <WarningList warnings={warnings} />
          </div>
        )}
      </div>
    </Modal>
  );
}

Object.assign(window, { AbsencePage, AbsenceModal, CategoryBadge, UnavailabilityChip, WarningList, isoToDe, deToIso, daysInRange, categoryMeta });