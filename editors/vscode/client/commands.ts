const commandPrefix = "oxc";

export const enum OxcCommands {
  RestartServer = `${commandPrefix}.restartServer`,
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
  // only for linter.ts usage
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,
}
