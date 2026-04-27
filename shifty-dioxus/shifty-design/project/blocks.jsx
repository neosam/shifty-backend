// blocks.jsx — reusable demo blocks rendered inside each direction's artboards.
// Pure presentational: takes a `t` (token bag) prop with merged colors+type+radius.
// Window.* exports at bottom for cross-script access.

const { useState } = React;

// ─── Primitives ─────────────────────────────────────────────
function Swatch({ color, name, value, ink, large }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6, minWidth: 0 }}>
      <div
        style={{
          background: color,
          height: large ? 56 : 40,
          borderRadius: 6,
          border: '1px solid rgba(0,0,0,0.06)',
        }}
      />
      <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: ink }}>{name}</span>
        <span style={{ fontSize: 10, fontFamily: 'ui-monospace, Menlo, monospace', color: ink, opacity: 0.6 }}>
          {value}
        </span>
      </div>
    </div>
  );
}

function SectionHeader({ t, eyebrow, title, sub }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 4, marginBottom: 14 }}>
      {eyebrow && (
        <span style={{
          fontSize: t.typeScale.micro.size,
          letterSpacing: t.typeScale.micro.tracking,
          fontWeight: t.typeScale.micro.weight,
          textTransform: 'uppercase',
          color: t.colors.inkMuted,
        }}>{eyebrow}</span>
      )}
      <h3 style={{
        margin: 0,
        fontSize: t.typeScale.h2.size,
        lineHeight: t.typeScale.h2.line,
        fontWeight: t.typeScale.h2.weight,
        letterSpacing: t.typeScale.h2.tracking,
        fontFamily: t.typeScale.h2.family === 'display' ? t.fonts.display
          : t.typeScale.h2.family === 'mono' ? t.fonts.mono : t.fonts.sans,
        color: t.colors.ink,
      }}>{title}</h3>
      {sub && (
        <p style={{ margin: 0, fontSize: 12, color: t.colors.inkMuted, lineHeight: 1.5 }}>{sub}</p>
      )}
    </div>
  );
}

// ─── Blocks ─────────────────────────────────────────────────

function ColorsBlock({ t }) {
  const c = t.colors;
  return (
    <div>
      <SectionHeader t={t} eyebrow="01 / Color" title="Surfaces, ink, and accent" sub="Tuned for low chroma in surfaces, single saturated accent." />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: 10 }}>
        <Swatch large color={c.bg} name="bg" value={c.bg} ink={c.ink} />
        <Swatch large color={c.surface} name="surface" value={c.surface} ink={c.ink} />
        <Swatch large color={c.surfaceAlt} name="surface-alt" value={c.surfaceAlt} ink={c.ink} />
        <Swatch large color={c.border} name="border" value={c.border} ink={c.ink} />
        <Swatch large color={c.ink} name="ink" value={c.ink} ink={c.ink} />
        <Swatch large color={c.accent} name="accent" value={c.accent} ink={c.ink} />
      </div>
      <div style={{ height: 12 }} />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: 10 }}>
        <Swatch color={c.good} name="good" value={c.good} ink={c.ink} />
        <Swatch color={c.goodSoft} name="good-soft" value={c.goodSoft} ink={c.ink} />
        <Swatch color={c.warn} name="warn (missing)" value={c.warn} ink={c.ink} />
        <Swatch color={c.warnSoft} name="warn-soft" value={c.warnSoft} ink={c.ink} />
        <Swatch color={c.bad} name="bad (blocked)" value={c.bad} ink={c.ink} />
        <Swatch color={c.badSoft} name="bad-soft" value={c.badSoft} ink={c.ink} />
      </div>
      <div style={{ height: 12 }} />
      <div>
        <span style={{ fontSize: 11, fontWeight: 600, color: c.inkMuted, letterSpacing: '0.04em', textTransform: 'uppercase' }}>
          Person fills · used in week view to tag individuals
        </span>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: 10, marginTop: 8 }}>
          {['slotA','slotB','slotC','slotD','slotE','slotF'].map((k, i) => (
            <Swatch key={k} color={c[k]} name={`person-${i+1}`} value={c[k]} ink={c.ink} />
          ))}
        </div>
      </div>
    </div>
  );
}

