// week-preview.jsx — a faithful, compact week-view sample for each direction.
// Shows the load-bearing screen of the app in the system's tokens.

const WEEK_HOURS = ['09:00', '10:00', '11:00', '12:00', '13:00', '14:00', '15:00'];
const WEEK_DAYS = [
  { name: 'Mo', date: '20.04', total: '8,0h' },
  { name: 'Di', date: '21.04', total: '8,0h' },
  { name: 'Mi', date: '22.04', total: '8,0h' },
  { name: 'Do', date: '23.04', total: '8,0h' },
  { name: 'Fr', date: '24.04', total: '7,5h' },
  { name: 'Sa', date: '25.04', total: '6,0h' },
];

// Pseudo-data: per-day list of slot rows (which person fills what)
const SLOTS = {
  0: [{ p: 'Astrid', s: 'A' }, { p: 'Stephan', s: 'B' }],
  1: [{ p: 'Siglinde*', s: 'C' }, { p: 'Sonja', s: 'D' }],
  2: [{ p: 'Astrid', s: 'A' }, { p: 'Sonja', s: 'D' }],
  3: [{ p: 'Ruben*', s: 'F' }, { p: 'Sonja', s: 'D' }],
  4: [{ p: 'Anina*', s: 'E' }, { p: 'Dany', s: 'B' }, { p: 'Sonja', s: 'D' }],
  5: [{ p: 'Anina*', s: 'E' }, { p: 'Franzi', s: 'C' }],
};

function WeekPreview({ t }) {
  const slotMap = (s) => ({
    A: t.colors.slotA, B: t.colors.slotB, C: t.colors.slotC,
    D: t.colors.slotD, E: t.colors.slotE, F: t.colors.slotF,
  }[s]);

  return (
    <div style={{
      background: t.colors.surface,
      border: `1px solid ${t.colors.border}`,
      borderRadius: t.radius.lg,
      overflow: 'hidden',
      fontFamily: t.fonts.sans,
    }}>
      {/* toolbar */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '10px 14px', borderBottom: `1px solid ${t.colors.border}`,
        background: t.colors.surfaceAlt,
      }}>
        <button style={{
          width: 26, height: 26, borderRadius: t.radius.sm,
          border: `1px solid ${t.colors.borderStrong}`, background: t.colors.surface,
          color: t.colors.ink, cursor: 'pointer', fontSize: 13,
        }}>‹</button>
        <span style={{
          fontFamily: t.fonts.mono, fontSize: 12, color: t.colors.ink, fontWeight: 600,
          fontVariantNumeric: 'tabular-nums',
        }}>KW 17 / 2026 · vom 20.04</span>
        <button style={{
          width: 26, height: 26, borderRadius: t.radius.sm,
          border: `1px solid ${t.colors.borderStrong}`, background: t.colors.surface,
          color: t.colors.ink, cursor: 'pointer', fontSize: 13,
        }}>›</button>
        <span style={{
          marginLeft: 8, padding: '4px 10px', borderRadius: t.radius.sm,
          background: t.colors.accent, color: t.colors.accentInk,
          fontSize: 12, fontWeight: 600,
        }}>Woche</span>
        <span style={{
          padding: '4px 10px', borderRadius: t.radius.sm,
          color: t.colors.inkSoft, fontSize: 12, fontWeight: 500,
        }}>Tag</span>
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 11, color: t.colors.inkMuted }}>Du bearbeitest:</span>
        <span style={{
          padding: '3px 8px', borderRadius: t.radius.sm,
          background: t.colors.surface, border: `1px solid ${t.colors.border}`,
          fontSize: 12, color: t.colors.ink, fontWeight: 500,
        }}>Simon ⌄</span>
      </div>

      {/* tabs */}
      <div style={{
        display: 'flex', gap: 0, padding: '0 14px',
        borderBottom: `1px solid ${t.colors.border}`,
        background: t.colors.surface,
      }}>
        {['Laden', 'Backen', 'Reinigung'].map((tab, i) => (
          <button key={tab} style={{
            background: 'transparent', border: 'none',
            padding: '8px 14px', cursor: 'pointer',
            fontSize: 12, fontWeight: i === 0 ? 600 : 500,
            color: i === 0 ? t.colors.accent : t.colors.inkMuted,
            borderBottom: `2px solid ${i === 0 ? t.colors.accent : 'transparent'}`,
            marginBottom: -1, fontFamily: t.fonts.sans,
          }}>{tab}</button>
        ))}
      </div>

      {/* grid */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '60px repeat(6, 1fr)',
        background: t.colors.surface,
      }}>
        <div /> {/* corner */}
        {WEEK_DAYS.map((d, i) => (
          <div key={i} style={{
            padding: '8px 10px',
            borderLeft: `1px solid ${t.colors.border}`,
            borderBottom: `1px solid ${t.colors.border}`,
            background: t.colors.surfaceAlt,
          }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: t.colors.ink }}>{d.name}, {d.date}</div>
            <div style={{
              fontSize: 10, color: t.colors.inkMuted, fontFamily: t.fonts.mono,
              fontVariantNumeric: 'tabular-nums', marginTop: 2,
            }}>{d.total}</div>
          </div>
        ))}

        {WEEK_HOURS.map((h, hi) => {
          const isHighlight = hi === 4; // a missing row
          return (
            <React.Fragment key={h}>
              <div style={{
                padding: '8px 6px', fontFamily: t.fonts.mono, fontSize: 10,
                color: t.colors.inkMuted, borderBottom: `1px solid ${t.colors.border}`,
                fontVariantNumeric: 'tabular-nums', background: t.colors.surface,
              }}>{h}</div>
              {WEEK_DAYS.map((_, di) => {
                const people = SLOTS[di] || [];
                const need = di === 4 && hi === 3; // missing
                const blocked = di === 5 && hi === 5; // blocked
                return (
                  <div key={di} style={{
                    padding: 6,
                    borderLeft: `1px solid ${t.colors.border}`,
                    borderBottom: `1px solid ${t.colors.border}`,
                    background: blocked ? t.colors.badSoft : need ? t.colors.warnSoft : t.colors.surface,
                    minHeight: 38,
                    display: 'flex', alignItems: 'flex-start',
                    gap: 4, flexWrap: 'wrap',
                  }}>
                    <span style={{
                      fontFamily: t.fonts.mono, fontSize: 9, fontWeight: 700,
                      color: need ? t.colors.warn : blocked ? t.colors.bad : t.colors.inkMuted,
                      lineHeight: '18px',
                    }}>{people.length}/{need ? 3 : people.length}</span>
                    {people.map((p, i) => (
                      <span key={i} style={{
                        padding: '0 6px', borderRadius: t.radius.sm,
                        background: slotMap(p.s),
                        fontSize: 10, color: t.colors.ink, fontWeight: 500, lineHeight: '18px',
                      }}>{p.p}</span>
                    ))}
                  </div>
                );
              })}
            </React.Fragment>
          );
        })}
      </div>
    </div>
  );
}

