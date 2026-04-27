// preview-data.jsx — realistic dummy data for Shifty preview
// Mirrors the shapes used in shifty-dioxus state types (sales_person,
// shiftplan, weekly_summary, blocks). German locale.

// `color: null` = no shiftplan_color set → render with neutral chip
const PEOPLE = [
  { id: 'p1', name: 'Lena',    initials: 'LB', color: '#dbe0ff', paid: true,  hours: 32.0, target: 38.0 },
  { id: 'p2', name: 'Tobias',  initials: 'TK', color: '#dceadc', paid: true,  hours: 38.0, target: 38.0 },
  { id: 'p3', name: 'Hannah',  initials: 'HM', color: null,       paid: false, hours: 18.5, target: 24.0 },
  { id: 'p4', name: 'Mia',     initials: 'MW', color: '#fadcd8', paid: true,  hours: 30.0, target: 32.0 },
  { id: 'p5', name: 'Jonas',   initials: 'JN', color: '#e6dcf5', paid: false, hours: 12.0, target: 16.0 },
  { id: 'p6', name: 'Leon',    initials: 'LH', color: '#d4e8ec', paid: true,  hours: 24.0, target: 24.0 },
  { id: 'p7', name: 'Emma',    initials: 'EK', color: null,       paid: false, hours: 8.0,  target: 12.0 },
  { id: 'p8', name: 'Stefan G.', initials: 'SG', color: '#e0e6cf', paid: true,  hours: 36.0, target: 38.0 },
  { id: 'p9', name: 'Petra N.',  initials: 'PN', color: '#f3d4dc', paid: true,  hours: 20.0, target: 20.0 },
  { id: 'p10',name: 'Michael', initials: 'MH', color: null,       paid: false, hours: 0.0,  target: 8.0 },
  { id: 'p11',name: 'Sabine',  initials: 'SF', color: '#dcd6e8', paid: true,  hours: 14.0, target: 16.0 },
  { id: 'p12',name: 'Julia',   initials: 'JM', color: null,    paid: false, hours: 6.5,  target: 12.0 },
  { id: 'p13',name: 'Andrea',  initials: 'AB', color: '#d8dfd2', paid: true,  hours: 22.0, target: 24.0 },
  { id: 'p14',name: 'Thomas K.', initials: 'TK', color: '#f5dcc6', paid: true,  hours: 28.0, target: 32.0 },
];

const PERSON_BY_ID = Object.fromEntries(PEOPLE.map((p) => [p.id, p]));