function TypeBlock({ t }) {
  const samples = [
    { key: 'display', label: 'Display · 38/42', text: 'Schichtplanung, klar.' },
    { key: 'h1', label: 'H1 · Page title', text: 'Mitarbeiter · April 2026' },
    { key: 'h2', label: 'H2 · Section', text: 'Kalenderwoche 17' },
    { key: 'body', label: 'Body · Default', text: 'Astrid hat 38,5h gebucht — 1,5h fehlen.' },
    { key: 'small', label: 'Small · Captions', text: 'Letzte Änderung vor 4 Minuten' },
    { key: 'micro', label: 'Micro · Eyebrows', text: 'WORKING HOURS · WEEK 17' },
  ];
  return (
    <div>
      <SectionHeader t={t} eyebrow="02 / Type" title={`Type pairing · ${t.fonts.display.split(',')[0].replace(/"/g,'')} + ${t.fonts.sans.split(',')[0].replace(/"/g,'')}`} sub="Display for hierarchy, sans for chrome, mono for numerics." />
      <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        {samples.map((s) => {
          const ts = t.typeScale[s.key];
          const fam = ts.family === 'display' ? t.fonts.display : ts.family === 'mono' ? t.fonts.mono : t.fonts.sans;
          return (
            <div key={s.key} style={{ display: 'grid', gridTemplateColumns: '140px 1fr', gap: 16, alignItems: 'baseline', borderBottom: `1px dashed ${t.colors.border}`, paddingBottom: 10 }}>
              <div style={{ fontSize: 11, fontWeight: 600, color: t.colors.inkMuted, letterSpacing: '0.04em', textTransform: 'uppercase' }}>{s.label}</div>
              <div style={{
                fontFamily: fam,
                fontSize: ts.size,
                lineHeight: ts.line,
                fontWeight: ts.weight,
                letterSpacing: ts.tracking,
                color: t.colors.ink,
                textTransform: s.key === 'micro' ? 'uppercase' : 'none',
              }}>{s.text}</div>
            </div>
          );
        })}
      </div>
      <div style={{ marginTop: 16, padding: 12, background: t.colors.surfaceAlt, borderRadius: t.radius.md, fontFamily: t.fonts.mono, fontSize: 12, color: t.colors.inkSoft, fontVariantNumeric: 'tabular-nums' }}>
        <div>09:00 — 13:00 · 4,0h</div>
        <div>13:30 — 17:00 · 3,5h</div>
        <div style={{ fontWeight: 600, color: t.colors.ink, marginTop: 4 }}>Σ 38,5h / 40,0h</div>
      </div>
    </div>
  );
}

function ButtonsBlock({ t }) {
  const Btn = ({ kind, children, icon }) => {
    const base = {
      fontFamily: t.fonts.sans,
      fontSize: 13,
      fontWeight: 500,
      padding: '8px 14px',
      borderRadius: t.radius.md,
      border: '1px solid transparent',
      cursor: 'pointer',
      display: 'inline-flex',
      alignItems: 'center',
      gap: 6,
      transition: 'all 0.12s',
    };
    const styles = {
      primary: { ...base, background: t.colors.accent, color: t.colors.accentInk, borderColor: t.colors.accent },
      secondary: { ...base, background: t.colors.surface, color: t.colors.ink, borderColor: t.colors.borderStrong },
      ghost: { ...base, background: 'transparent', color: t.colors.inkSoft, borderColor: 'transparent' },
      danger: { ...base, background: t.colors.surface, color: t.colors.bad, borderColor: t.colors.bad },
    };
    return <button style={styles[kind]}>{icon && <span style={{ fontFamily: t.fonts.mono, fontSize: 14, lineHeight: 1 }}>{icon}</span>}{children}</button>;
  };

  return (
    <div>
      <SectionHeader t={t} eyebrow="03 / Buttons" title="Actions" sub="Primary for the one main action per surface. Secondary for everything else." />
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginBottom: 12 }}>
        <Btn kind="primary">Schicht speichern</Btn>
        <Btn kind="secondary">Abbrechen</Btn>
        <Btn kind="ghost">Zurück</Btn>
        <Btn kind="danger">Löschen</Btn>
      </div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
        <Btn kind="primary" icon="+">Mitarbeiter hinzufügen</Btn>
        <Btn kind="secondary" icon="↓">Export</Btn>
        <Btn kind="secondary" icon="‹">Vorwoche</Btn>
        <Btn kind="secondary" icon="›">Nächste Woche</Btn>
      </div>
    </div>
  );
}

