# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.15.0] - 2024-06-18

- 5c38a0f codegen: [**BREAKING**] New code gen API (#3740) (Boshen)

### Features

- ee627c3 isolated-declarations: Create unique name for `_default` (#3730) (Dunqing)
- 81e9526 isolated-declarations: Inferring set accessor parameter type from get accessor return type (#3725) (Dunqing)
- 77d5533 isolated-declarations: Report errors that are consistent with typescript. (#3720) (Dunqing)
- 0b8098a napi: Isolated-declaration (#3718) (Boshen)

### Bug Fixes

- f1b793f isolated-declarations: Function overloads reaching unreachable (#3739) (Dunqing)
- 0fbecdc isolated-declarations: Should be added to references, not bindings (#3726) (Dunqing)

### Refactor

- 3c59735 isolated-declarations: Remove `TransformDtsCtx` (#3719) (Boshen)
- 815260e isolated-declarations: Decouple codegen (#3715) (Boshen)

