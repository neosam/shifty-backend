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
// Week 17 fully overlaps absence a-001 (Vacation 20.04–26.04). Per UX-spec these
// bookings stay (no auto-cleanup) but get the booking_on_absence_day reverse-warning.
const MY_SHIFTS = [
  {
    week: 17, year: 2026, range: '20.04 – 26.04',
    absence: { id: 'a-001', category: 'Vacation', from: '2026-04-20', to: '2026-04-26', description: 'Familienurlaub Italien' },
    days: [
      { day: 'Mo 20.04', date: '2026-04-20', items: [{ time: '09:00–13:00', area: 'Laden' }, { time: '14:00–17:00', area: 'Laden' }], hours: 7.0, absent: true, warning: 'Buchung auf Urlaubstag' },
      { day: 'Di 21.04', date: '2026-04-21', items: [], hours: 0, absent: true },
      { day: 'Mi 22.04', date: '2026-04-22', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0, absent: true, warning: 'Buchung auf Urlaubstag' },
      { day: 'Do 23.04', date: '2026-04-23', items: [{ time: '09:00–12:00', area: 'Laden' }], hours: 3.0, absent: true, warning: 'Buchung auf Urlaubstag' },
      { day: 'Fr 24.04', date: '2026-04-24', items: [{ time: '13:00–18:00', area: 'Laden' }], hours: 5.0, absent: true, warning: 'Buchung auf Urlaubstag' },
      { day: 'Sa 25.04', date: '2026-04-25', items: [{ time: '14:00–18:00', area: 'Backen' }], hours: 4.0, absent: true, warning: 'Buchung auf Urlaubstag' },
    ],
    total: 23.0,
  },
  {
    week: 18, year: 2026, range: '27.04 – 03.05',
    days: [
      { day: 'Mo 27.04', date: '2026-04-27', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0 },
      { day: 'Di 28.04', date: '2026-04-28', items: [{ time: '09:00–17:00', area: 'Laden' }], hours: 8.0 },
      { day: 'Mi 29.04', date: '2026-04-29', items: [], hours: 0 },
      { day: 'Do 30.04', date: '2026-04-30', items: [{ time: '09:00–13:00', area: 'Laden' }], hours: 4.0 },
      { day: 'Fr 01.05', date: '2026-05-01', items: [], hours: 0, note: 'Feiertag' },
      { day: 'Sa 02.05', date: '2026-05-02', items: [{ time: '10:00–14:00', area: 'Laden' }], hours: 4.0 },
    ],
    total: 20.0,
  },
];

// ─── Absence-Domain (Range-based, Backend v1.0) ──────────────
// Mirrors AbsencePeriodTO from rest-types: from_date / to_date inclusive.
// Stunden pro Tag werden serverseitig aus dem gültigen Vertrag abgeleitet
// (derive_hours_for_range) — Frontend hält nur Range + Kategorie + $version.
const ABSENCE_CATEGORIES = {
  Vacation:    { id: 'Vacation',    label: 'Urlaub',            short: 'U',  color: 'var(--good)',      soft: 'var(--good-soft)' },
  SickLeave:   { id: 'SickLeave',   label: 'Krank',             short: 'K',  color: 'var(--bad)',       soft: 'var(--bad-soft)' },
  UnpaidLeave: { id: 'UnpaidLeave', label: 'Unbezahlter Urlaub',short: 'UU', color: 'var(--ink-muted)', soft: 'var(--surface-2)' },
};

