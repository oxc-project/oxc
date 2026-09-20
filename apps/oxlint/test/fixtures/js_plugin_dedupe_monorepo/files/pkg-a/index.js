// No diagnostic here: `pkg-a` loads the plugin first, and only `pkg-b` has to reuse it.
// Diagnostics in a single file keep the snapshot independent of the order files are linted in.
export {};
