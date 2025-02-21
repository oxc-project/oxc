# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


## [0.49.0] - 2025-02-10

### Bug Fixes

- 7e6a537 ast: Include `directives` in `body` (#8981) (hi-ogawa)

## [0.36.0] - 2024-11-09

- 092de67 types: [**BREAKING**] Append `rest` field into `elements` for objects and arrays to align with estree (#7212) (ottomated)

### Features

- dc0215c ast_tools: Add #[estree(append_to)], remove some custom serialization code (#7149) (ottomated)

### Bug Fixes


## [0.32.0] - 2024-10-19

### Features

- e310e52 parser: Generate `Serialize` impls in ast_tools (#6404) (ottomated)