// One full week, 8 hourly slots × 6 days, with realistic gaps and conflicts
const WEEK_DEFAULT = {
  week: 17, year: 2026, label: 'KW 17 / 2026 · vom 20.04',
  tabs: ['Laden', 'Backen', 'Reinigung'],
  hours: ['09:00','10:00','11:00','12:00','13:00','14:00','15:00','16:00','17:00','18:00','19:00'],
  days: [
    { name: 'Mo', long: 'Montag', date: '20.04' },
    { name: 'Di', long: 'Dienstag', date: '21.04' },
    { name: 'Mi', long: 'Mittwoch', date: '22.04' },
    { name: 'Do', long: 'Donnerstag', date: '23.04' },
    { name: 'Fr', long: 'Freitag', date: '24.04' },
    { name: 'Sa', long: 'Samstag', date: '25.04' },
  ],
  // grid[hourIdx][dayIdx] = { need, assigned: [{id, conflict?}] }
  grid: (() => {
    const g = [];
    const fill = (need, ids, conflicts = []) => ({
      need, assigned: ids.map((id) => ({ id, conflict: conflicts.includes(id) })),
    });
    g.push([fill(2,['p1','p2']), fill(2,['p11','p3']),     fill(2,['p1','p3']),  fill(2,['p6','p3']),  fill(3,['p5','p7','p3'],['p5']), fill(2,['p5','p4'],['p5'])]);
    g.push([fill(3,['p1','p2','p13']), fill(2,['p11','p3'],['p11']), fill(2,['p1','p3']), fill(2,['p6','p3'],['p6']), fill(3,['p5','p7','p3'],['p5']), fill(2,['p5','p4','p11'],['p5'])]);
    g.push([fill(3,['p1','p2','p13']), fill(2,['p11','p3']), fill(2,['p1','p3']), fill(2,['p6','p3']), fill(3,['p5','p7','p3'],['p5']), fill(2,['p5','p4','p11'],['p5'])]);
    g.push([fill(3,['p1','p2','p13']), fill(2,['p11','p3']), fill(2,['p1','p3']), fill(2,['p6','p3']), fill(3,['p7','p3']), fill(2,['p5','p4','p11'],['p5'])]);
    g.push([fill(2,['p1','p2']), fill(2,['p11','p3']), fill(2,['p1','p3']), fill(2,['p6','p3']), fill(2,['p7','p3']), fill(2,['p4','p14'])]);
    g.push([fill(1,['p1']), fill(1,['p2']), fill(2,['p1','p10'],['p10']), fill(2,['p1','p6'],['p6']), fill(2,['p1','p3']), fill(2,['p4','p14'])]);
    g.push([fill(1,['p1']), fill(1,['p2']), fill(1,['p10'],['p10']), fill(1,['p1']), fill(3,['p1','p12','p3'],['p12','p3']), fill(2,['p4','p14'])]);
    g.push([fill(2,['p1','p10'],['p10']), fill(1,['p2']), fill(2,['p10','p2'],['p10']), fill(2,['p1','p2']), fill(6,['p1','p12','p3','p2','p11','p4'],['p12','p3']), fill(2,['p4','p14'])]);
    g.push([fill(2,['p8','p9']), fill(1,['p2']), fill(1,['p2']), fill(1,['p2']), fill(3,['p1','p3','p2'],['p3']), fill(0,[])]);
    g.push([fill(3,['p8','p9','p2'],['p2']), fill(1,['p2']), fill(1,['p2']), fill(1,['p2']), fill(2,['p1','p2']), fill(0,[])]);
    g.push([fill(2,['p8','p9']), fill(1,['p2']), fill(1,['p2']), fill(1,['p2']), fill(2,['p1','p2']), fill(0,[])]);
    return g;
  })(),
};

// Year overview — 52 weeks of paid/required/missing
const YEAR_SUMMARY = (() => {
  const out = [];
  for (let w = 1; w <= 52; w++) {
    const required = 220 + Math.round(Math.sin(w / 4) * 18);
    const paid = required + Math.round((Math.sin(w * 0.7) + Math.cos(w * 1.3)) * 12) - (w === 17 ? 8 : 0);
    const volunteer = 30 + Math.round(Math.cos(w / 3) * 8);
    out.push({ week: w, year: 2026, required, paid, volunteer, missing: required - paid });
  }
  return out;
})();

// "Meine Schichten" — Astrid's upcoming shifts
const MY_SHIFTS = [
  {
    week: 17, year: 2026, range: '20.04 – 26.04',
    days: [
      { day: 'Mo 20.04', items: [{ time: '09:00–13:00', area: 'Laden' }, { time: '14:00–17:00', area: 'Laden' }], hours: 7.0 },
      { day: 'Di 21.04', items: [], hours: 0 },
      { day: 'Mi 22.04', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0 },
      { day: 'Do 23.04', items: [{ time: '09:00–12:00', area: 'Laden' }], hours: 3.0 },
      { day: 'Fr 24.04', items: [{ time: '13:00–18:00', area: 'Laden' }], hours: 5.0, note: '⚠ Konflikt: 14:00 doppelt gebucht' },
      { day: 'Sa 25.04', items: [{ time: '14:00–18:00', area: 'Backen' }], hours: 4.0 },
    ],
    total: 23.0,
  },
  {
    week: 18, year: 2026, range: '27.04 – 03.05',
    days: [
      { day: 'Mo 27.04', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0 },
      { day: 'Di 28.04', items: [{ time: '09:00–17:00', area: 'Laden' }], hours: 8.0 },
      { day: 'Mi 29.04', items: [], hours: 0 },
      { day: 'Do 30.04', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0 },
      { day: 'Fr 01.05', items: [], hours: 0, note: 'Feiertag' },
      { day: 'Sa 02.05', items: [{ time: '10:00–14:00', area: 'Laden' }], hours: 4.0 },
    ],
    total: 20.0,
  },
];

window.SHIFTY_DATA = { PEOPLE, PERSON_BY_ID, WEEK_DEFAULT, YEAR_SUMMARY, MY_SHIFTS };