function FormBlock({ t }) {
  const inp = {
    fontFamily: t.fonts.sans,
    fontSize: 13,
    padding: '7px 10px',
    borderRadius: t.radius.sm,
    border: `1px solid ${t.colors.border}`,
    background: t.colors.surface,
    color: t.colors.ink,
    width: '100%',
    boxSizing: 'border-box',
  };
  const lbl = { fontSize: 11, fontWeight: 600, color: t.colors.inkMuted, letterSpacing: '0.02em', textTransform: 'uppercase' };
  return (
    <div>
      <SectionHeader t={t} eyebrow="04 / Form" title="Inputs" />
      <div style={{ display: 'grid', gap: 12, gridTemplateColumns: '1fr 1fr' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={lbl}>Name</span>
          <input style={inp} defaultValue="Astrid Bauer" />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={lbl}>Rolle</span>
          <select style={inp}><option>Vollzeit</option><option>Teilzeit</option><option>Aushilfe</option></select>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={lbl}>Datum</span>
          <input style={{ ...inp, fontFamily: t.fonts.mono, fontVariantNumeric: 'tabular-nums' }} defaultValue="2026-04-20" />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={lbl}>Stunden</span>
          <input style={{ ...inp, fontFamily: t.fonts.mono, fontVariantNumeric: 'tabular-nums' }} defaultValue="38,5" />
        </div>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 10, fontSize: 13, color: t.colors.inkSoft }}>
        <span style={{ width: 16, height: 16, borderRadius: 3, border: `1.5px solid ${t.colors.accent}`, background: t.colors.accent, display: 'inline-flex', alignItems: 'center', justifyContent: 'center', color: t.colors.accentInk, fontSize: 11, fontWeight: 700 }}>✓</span>
        Bezahlte Stelle
      </div>
    </div>
  );
}

function PillsBlock({ t }) {
  const Pill = ({ bg, text, label, soft }) => (
    <span style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: 6,
      padding: '3px 10px',
      borderRadius: 999,
      background: soft ? bg : 'transparent',
      border: soft ? 'none' : `1px solid ${bg}`,
      color: soft ? text : bg,
      fontSize: 11,
      fontFamily: t.fonts.sans,
      fontWeight: 600,
      letterSpacing: '0.02em',
    }}>
      <span style={{ width: 6, height: 6, borderRadius: '50%', background: soft ? text : bg }} />
      {label}
    </span>
  );

  // Person chip — like the colored name tags in the current UI
  const Person = ({ fill, name, conflict }) => (
    <span style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: 4,
      padding: '2px 7px',
      borderRadius: t.radius.sm,
      background: fill,
      color: t.colors.ink,
      fontSize: 12,
      fontFamily: t.fonts.sans,
      fontWeight: 500,
      lineHeight: '18px',
      border: conflict ? `1.5px solid ${t.colors.bad}` : 'none',
    }}>
      {name}{conflict && <span style={{ color: t.colors.bad, fontWeight: 700 }}>*</span>}
    </span>
  );

  return (
    <div>
      <SectionHeader t={t} eyebrow="05 / Status" title="Pills & person chips" sub="Person chips are the load-bearing element of the week view." />
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginBottom: 14 }}>
        <Pill soft bg={t.colors.goodSoft} text={t.colors.good} label="Bestätigt" />
        <Pill soft bg={t.colors.warnSoft} text={t.colors.warn} label="Schicht offen" />
        <Pill soft bg={t.colors.badSoft} text={t.colors.bad} label="Konflikt" />
        <Pill bg={t.colors.accent} text={t.colors.accent} label="In Bearbeitung" />
      </div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
        <Person fill={t.colors.slotA} name="Astrid" />
        <Person fill={t.colors.slotB} name="Stephan" />
        <Person fill={t.colors.slotC} name="Sonja" conflict />
        <Person fill={t.colors.slotD} name="Franzi" />
        <Person fill={t.colors.slotE} name="Anina" conflict />
        <Person fill={t.colors.slotF} name="Ruben" />
      </div>
    </div>
  );
}

