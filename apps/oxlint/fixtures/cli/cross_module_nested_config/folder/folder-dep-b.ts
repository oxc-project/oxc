// this file is also included in folder-dep-a.ts and folder-dep-a.ts should report a no-cycle diagnostic
import './folder-dep-a.ts';

export function b() { /* ... */ }
