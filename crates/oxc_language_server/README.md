# oxc_language_server

This crate provides an [LSP](https://microsoft.github.io/language-server-protocol/) Server which is used inside an editor or IDE.

## Server Capabilities

- [Text Document Synchronization](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization): `FULL`,
- Workspace
  - [Workspace Folders](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspaceFoldersServerCapabilities): `true`
  - File Operations: `false`

### For oxlint

- [Workspace commands](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand)
  - `oxc.fixAll`, requires `{ uri: URL }` as command argument. Does safe fixes in `uri` file.
- [Code Actions Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind):
  - `quickfix`
  - `source.fixAll.oxc`, behaves the same as `quickfix` only used when the `CodeActionContext#only` contains
    `source.fixAll.oxc`.
- [Diagnostic Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_pullDiagnostics)
  - Only when [Diagnostics Refresh](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic_refresh) is supported by your client

### For oxfmt

- [DocumentFormattingProvider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting)

## Workspace Options

These options can be passed with [initialize](#initialize), [workspace/didChangeConfiguration](#workspace/didChangeConfiguration) and [workspace/configuration](#workspace/configuration).

| Option Key                | Value(s)                          | Default    | Description                                                                                                                                            |
| ------------------------- | --------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `configPath`              | `<string>` \| `null`              | `null`     | Path to a oxlint configuration file, passing a string will disable nested configuration                                                                |
| `tsConfigPath`            | `<string>` \| `null`              | `null`     | Path to a TypeScript configuration file. If your `tsconfig.json` is not at the root, alias paths will not be resolve correctly for the `import` plugin |
| `unusedDisableDirectives` | `"allow" \| "warn"` \| "deny"`    | `"allow"`  | Define how directive comments like `// oxlint-disable-line` should be reported, when no errors would have been reported on that line anyway            |
| `typeAware`               | `true` \| `false`                 | `false`    | Enables type-aware linting                                                                                                                             |
| `disableNestedConfig`     | `false` \| `true`                 | `false`    | Disabled nested configuration and searches only for `configPath`.                                                                                      |
| `fixKind`                 | [fixKind values](#fixkind-values) | `safe_fix` | The level of a possible fix for a diagnostic, will be applied for the complete workspace (diagnostic, code action, commands and more).                 |
| `fmt.configPath`          | `<string>` \| `null`              | `null`     | Path to a oxfmt configuration file, when `null` is passed, the server will use `.oxfmtrc.json` and the workspace root                                  |
| Diagnostic Pull Mode      |                                   |            |                                                                                                                                                        |
| `run`                     | `"onSave" \| "onType"`            | `"onType"` | Should the server lint the files when the user is typing or saving. In Pull Mode, the editor requests the diagnostic.                                  |
| Deprecated                |                                   |            |                                                                                                                                                        |
| `fmt.experimental`        | `true` \| `false`                 | `false`    | (deprecated) Enables experimental formatting with `oxc_formatter`                                                                                      |
| `flags`                   | `Map<string, string>`             | `<empty>`  | (deprecated) Custom flags passed to the language server.                                                                                               |

### `fixKind` values:

- `"safe_fix"` (default)
- `"safe_fix_or_suggestion"`
- `"dangerous_fix"`
- `"dangerous_fix_or_suggestion"`
- `"none"`
- `"all"`

## Diagnostics Modes

Depending on the client, the server will push diagnostics, or will wait for a pull request from the client.
The server will prefer pull diagnostics when the client supports it and is also supporting [workspace/diagnostic/refresh](#workspacediagnosticrefresh).

## Supported LSP Specifications from Server

### [initialize](https://microsoft.github.io/language-server-protocol/specification#initialize)

Returns the [Server Capabilities](#server-capabilities).\
The client can pass the workspace options like following:

```json
{
  "initializationOptions": [
    {
      "workspaceUri": "file://workspace-directory",
      "options": {
        "run": "onType",
        "configPath": null,
        "tsConfigPath": null,
        "unusedDisableDirectives": "allow",
        "typeAware": false,
        "disableNestedConfig": false,
        "fixKind": "safe_fix",
        "fmt.configPath": null
      }
    }
  ]
}
```

#### Flags (deprecated)

- `key: disable_nested_config`: Disabled nested configuration and searches only for `configPath`
- `key: fix_kind`: see [FixKind values](#fixkind-values) for possible values

### [initialized](https://microsoft.github.io/language-server-protocol/specification#initialized)

When the client did not pass the workspace configuration in [initialize](#initialize), the server will request the configuration for every workspace with [workspace/configuration](#workspaceconfiguration).
The server will tell the client with [client/registerCapability](#clientregistercapability) to watch for `.oxlintrc.json` files or a custom `oxc.configPath`.

### [shutdown](https://microsoft.github.io/language-server-protocol/specification#shutdown)

The server will reset the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

### Workspace

#### [workspace/didChangeConfiguration](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)

The client can pass the workspace options like following:

```json
{
  "settings": [
    {
      "workspaceUri": "file://workspace-directory",
      "options": {
        "run": "onType",
        "configPath": null,
        "tsConfigPath": null,
        "unusedDisableDirectives": "allow",
        "typeAware": false,
        "disableNestedConfig": false,
        "fixKind": "safe_fix",
        "fmt.configPath": null
      }
    }
  ]
}
```

When the client does not pass workspace options, the server will request them with [workspace/configuration](#workspace/configuration).
The server will revalidate or reset the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

When changing the `oxc.configPath` settings:
The server will tell clients with [client/registerCapability](#clientregistercapability) to watch for `.oxlintrc.json` files or a custom `oxc.configPath`.
The server will tell clients with [client/unregisterCapability](#clientunregistercapability) to stop watching for `.oxlintrc.json` files or a custom `oxc.configPath`.

#### [workspace/didChangeWatchedFiles](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)

The server expects this request when one oxlint configuration is changed, added or deleted.
When the server is using [Push Mode](#diagnostics-modes),
the server will revalidate the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.
When the server is using [Pull Mode](#diagnostics-modes),
the server will tell the client to revalidate all diagnostics with [`workspace/diagnostic/refresh`](#workspacediagnosticrefresh).

#### [workspace/didChangeWorkspaceFolders](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_didChangeWorkspaceFolders)

The server expects this request when adding or removing workspace folders.
The server will request the specific workspace configuration, if the client supports it.
The server will tell clients with [client/registerCapability](#clientregistercapability) to watch for `.oxlintrc.json` files or a custom `oxc.configPath`.
The server will tell clients with [client/unregisterCapability](#clientunregistercapability) to stop watching for `.oxlintrc.json` files or a custom `oxc.configPath`.
When the server is using [Push Mode](#diagnostics-modes),
the server will revalidate the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.
When the server is using [Pull Mode](#diagnostics-modes),
the server will tell the client to revalidate all diagnostics with [`workspace/diagnostic/refresh`](#workspacediagnosticrefresh).

#### [workspace/executeCommand](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand)

Executes a [Command](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand) if it exists. See [Server Capabilities](#server-capabilities)

### TextDocument

#### [textDocument/didOpen](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)

The server will cache the internal content of the text document.
When the server is using [Push Mode](#diagnostics-modes), the server will validate the text document and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didSave](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)

When the server is using [Push Mode](#diagnostics-modes) and configuration `run` is set to `onSave`,
the server will validate the text document and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didChange](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)

The server will cache the internal content of the text document.
When the server is using [Push Mode](#diagnostics-modes) and configuration `run` is set to `onType`,
the server will validate the text document and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didClose](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)

It will remove the reference internal.

#### [textDocument/codeAction](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)

Returns a list of [CodeAction](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction)

#### [textDocument/diagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_diagnostic)

Should only be used when the server is using the [Pull Mode](#diagnostics-modes) for diagnostics.
The server will lint the file and report the diagnostics back to the client.

#### [textDocument/formatting](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting)

Returns a list of [TextEdit](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textEdit)

## Optional LSP Specifications from Client

### Client

#### [textDocument/publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics)

When the server is using [Push Mode](#diagnostics-modes) it will lint the file [onOpen](#textdocumentdidopen) and [onChange](#textdocumentdidchange) or [onSave](#textdocumentdidsave)
(depending on the configuration the client passed).

#### [workspace/diagnostic/refresh](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic_refresh)

When the server is using the [Pull Mode](#diagnostics-modes) it will request the client sometimes to re-pull the diagnostics.
This will happen when changing watched files or specific server configurations.

#### [client/registerCapability](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#client_registerCapability)

The server will send this request to watch for specific files. The method `workspace/didChangeWatchedFiles` will be used with custom `registerOptions`.

#### [client/unregisterCapability](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#client_unregisterCapability)

The server will send this request to stop watching for specific files. The `id` will match from [client/registerCapability](#clientregistercapability).

### Workspace

#### [workspace/configuration](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_configuration)

The server will request workspace configurations. The server expects the received items to match the order of the requested items.
Only will be requested when the `ClientCapabilities` has `workspace.configuration` set to true.

The client can return a response like:

```json
[
  {
    "run": "onType",
    "configPath": null,
    "tsConfigPath": null,
    "unusedDisableDirectives": "allow",
    "typeAware": false,
    "disableNestedConfig": false,
    "fixKind": "safe_fix",
    "fmt.configPath": null
  }
]
```
