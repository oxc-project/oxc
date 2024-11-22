# oxc_language_server

This crate provides an [LSP](https://microsoft.github.io/language-server-protocol/) Server which is used inside an editor or IDE.

## Server Capabilities

- Text Document Sync: `FULL`, Documents are synced by always sending the full content of the document
- Workspace
  - Workspace Folders: `true`
  - File Operations: `false`
- Code Actions Provider:
  - `quickfix`

## Supported LSP Specifications

- [initialize](https://microsoft.github.io/language-server-protocol/specification#initialize)
- [initialized](https://microsoft.github.io/language-server-protocol/specification#initialized)
- [shutdown](https://microsoft.github.io/language-server-protocol/specification#shutdown)

### Workspace

#### [workspace/didChangeConfiguration](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeConfiguration)

-- ToDo

When settings like `configPath` is changing

#### [workspace/didChangeWatchedFiles](https://microsoft.github.io/language-server-protocol/specification#workspace_didChangeWatchedFiles)

-- ToDo

When oxlint configuration is changing

### TextDocument

-- ToDo

#### [textDocument/didOpen](https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen)

-- ToDo

#### [textDocument/didSave](https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave)

-- ToDo

#### [textDocument/didChange](https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange)

-- ToDo

#### [textDocument/didClose](https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose)

- ToDo

#### [textDocument/codeAction](https://microsoft.github.io/language-server-protocol/specification#textDocument_codeAction)

- ToDo