function ModalPreview({ t }) {
  return (
    <div style={{
      background: t.colors.bg,
      border: `1px solid ${t.colors.border}`,
      borderRadius: t.radius.lg,
      padding: 20,
      position: 'relative',
    }}>
      <div style={{
        background: t.colors.surface,
        border: `1px solid ${t.colors.border}`,
        borderRadius: t.radius.lg,
        boxShadow: t.shadow,
        padding: 18,
        maxWidth: 380,
        margin: '0 auto',
      }}>
        <div style={{
          fontFamily: t.typeScale.h1.family === 'display' ? t.fonts.display : t.fonts.sans,
          fontSize: 18, fontWeight: 600, color: t.colors.ink, marginBottom: 4,
          letterSpacing: '-0.01em',
        }}>Schicht zuweisen</div>
        <div style={{ fontSize: 12, color: t.colors.inkMuted, marginBottom: 14 }}>
          Mittwoch, 22. April · 11:00–12:00 · Laden
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6, marginBottom: 14 }}>
          {[
            { name: 'Astrid', hours: '32,0h / 38h', fill: t.colors.slotA, primary: true },
            { name: 'Stephan', hours: '38,0h / 38h', fill: t.colors.slotB, full: true },
            { name: 'Sonja', hours: '24,0h / 30h', fill: t.colors.slotD },
          ].map((p) => (
            <div key={p.name} style={{
              display: 'flex', alignItems: 'center', gap: 10,
              padding: '8px 10px', borderRadius: t.radius.md,
              background: p.primary ? t.colors.accentSoft : 'transparent',
              border: p.primary ? `1px solid ${t.colors.accent}` : `1px solid ${t.colors.border}`,
              cursor: p.full ? 'not-allowed' : 'pointer',
              opacity: p.full ? 0.5 : 1,
            }}>
              <span style={{
                width: 26, height: 26, borderRadius: '50%', background: p.fill,
                display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 11, fontWeight: 700, color: t.colors.ink,
              }}>{p.name[0]}</span>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 13, color: t.colors.ink, fontWeight: 500 }}>{p.name}</div>
                <div style={{ fontSize: 10, color: t.colors.inkMuted, fontFamily: t.fonts.mono, fontVariantNumeric: 'tabular-nums' }}>
                  {p.hours}{p.full && ' · voll'}
                </div>
              </div>
              {p.primary && (
                <span style={{
                  fontSize: 11, color: t.colors.accent, fontWeight: 600,
                  background: t.colors.surface, padding: '2px 8px', borderRadius: 999,
                }}>vorgeschlagen</span>
              )}
            </div>
          ))}
        </div>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
          <button style={{
            padding: '7px 12px', borderRadius: t.radius.md,
            background: 'transparent', border: 'none', color: t.colors.inkSoft,
            fontSize: 13, fontWeight: 500, cursor: 'pointer', fontFamily: t.fonts.sans,
          }}>Abbrechen</button>
          <button style={{
            padding: '7px 14px', borderRadius: t.radius.md,
            background: t.colors.accent, color: t.colors.accentInk,
            border: `1px solid ${t.colors.accent}`,
            fontSize: 13, fontWeight: 500, cursor: 'pointer', fontFamily: t.fonts.sans,
          }}>Astrid zuweisen</button>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { WeekPreview, ModalPreview });
