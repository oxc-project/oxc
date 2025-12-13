const commandPrefix = "oxc";

export const enum OxcCommands {
  // always available, even if no tool is active
  ShowOutputChannelLint = `${commandPrefix}.showOutputChannel`,
  ShowOutputChannelFmt = `${commandPrefix}.showOutputChannelFormatter`,

  // only for linter.ts usage
  RestartServerLint = `${commandPrefix}.restartServer`, // without `Linter` suffix for backward compatibility
  ToggleEnableLint = `${commandPrefix}.toggleEnable`, // without `Linter` suffix for backward compatibility
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,

  // only for formatter.ts usage
  RestartServerFmt = `${commandPrefix}.restartServerFormatter`,
}
