# Task: Test Oxfmt's JSDoc Formatting Against Real-World Repos

Test the ported `prettier-plugin-jsdoc` support in oxfmt by running it against real repos.

## Target Repos

### Group A: Repos using prettier-plugin-jsdoc (migrate from Prettier → oxfmt)
These repos already use the plugin, so we can compare Prettier+plugin output vs oxfmt output.

| Repo | Stars | Version | Notes |
|------|-------|---------|-------|
| wxt-dev/wxt | 9,330 | ^1.8.0 (latest) | Browser extension framework, TS |
| evoluhq/evolu | 1,818 | ^1.3.3 | Local-first framework, TS |
| alabsi91/reanimated-color-picker | 444 | ^1.3.0 | React Native, TS |
| GregRos/parjs | 316 | ^1.3.0 | Parser combinator lib, TS |

### Group B: JSDoc-heavy repos (test oxfmt jsdoc on raw JSDoc)
These repos have heavy JSDoc but don't use prettier-plugin-jsdoc. Good for testing our formatter on diverse JSDoc patterns.

| Repo | Stars | Notes |
|------|-------|-------|
| openlayers/openlayers | 12,300 | Extremely heavy JSDoc, @module, @typedef, import() types |
| chartjs/Chart.js | 67,200 | Extensive @param, @returns, @typedef |
| sveltejs/svelte | 86,000 | Migrated TS→JS+JSDoc, heavy @type imports |
| processing/p5.js | 23,500 | Heavy @method, @param, @example blocks |
| videojs/video.js | 39,500 | @param, @returns, @fires, @event tags |

## Stages

### Stage 1: Setup — clone repos and build oxfmt
- **Goal**: Clone all target repos into `/tmp/jsdoc-test-repos/`, build oxfmt release binary
- **Depends on**: none
- **Parallel**: yes (clone and build can happen simultaneously)
- **Success criteria**: All repos cloned, oxfmt binary built and runnable
- **Status**: Not Started

### Stage 2: Test Group A — prettier-plugin-jsdoc repos
- **Goal**: For each Group A repo: (1) run Prettier+plugin to get baseline, (2) migrate to oxfmt, (3) run oxfmt, (4) diff JSDoc comments between the two outputs
- **Depends on**: Stage 1
- **Parallel**: yes (each repo independently)
- **Success criteria**: Diff report for each repo showing JSDoc-specific differences
- **Status**: Not Started

### Stage 3: Test Group B — JSDoc-heavy repos
- **Goal**: For each Group B repo: (1) run oxfmt with jsdoc enabled, (2) check for crashes/errors, (3) sample-check JSDoc formatting quality
- **Depends on**: Stage 1
- **Parallel**: yes (each repo independently)
- **Success criteria**: No crashes, report on formatting behavior for each repo
- **Status**: Not Started

### Stage 4: Summarize findings
- **Goal**: Aggregate all results into a single report
- **Depends on**: Stage 2, Stage 3
- **Parallel**: no
- **Success criteria**: Summary report with actionable findings
- **Status**: Not Started
