const commandPrefix = "oxc";

export const enum OxcCommands {
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  // only for linter.ts usage
  RestartServer = `${commandPrefix}.restartServer`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,
}
