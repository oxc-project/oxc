const enum Phase {
  idle,
  active,
}

const enum Theme {
  Light = "Light",
  Dark = "Dark",
}

// `as` type-cast
const a1 = Phase.idle as Phase;
const a2 = Phase["active"] as Phase;
const a3 = Theme.Light as Theme;

// `satisfies` type-cast
const s1 = Phase.idle satisfies Phase;
const s2 = Theme.Dark satisfies Theme;

// `!` non-null assertion
const n1 = Phase.active!;
const n2 = Theme.Light!;

// `<T>` type assertion
const t1 = <Phase>Phase.idle;

// nested / combined
const c1 = (Phase.active as Phase) satisfies Phase;
const c2 = (Phase.idle satisfies Phase)!;
