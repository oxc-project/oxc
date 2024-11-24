# oxc_language_server

This crate provides an [LSP](https://microsoft.github.io/language-server-protocol/) Server which is used inside an editor or IDE.

## Server Capabilities

- [Text Document Synchronization](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization): `FULL`,
- Workspace
  - [Workspace Folders](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspaceFoldersServerCapabilities): `true`
  - File Operations: `false`
- [Diagnostic Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_pullDiagnostics)
  - `interFileDependencies`: `false`
  - `workspaceDiagnostics`: `false`
- [Code Actions Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind):
  - `quickfix`

## Supported LSP Specifications from Server

- [initialize](https://microsoft.github.io/language-server-protocol/specification#initialize)
  - Returns the [Server Capabilities](#server-capabilities)
- [initialized](https://microsoft.github.io/language-server-protocol/specification#initialized)
- [shutdown](https://microsoft.github.io/language-server-protocol/specification#shutdown)

### Workspace

#### [workspace/didChangeConfiguration](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)

The server expects this request when settings like `run`, `enable` or `configPath` are changed.
The server will revalidate or reset the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

#### [workspace/didChangeWatchedFiles](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)

The server expects this request when the oxlint configuration is changed.
The server will revalidate the diagnostics for all open files and send one or more [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) requests to the client.

### TextDocument

#### [textDocument/didOpen](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)

It will save the reference internal.

#### [textDocument/didChange](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)

It will update the reference internal.

#### [textDocument/didClose](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)

It will remove the reference internal.

#### [textDocument/diagnostic](https://microsoft.github.io/language-server-protocol/specification#textDocument_diagnostic)

Returns all Diagnostics for the requested file

#### [textDocument/codeAction](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)

Returns a list of [CodeAction](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction)

## Expected LSP Specification from Client

### TextDocument

#### [textDocument/publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics)

Returns a [PublishDiagnostic object](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#publishDiagnosticsParams)
