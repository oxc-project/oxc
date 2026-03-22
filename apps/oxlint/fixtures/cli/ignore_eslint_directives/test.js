// eslint-disable-next-line no-debugger -- IGNORED
debugger;

// oxlint-disable-next-line no-console -- suppressed
console.log('suppressed by oxlint-disable-next-line');

// eslint-disable-next-line no-console -- IGNORED
console.log('not suppressed (eslint-disable-next-line ignored)');

/* eslint-disable no-debugger */ // IGNORED
debugger;

/* oxlint-disable no-debugger */ // suppressed
debugger;

/* eslint-enable no-debugger */ // IGNORED (no effect)
debugger;

/* oxlint-enable no-debugger */ // re-enabled
debugger;

console.log('not suppressed (eslint-disable-line ignored)'); // eslint-disable-line no-console -- IGNORED

console.log('suppressed by oxlint-disable-line'); // oxlint-disable-line no-console -- suppressed
