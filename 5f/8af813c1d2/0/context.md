# Session Context

## User Prompts

### Prompt 1

Extract oxc_jsdoc out of oxc_semantic

### Prompt 2

create pr

### Prompt 3

do we leave out tests, why diff 20 files changed, 552 insertions(+), 1103 deletions(-)

### Prompt 4

change the feature to jsdoc

### Prompt 5

This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation.

Analysis:
Let me chronologically trace through the conversation:

1. User's first request: "Extract oxc_jsdoc out of oxc_semantic"
2. I explored the JSDoc code in oxc_semantic, understanding the structure
3. Created the new `oxc_jsdoc` crate with parser files
4. Kept `JSDocFinder` in `oxc_semantic` since it depends on semantic types
5. Had to it...

