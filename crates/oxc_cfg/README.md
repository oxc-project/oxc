# Oxc CFG

Control Flow Graph construction and analysis for JavaScript and TypeScript.

## Overview

This crate provides data structures and algorithms for building and analyzing Control Flow Graphs (CFGs) from AST nodes. CFGs are essential for advanced static analysis, optimization, and understanding program flow.

## Key Features

- **CFG construction**: Build control flow graphs from AST nodes
- **Block-based representation**: Organizes code into basic blocks
- **Graph analysis**: Traverse and analyze control flow patterns
- **DOT export**: Visualize CFGs using Graphviz dot format
- **Visitor integration**: Works with oxc visitor patterns

## Architecture

### Basic Blocks

The CFG organizes code into basic blocks - sequences of instructions with:

- Single entry point (first instruction)
- Single exit point (last instruction)
- No internal branches or jumps

### Graph Structure

- **Nodes**: Basic blocks containing instructions
- **Edges**: Control flow between blocks (conditional, unconditional, exception)
- **Entry/Exit**: Special blocks for function entry and exit points

### Analysis Applications

- **Dead code elimination**: Find unreachable blocks
- **Data flow analysis**: Track variable usage across control paths
- **Loop detection**: Identify back edges and loop structures
- **Path analysis**: Enumerate possible execution paths

The CFG integrates with semantic analysis to provide comprehensive program understanding.
