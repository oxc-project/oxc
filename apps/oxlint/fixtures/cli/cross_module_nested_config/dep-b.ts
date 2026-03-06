// this file is also included in dep-a.ts and dep-a.ts should report a no-cycle diagnostic
import './dep-a.ts';

export function b() { /* ... */ }
