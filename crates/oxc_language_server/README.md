# oxc_language_server

This crate provides an [LSP](https://microsoft.github.io/language-server-protocol/) Server which is used inside an editor or IDE.

## Server Capabilities

- [Text Document Synchronization](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization): `FULL`,
- Workspace
  - [Workspace Folders](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspaceFoldersServerCapabilities): `true`
  - File Operations: `false`
- [Code Actions Provider](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind):
  - `quickfix`

## Supported LSP Specifications from Server

- [initialize](https://microsoft.github.io/language-server-protocol/specification#initialize)
- [initialized](https://microsoft.github.io/language-server-protocol/specification#initialized)
- [shutdown](https://microsoft.github.io/language-server-protocol/specification#shutdown)

### Workspace

#### [workspace/didChangeConfiguration](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)

The server expect this request when settings like `run`, `enable` or `configPath` are changed.
The server will revalidate or reset the diagnostics for all open files and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

#### [workspace/didChangeWatchedFiles](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)

The server expect this request when the oxlint configuration is changed.
The server will revalidate the diagnostics for all open files and send a [textDocument/publishDiagnostics](#textdocumentpublishdiagnostics) request to the client.

### TextDocument

-- ToDo

#### [textDocument/didOpen](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)

-- ToDo

#### [textDocument/didSave](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)

-- ToDo

#### [textDocument/didChange](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)

-- ToDo

#### [textDocument/didClose](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)

-- ToDo

#### [textDocument/codeAction](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)

-- ToDo

## Expected LSP Specification from Client

### TextDocument

#### [textDocument/publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics)

-- ToDo
