// ─── Modal-Komponente ─────────────────────────────────────────
// Drei Stile, umschaltbar via prop `variant`:
//   'center'   → klassisches Center-Modal mit Backdrop
//   'sheet'    → Side-Sheet von rechts
//   'bottom'   → Bottom-Sheet (gut auf Mobile)
//   'auto'     → Bottom-Sheet auf < 720px, sonst Center
//
// Verwendet die existierenden CSS-Variablen (var(--surface), --border, --ink, --accent…)
// und respektiert Light/Dark via `--modal-veil`.

const { useEffect: useEffectM, useState: useStateM, useRef: useRefM } = React;

function Modal({ open, onClose, title, subtitle, children, footer, variant = 'auto', width = 460 }) {
  // Resolve 'auto' → bottom on mobile, center on desktop
  const [resolved, setResolved] = useStateM(() => {
    if (variant !== 'auto') return variant;
    return (typeof window !== 'undefined' && window.matchMedia('(max-width: 720px)').matches) ? 'bottom' : 'center';
  });
  useEffectM(() => {
    if (variant !== 'auto') { setResolved(variant); return; }
    const mq = window.matchMedia('(max-width: 720px)');
    const apply = () => setResolved(mq.matches ? 'bottom' : 'center');
    apply();
    mq.addEventListener('change', apply);
    return () => mq.removeEventListener('change', apply);
  }, [variant]);

  // ESC schließt
  useEffectM(() => {
    if (!open) return;
    const onKey = (e) => { if (e.key === 'Escape') onClose && onClose(); };
    window.addEventListener('keydown', onKey);
    // Body scroll lock
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => {
      window.removeEventListener('keydown', onKey);
      document.body.style.overflow = prev;
    };
  }, [open, onClose]);

  if (!open) return null;

  const isCenter = resolved === 'center';
  const isSheet  = resolved === 'sheet';
  const isBottom = resolved === 'bottom';

  // Backdrop
  const backdropStyle = {
    position: 'fixed', inset: 0,
    background: 'var(--modal-veil)',
    zIndex: 200,
    display: 'flex',
    justifyContent: isSheet ? 'flex-end' : 'center',
    alignItems: isBottom ? 'flex-end' : isSheet ? 'stretch' : 'center',
    padding: isCenter ? 16 : 0,
    animation: 'shifty-modal-fade 160ms ease-out',
  };

  // Panel
  const panelBase = {
    background: 'var(--surface)',
    color: 'var(--ink)',
    border: '1px solid var(--border)',
    boxShadow: '0 12px 40px rgba(0,0,0,0.18), 0 2px 6px rgba(0,0,0,0.08)',
    display: 'flex', flexDirection: 'column',
    maxHeight: isBottom ? '92vh' : isSheet ? '100vh' : 'min(86vh, 720px)',
    overflow: 'hidden',
  };
  const panelStyle = isCenter ? {
    ...panelBase,
    width: 'min(' + width + 'px, 100%)',
    borderRadius: 'var(--r-lg)',
    animation: 'shifty-modal-pop 180ms cubic-bezier(.2,.8,.2,1)',
  } : isSheet ? {
    ...panelBase,
    width: 'min(' + (width + 60) + 'px, 100%)',
    height: '100vh',
    borderRadius: 0,
    borderRight: 'none', borderTop: 'none', borderBottom: 'none',
    animation: 'shifty-modal-slide-right 220ms cubic-bezier(.2,.8,.2,1)',
  } : { // bottom
    ...panelBase,
    width: '100%',
    borderRadius: 'var(--r-lg) var(--r-lg) 0 0',
    borderBottom: 'none',
    animation: 'shifty-modal-slide-up 220ms cubic-bezier(.2,.8,.2,1)',
  };

  return (
    <div style={backdropStyle} onClick={onClose} role="presentation">
      <div
        style={panelStyle}
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="shifty-modal-title"
      >
        {/* Drag-handle on bottom-sheet */}
        {isBottom && (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '8px 0 0' }}>
            <div style={{ width: 36, height: 4, borderRadius: 999, background: 'var(--border-strong)' }} />
          </div>
        )}

        {/* Header */}
        <div style={{
          display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between',
          padding: isBottom ? '8px 18px 0' : '16px 18px 0',
          gap: 12,
        }}>
          <div style={{ minWidth: 0 }}>
            <h3 id="shifty-modal-title" style={{ margin: 0, fontSize: 16, fontWeight: 700, letterSpacing: '-0.01em' }}>
              {title}
            </h3>
            {subtitle && (
              <div style={{ fontSize: 12, color: 'var(--ink-muted)', marginTop: 2 }}>{subtitle}</div>
            )}
          </div>
          <button
            onClick={onClose}
            aria-label="Schließen"
            style={{
              width: 28, height: 28, borderRadius: 'var(--r-md)',
              border: '1px solid transparent', background: 'transparent',
              color: 'var(--ink-muted)', cursor: 'pointer',
              display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
              fontSize: 18, lineHeight: 1, flexShrink: 0,
            }}
            onMouseEnter={(e) => { e.currentTarget.style.background = 'var(--surface-alt)'; e.currentTarget.style.color = 'var(--ink)'; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.color = 'var(--ink-muted)'; }}
          >×</button>
        </div>

        {/* Body */}
        <div style={{ padding: '14px 18px 16px', overflowY: 'auto', flex: 1 }}>
          {children}
        </div>

        {/* Footer */}
        {footer && (
          <div style={{
            display: 'flex', justifyContent: 'flex-end', gap: 8,
            padding: '12px 18px',
            borderTop: '1px solid var(--border)',
            background: 'var(--surface-alt)',
          }}>
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Form bits ───────────────────────────────────────────────
function Field({ label, hint, error, children, span }) {
  return (
    <label style={{ display: 'flex', flexDirection: 'column', gap: 4, gridColumn: span === 2 ? 'span 2' : 'auto', minWidth: 0 }}>
      <span style={{ fontSize: 11, fontWeight: 600, color: 'var(--ink-soft)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
        {label}
      </span>
      {children}
      {hint && !error && <span style={{ fontSize: 11, color: 'var(--ink-muted)' }}>{hint}</span>}
      {error && <span style={{ fontSize: 11, color: 'var(--bad)' }}>{error}</span>}
    </label>
  );
}

const inputStyle = {
  height: 34, padding: '0 10px',
  border: '1px solid var(--border-strong)', borderRadius: 'var(--r-md)',
  background: 'var(--surface)', color: 'var(--ink)',
  fontSize: 13, fontFamily: 'inherit', minWidth: 0, width: '100%',
  outline: 'none',
};

function TextInput(props) {
  return <input {...props} style={{ ...inputStyle, ...(props.style || {}) }}
    onFocus={(e) => { e.target.style.borderColor = 'var(--accent)'; e.target.style.boxShadow = '0 0 0 3px var(--accent-soft)'; props.onFocus && props.onFocus(e); }}
    onBlur={(e) => { e.target.style.borderColor = 'var(--border-strong)'; e.target.style.boxShadow = 'none'; props.onBlur && props.onBlur(e); }}
  />;
}
function SelectInput({ children, ...props }) {
  return <select {...props} style={{ ...inputStyle, paddingRight: 28, appearance: 'none',
    backgroundImage: "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path d='M1 1l4 4 4-4' stroke='%236b7382' stroke-width='1.5' fill='none' stroke-linecap='round'/></svg>\")",
    backgroundRepeat: 'no-repeat', backgroundPosition: 'right 10px center',
    ...(props.style || {}) }}
    onFocus={(e) => { e.target.style.borderColor = 'var(--accent)'; e.target.style.boxShadow = '0 0 0 3px var(--accent-soft)'; }}
    onBlur={(e) => { e.target.style.borderColor = 'var(--border-strong)'; e.target.style.boxShadow = 'none'; }}
  >{children}</select>;
}
function TextareaInput(props) {
  return <textarea {...props} rows={props.rows || 3} style={{
    ...inputStyle, height: 'auto', padding: '8px 10px', resize: 'vertical', lineHeight: 1.45, ...(props.style || {})
  }}
    onFocus={(e) => { e.target.style.borderColor = 'var(--accent)'; e.target.style.boxShadow = '0 0 0 3px var(--accent-soft)'; }}
    onBlur={(e) => { e.target.style.borderColor = 'var(--border-strong)'; e.target.style.boxShadow = 'none'; }}
  />;
}

// ─── Konkrete Dialoge ─────────────────────────────────────────

// Datum-Helfer: konvertiert zwischen "TT.MM.JJJJ" (App-Konvention) und "YYYY-MM-DD" (input[type=date]).
function toIso(de) {
  if (!de) return '';
  const m = /^(\d{2})\.(\d{2})\.(\d{4})$/.exec(de);
  if (m) return `${m[3]}-${m[2]}-${m[1]}`;
  if (/^\d{4}-\d{2}-\d{2}$/.test(de)) return de;
  return '';
}
function fromIso(iso) {
  if (!iso) return '';
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  return m ? `${m[3]}.${m[2]}.${m[1]}` : iso;
}

function ContractModal({ open, onClose, variant, initial }) {
  const isEdit = !!initial;
  const [from, setFrom]       = useStateM(initial?.from || '01.01.2026');
  const [to, setTo]           = useStateM(initial?.to || '31.12.2026');
  const [hours, setHours]     = useStateM(initial?.hours ?? 38);
  const [days, setDays]       = useStateM(initial?.days ?? 5);
  const [vacation, setVac]    = useStateM(initial?.vacation ?? 28);
  const [weekdays, setWeekdays] = useStateM(initial?.weekdays || ['Mo', 'Di', 'Mi', 'Do', 'Fr']);

  // Abgeleitet (vgl. employee_work_details.rs::vacation_day_in_hours): expected_hours / workdays_per_week.
  const hoursPerVacationDay = days > 0 ? (hours / days) : 0;
  const isOpenEnd = to === '31.12.2050';

  const allDays = ['Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa', 'So'];
  const toggleDay = (d) => setWeekdays((ws) => ws.includes(d) ? ws.filter((x) => x !== d) : [...ws, d]);

  return (
    <Modal
      open={open} onClose={onClose} variant={variant}
      title={isEdit ? 'Arbeitsvertrag bearbeiten' : 'Arbeitsvertrag hinzufügen'}
      subtitle="Zeitraum, Stunden und Urlaubsanspruch festlegen."
      width={500}
      footer={<>
        {isEdit && <Btn kind="danger" onClick={onClose}>Löschen</Btn>}
        <span style={{ flex: 1 }} />
        <Btn kind="ghost" onClick={onClose}>Abbrechen</Btn>
        <Btn kind="primary" onClick={onClose}>{isEdit ? 'Speichern' : 'Hinzufügen'}</Btn>
      </>}
    >
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <Field label="Gültig ab">
          <TextInput type="date" value={toIso(from)} onChange={(e) => setFrom(fromIso(e.target.value))} />
        </Field>
        <Field label="Gültig bis" hint={isOpenEnd ? 'unbefristet (31.12.2050)' : null}>
          <TextInput type="date" value={toIso(to)} onChange={(e) => setTo(fromIso(e.target.value))} />
        </Field>
        <button type="button" onClick={() => setTo('31.12.2050')}
          style={{
            gridColumn: 'span 2', justifySelf: 'start', marginTop: -4,
            background: 'transparent', border: 'none', padding: 0,
            color: 'var(--accent)', fontSize: 12, fontWeight: 500,
            cursor: 'pointer', textDecoration: 'underline',
          }}>
          Unbefristet setzen (31.12.2050)
        </button>

        <Field label="Stunden / Woche">
          <TextInput type="number" step="0.5" min="0" max="60" value={hours} onChange={(e) => setHours(parseFloat(e.target.value) || 0)} />
        </Field>
        <Field label="Tage / Woche">
          <TextInput type="number" step="1" min="1" max="7" value={days} onChange={(e) => setDays(parseInt(e.target.value, 10) || 0)} />
        </Field>

        <Field label="Stunden / Urlaubstag" span={2} hint="Automatisch berechnet aus Stunden / Tage">
          <div style={{
            height: 36, padding: '0 12px', display: 'flex', alignItems: 'center',
            border: '1px dashed var(--border-strong)', borderRadius: 'var(--r-md)',
            background: 'var(--surface-2)', color: 'var(--ink-soft)',
            fontSize: 14, fontVariantNumeric: 'tabular-nums',
          }}>
            {hoursPerVacationDay.toFixed(2)} h
          </div>
        </Field>

        <Field label="Wochentage" span={2} hint={weekdays.length === 0 ? 'Mindestens einen Tag wählen' : null}>
          <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
            {allDays.map((d) => {
              const active = weekdays.includes(d);
              const weekend = d === 'Sa' || d === 'So';
              return (
                <button key={d} type="button" onClick={() => toggleDay(d)}
                  style={{
                    minWidth: 38, height: 32, padding: '0 8px',
                    border: '1px solid ' + (active ? 'var(--accent)' : 'var(--border-strong)'),
                    background: active ? 'var(--accent)' : 'var(--surface)',
                    color: active ? 'var(--accent-ink)' : (weekend ? 'var(--ink-muted)' : 'var(--ink)'),
                    borderRadius: 'var(--r-md)',
                    fontSize: 12, fontWeight: active ? 600 : 500,
                    cursor: 'pointer',
                  }}>
                  {d}
                </button>
              );
            })}
          </div>
        </Field>

        <Field label="Urlaubstage / Jahr" span={2} hint="Gesetzlicher Mindesturlaub: 20 Tage bei 5-Tage-Woche.">
          <TextInput type="number" step="1" min="0" max="40" value={vacation} onChange={(e) => setVac(parseInt(e.target.value, 10) || 0)} />
        </Field>

        {/* Live preview */}
        <div style={{ gridColumn: 'span 2', marginTop: 4, padding: 10, background: 'var(--surface-alt)', border: '1px solid var(--border)', borderRadius: 'var(--r-md)' }}>
          <div style={{ fontSize: 11, color: 'var(--ink-muted)', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: 4 }}>Vorschau</div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div className="mono" style={{ fontSize: 13, fontWeight: 600 }}>{from} – {isOpenEnd ? 'unbefristet' : to}</div>
              <div style={{ fontSize: 11, color: 'var(--ink-muted)' }}>
                {weekdays.length > 0 ? weekdays.join(' · ') : '— keine Tage —'} · {vacation} Urlaubstage
              </div>
            </div>
            <div className="mono" style={{ fontSize: 14, fontWeight: 700 }}>{Number(hours).toFixed(1)}h</div>
          </div>
        </div>
      </div>
    </Modal>
  );
}

function ExtraHoursModal({ open, onClose, variant }) {
  const [date, setDate]     = useStateM('17.04.2026');
  const [cat, setCat]       = useStateM('Urlaub');
  const [hours, setHours]   = useStateM(8);
  const [note, setNote]     = useStateM('');

  const cats = [
    { id: 'Urlaub',       color: 'var(--good)',   soft: 'var(--good-soft)' },
    { id: 'Krank',        color: 'var(--bad)',    soft: 'var(--bad-soft)' },
    { id: 'Zusatzarbeit', color: 'var(--accent)', soft: 'var(--accent-soft)' },
    { id: 'Feiertage',    color: 'var(--warn)',   soft: 'var(--warn-soft)' },
    { id: 'Unbezahlt',    color: 'var(--ink-muted)', soft: 'var(--surface-2)' },
  ];

  return (
    <Modal
      open={open} onClose={onClose} variant={variant}
      title="Sonstige Stunden hinzufügen"
      subtitle="Urlaub, Krankheit, Zusatzarbeit oder Feiertage eintragen."
      width={460}
      footer={<>
        <Btn kind="ghost" onClick={onClose}>Abbrechen</Btn>
        <Btn kind="primary" onClick={onClose}>Hinzufügen</Btn>
      </>}
    >
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <Field label="Datum">
          <TextInput type="date" value={toIso(date)} onChange={(e) => setDate(fromIso(e.target.value))} />
        </Field>
        <Field label="Stunden">
          <TextInput type="number" step="0.5" min="0" max="24" value={hours} onChange={(e) => setHours(parseFloat(e.target.value) || 0)} />
        </Field>

        <Field label="Kategorie" span={2}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {cats.map((c) => {
              const active = cat === c.id;
              return (
                <button key={c.id} onClick={() => setCat(c.id)} type="button"
                  style={{
                    padding: '6px 10px', borderRadius: 999,
                    border: '1px solid ' + (active ? c.color : 'var(--border-strong)'),
                    background: active ? c.soft : 'var(--surface)',
                    color: active ? c.color : 'var(--ink-soft)',
                    fontSize: 12, fontWeight: active ? 600 : 500, cursor: 'pointer',
                    display: 'inline-flex', alignItems: 'center', gap: 6,
                  }}>
                  <span style={{ width: 8, height: 8, borderRadius: '50%', background: c.color, display: 'inline-block' }} />
                  {c.id}
                </button>
              );
            })}
          </div>
        </Field>

        <Field label="Notiz (optional)" span={2}>
          <TextareaInput value={note} onChange={(e) => setNote(e.target.value)} placeholder={cat === 'Urlaub' ? 'z.B. Karfreitag' : cat === 'Zusatzarbeit' ? 'z.B. Inventur' : ''} />
        </Field>
      </div>
    </Modal>
  );
}

// Modal-style chooser (used by tweaks panel)
function ModalVariantTweak({ value, onChange }) {
  const opts = [
    { id: 'auto',   label: 'Auto',     hint: 'Bottom-Sheet auf Mobile' },
    { id: 'center', label: 'Zentriert', hint: 'Klassischer Dialog' },
    { id: 'sheet',  label: 'Side-Sheet', hint: 'Von rechts' },
    { id: 'bottom', label: 'Bottom',   hint: 'Immer von unten' },
  ];
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 6 }}>
      {opts.map((o) => {
        const active = value === o.id;
        return (
          <button key={o.id} onClick={() => onChange(o.id)} type="button"
            style={{
              textAlign: 'left', padding: '8px 10px',
              border: '1px solid ' + (active ? 'var(--accent)' : 'var(--border-strong)'),
              background: active ? 'var(--accent-soft)' : 'var(--surface)',
              color: 'var(--ink)', borderRadius: 'var(--r-md)', cursor: 'pointer',
            }}>
            <div style={{ fontSize: 12, fontWeight: 600 }}>{o.label}</div>
            <div style={{ fontSize: 11, color: 'var(--ink-muted)' }}>{o.hint}</div>
          </button>
        );
      })}
    </div>
  );
}

Object.assign(window, { Modal, Field, TextInput, SelectInput, TextareaInput, ContractModal, ExtraHoursModal, ModalVariantTweak });
