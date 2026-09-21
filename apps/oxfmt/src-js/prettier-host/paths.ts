// Locates this directory at runtime.
//
// Lives here, as its own build entry, because `import.meta.url` must resolve
// against `dist/prettier-host/`. Computing it from `libs/plugin-resolution.ts`
// would instead resolve against whichever chunk absorbed that module.
export const HOST_DIR: string = new URL("./", import.meta.url).href;
