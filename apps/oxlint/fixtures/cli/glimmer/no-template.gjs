// A `.gjs` file with no `<template>` block is plain JS: the loader returns it as a
// whole (non-partial) source, so template-sensitive rules like `no-unused-vars` run
// normally here (unlike a template-bearing Glimmer file, where they are skipped).
const unused = 1;
