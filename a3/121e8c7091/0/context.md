# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Cursor-Based Comment Printing — Print ALL Comments Correctly

## Context

Current codegen is **opt-in per Gen impl**: only nodes with explicit `print_comments_at()` calls print comments. Many node types have no call, so comments are silently dropped. The HashMap-based lookup also has allocation/hashing overhead.

The goal: print **every** comment in the correct position, with better performance than today.

## Design: Auto-Flushing Cursor

A cursor into t...

