export enum Ev {
  Prefix = "app",
  Click = `${Prefix}:click`,
  Multi = `line1\nline2`,
  Num = `x${1}y`,
  Empty = ``,
  Adjacent = `${Prefix}${Num}`,
  Qualified = `${Ev.Prefix}:${Ev["Num"]}`,
  Nested = `a${`b${1 + 2}c`}d`,
  Escapes = `\x41\u0042\u{1F600}\t\`\$\\`,
  Surrogates = `\uD800${"\uDC00"}\uD801`,
  Numbers = `${-0},${-1.5},${1e21},${Infinity},${NaN}`,
  Runtime = `prefix${Math.random()}suffix`,
}

export const enum ConstEv {
  Prefix = "app",
  Click = `${Prefix}:click`,
  Multi = `line1\nline2`,
  Num = `x${1}y`,
  Runtime = `prefix${Math.random()}suffix`,
}