function SlotCardBlock({ t }) {
  const Slot = ({ filled, total, people, time, missing, blocked }) => (
    <div style={{
      border: `1px solid ${blocked ? t.colors.bad : missing ? t.colors.warn : t.colors.border}`,
      background: blocked ? t.colors.badSoft : missing ? t.colors.warnSoft : t.colors.surface,
      borderRadius: t.radius.md,
      padding: '8px 10px',
      display: 'flex',
      flexDirection: 'column',
      gap: 6,
      minWidth: 0,
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontFamily: t.fonts.mono, fontSize: 11, fontVariantNumeric: 'tabular-nums', color: t.colors.inkMuted }}>{time}</span>
        <span style={{
          fontFamily: t.fonts.mono,
          fontSize: 11,
          fontWeight: 700,
          color: filled === total ? t.colors.good : missing ? t.colors.warn : t.colors.inkSoft,
        }}>{filled}/{total}</span>
      </div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
        {people.map((p, i) => (
          <span key={i} style={{
            padding: '1px 6px', borderRadius: t.radius.sm, background: p.fill, fontSize: 11, color: t.colors.ink, fontWeight: 500,
          }}>{p.name}{p.conflict && <span style={{ color: t.colors.bad, fontWeight: 700 }}>*</span>}</span>
        ))}
      </div>
    </div>
  );
  return (
    <div>
      <SectionHeader t={t} eyebrow="06 / Slot cards" title="Shift cells" sub="Three states: filled, missing, blocked. Border + tint communicate state." />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 10 }}>
        <Slot time="09:00–10:00" filled={2} total={2} people={[{ name:'Astrid', fill:t.colors.slotA }, { name:'Stephan', fill:t.colors.slotB }]} />
        <Slot missing time="11:00–12:00" filled={1} total={3} people={[{ name:'Astrid', fill:t.colors.slotA }]} />
        <Slot blocked time="14:00–15:00" filled={2} total={2} people={[{ name:'Sonja', fill:t.colors.slotC, conflict:true }, { name:'Anina', fill:t.colors.slotE, conflict:true }]} />
      </div>
    </div>
  );
}

function NavBlock({ t }) {
  const items = ['Schichtplan', 'Meine Schichten', 'Jahresübersicht', 'Mitarbeiter', 'Benutzerverwaltung'];
  const [active, setActive] = useState(0);
  return (
    <div>
      <SectionHeader t={t} eyebrow="07 / Navigation" title="Top bar" />
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 4,
        padding: '10px 14px',
        background: t.colors.surface,
        borderBottom: `1px solid ${t.colors.border}`,
        borderRadius: t.radius.md,
      }}>
        <span style={{
          fontFamily: t.fonts.display,
          fontSize: 18,
          fontWeight: 700,
          color: t.colors.ink,
          marginRight: 16,
          letterSpacing: '-0.01em',
        }}>
          Shifty<span style={{ color: t.colors.accent }}>.</span>
        </span>
        {items.map((it, i) => (
          <button key={it} onClick={() => setActive(i)} style={{
            background: active === i ? t.colors.accentSoft : 'transparent',
            border: 'none',
            color: active === i ? t.colors.accent : t.colors.inkSoft,
            fontSize: 13,
            fontWeight: active === i ? 600 : 500,
            padding: '5px 10px',
            borderRadius: t.radius.sm,
            cursor: 'pointer',
            fontFamily: t.fonts.sans,
          }}>{it}</button>
        ))}
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 12, color: t.colors.inkMuted, fontFamily: t.fonts.sans }}>simon</span>
        <span style={{ width: 24, height: 24, borderRadius: '50%', background: t.colors.slotE, display: 'inline-flex', alignItems: 'center', justifyContent: 'center', fontSize: 11, fontWeight: 700, color: t.colors.ink }}>S</span>
      </div>
    </div>
  );
}

Object.assign(window, {
  ColorsBlock, TypeBlock, ButtonsBlock, FormBlock, PillsBlock, SlotCardBlock, NavBlock, SectionHeader,
});
