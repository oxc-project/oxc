# oxc_language_server

This crate provides an [LSP](https://microsoft.github.io/language-server-protocol/) Server which is used inside an editor or IDE.

## Server Capabilities

- [Text Document Synchronization](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization): `FULL`,
- Workspace
  - [Workspace Folders](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspaceFoldersServerCapabilities): `true`
  - File Operations: `false`
  - [Workspace commands](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand)
    - `oxc.fixAll`, requires `{ uri: URL }` as command argument. Does safe fixes in `uri` file.
- [Code Actions Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind):
  - `quickfix`
  - `source.fixAll.oxc`, behaves the same as `quickfix` only used when the `CodeActionContext#only` contains
    `source.fixAll.oxc`.

## Supported LSP Specifications from Server

### [initialize](https://microsoft.github.io/language-server-protocol/specification#initialize)

Returns the [Server Capabilities](#server-capabilities).\
Initialization Options:

| Option Key   | Value(s)               | Default          | Description                                                                                          |
| ------------ | ---------------------- | ---------------- | ---------------------------------------------------------------------------------------------------- |
| `run`        | `"onSave" \| "onType"` | `"onType"`       | Should the server lint the files when the user is typing or saving                                   |
| `configPath` | `<string>`             | `.oxlintrc.json` | Path to a oxlint configuration file, pass '' to enable nested configuration                          |
| `flags`      | `Map<string, string>`  | `<empty>`        | Special oxc language server flags, currently only one flag key is supported: `disable_nested_config` |

### [initialized](https://microsoft.github.io/language-server-protocol/specification#initialized)

### [shutdown](https://microsoft.github.io/language-server-protocol/specification#shutdown)

The server will reset the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

### Workspace

#### [workspace/didChangeConfiguration](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)

The server expects this request when settings like `run`, `flags` or `configPath` are changed.
The server will revalidate or reset the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

#### [workspace/didChangeWatchedFiles](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)

The server expects this request when one oxlint configuration is changed, added or deleted.
The server will revalidate the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

Note: When nested configuration is active, the client should send all `.oxlintrc.json` configurations to the server after the [initialized](#initialized) response.

#### [workspace/executeCommand](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand)

Executes a [Command](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand) if it exists. See [Server Capabilities](#server-capabilities)

### TextDocument

#### [textDocument/didOpen](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)

The server will validate the file content and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didSave](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)

When the configuration `run` is set to `onSave`, the server will validate the file content and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didChange](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)

When the configuration `run` is set to `onType`, the server will validate the file content and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [textDocument/didClose](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)

It will remove the reference internal.

#### [textDocument/codeAction](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)

Returns a list of [CodeAction](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction)

## Expected LSP Specification from Client

### TextDocument

#### [textDocument/publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics)

Returns a [PublishDiagnostic object](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#publishDiagnosticsParams)