// Sample AbsencePeriodTOs. Mix of past, current and future ranges.
// Note: WEEK_DEFAULT week = KW17/2026 (20.04 – 26.04).
const ABSENCES = [
  { id: 'a-001', sales_person_id: 'p3', category: 'Vacation',    from_date: '2026-04-20', to_date: '2026-04-26', description: 'Familienurlaub Italien', created: '2026-03-10T09:14:00Z', deleted: null, version: 'v-1', warnings: [
    { kind: 'absence_overlaps_booking', data: { absence_id: 'a-001', booking_id: 'b-771', date: '2026-04-22' } },
    { kind: 'absence_overlaps_booking', data: { absence_id: 'a-001', booking_id: 'b-772', date: '2026-04-23' } },
  ]},
  { id: 'a-002', sales_person_id: 'p5', category: 'SickLeave',   from_date: '2026-04-22', to_date: '2026-04-24', description: 'Grippe — Attest liegt vor', created: '2026-04-22T07:02:00Z', deleted: null, version: 'v-1', warnings: [
    { kind: 'absence_overlaps_booking', data: { absence_id: 'a-002', booking_id: 'b-803', date: '2026-04-22' } },
    { kind: 'absence_overlaps_manual_unavailable', data: { absence_id: 'a-002', unavailable_id: 'u-44' } },
  ]},
  { id: 'a-003', sales_person_id: 'p11', category: 'Vacation',   from_date: '2026-05-04', to_date: '2026-05-15', description: 'Urlaub Mai', created: '2026-02-18T11:30:00Z', deleted: null, version: 'v-1', warnings: [] },
  { id: 'a-004', sales_person_id: 'p1', category: 'Vacation',    from_date: '2026-07-13', to_date: '2026-08-02', description: 'Sommerurlaub', created: '2026-01-05T16:55:00Z', deleted: null, version: 'v-1', warnings: [] },
  { id: 'a-005', sales_person_id: 'p8', category: 'UnpaidLeave', from_date: '2026-06-01', to_date: '2026-06-07', description: 'Familienangelegenheit', created: '2026-04-12T08:21:00Z', deleted: null, version: 'v-1', warnings: [] },
  { id: 'a-006', sales_person_id: 'p2', category: 'Vacation',    from_date: '2026-03-23', to_date: '2026-03-27', description: 'Skiurlaub', created: '2026-01-30T14:12:00Z', deleted: null, version: 'v-1', warnings: [] },
  { id: 'a-007', sales_person_id: 'p4', category: 'SickLeave',   from_date: '2026-04-13', to_date: '2026-04-15', description: '', created: '2026-04-13T08:05:00Z', deleted: null, version: 'v-1', warnings: [] },
  { id: 'a-008', sales_person_id: 'p9', category: 'Vacation',    from_date: '2026-04-27', to_date: '2026-05-03', description: 'Pfingsten verlängert', created: '2026-03-22T10:00:00Z', deleted: null, version: 'v-1', warnings: [] },
];

// Per-day UnavailabilityMarker for the current shiftplan week, keyed by sales_person_id.
// Mirrors UnavailabilityMarkerTO. `Both` indicates redundant manual_unavailable after cutover.
// Index 0..5 = Mo..Sa to match WEEK_DEFAULT.days.
const UNAVAILABILITY_BY_PERSON = {
  p3:  [ {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}},
         {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}},
         {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}},
         {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}},
         {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}},
         {kind:'absence_period', data:{absence_id:'a-001', category:'Vacation'}} ],
  p5:  [ null,
         null,
         {kind:'absence_period', data:{absence_id:'a-002', category:'SickLeave'}},
         {kind:'both',           data:{absence_id:'a-002', category:'SickLeave'}},
         {kind:'absence_period', data:{absence_id:'a-002', category:'SickLeave'}},
         null ],
  p10: [ null, null, null, null, {kind:'manual_unavailable'}, {kind:'manual_unavailable'} ],
  p12: [ null, null, null, null, null, {kind:'manual_unavailable'} ],
};

// Cutover feature flag (`absence_range_source_active`). After flip, ExtraHours
// for V/SK/UL is deprecated → POST returns 403 ExtraHoursCategoryDeprecatedErrorTO.
const FEATURE_FLAGS = { absence_range_source_active: true };

// Vacation entitlement per person for the current year.
// Shape mirrors a backend "vacation_quota" view: total contractual days +
// carryover from previous year, plus already-approved/used days.
// (Backend may compute "used" from absence ranges; we mirror it for the demo.)
const VACATION_QUOTA = {
  p1:  { year: 2026, total: 28, carryover: 3,  used: 4,  pending: 0 },
  p2:  { year: 2026, total: 28, carryover: 0,  used: 12, pending: 0 },
  p3:  { year: 2026, total: 24, carryover: 2,  used: 7,  pending: 0 },
  p4:  { year: 2026, total: 28, carryover: 5,  used: 0,  pending: 0 },
  p5:  { year: 2026, total: 26, carryover: 1,  used: 8,  pending: 0 },
  p6:  { year: 2026, total: 28, carryover: 0,  used: 14, pending: 0 },
  p7:  { year: 2026, total: 28, carryover: 4,  used: 6,  pending: 0 },
  p8:  { year: 2026, total: 28, carryover: 2,  used: 10, pending: 0 },
  p9:  { year: 2026, total: 20, carryover: 0,  used: 5,  pending: 0 },
  p10: { year: 2026, total: 0,  carryover: 0,  used: 0,  pending: 0 }, // freiwillig
  p11: { year: 2026, total: 16, carryover: 0,  used: 3,  pending: 0 },
  p12: { year: 2026, total: 0,  carryover: 0,  used: 0,  pending: 0 }, // freiwillig
  p13: { year: 2026, total: 24, carryover: 1,  used: 8,  pending: 0 },
  p14: { year: 2026, total: 28, carryover: 0,  used: 11, pending: 0 },
};

window.SHIFTY_DATA = { PEOPLE, PERSON_BY_ID, WEEK_DEFAULT, YEAR_SUMMARY, MY_SHIFTS, ABSENCES, ABSENCE_CATEGORIES, UNAVAILABILITY_BY_PERSON, FEATURE_FLAGS, VACATION_QUOTA };
