const x = 1;
{ declare const x: number; { declare const x: number; use(x); } }
{ declare const missing: number; { declare const missing: number; missing++; } }
