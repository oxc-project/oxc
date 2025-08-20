import { mkdirSync, unlinkSync, writeFileSync } from 'fs';
import { join } from 'path';

import { spawnCommand } from './spawn.js';

/**
 * Write source code to a temporary file and return the path
 */
function writeTempFile(sourceCode: string, filename: string): string {
  // Create temp directory if it doesn't exist
  mkdirSync('/tmp/oxc-mcp', { recursive: true });
import { mkdirSync, unlinkSync, writeFileSync, mkdtempSync } from 'fs';
import { join } from 'path';
import * as os from 'os';

import { spawnCommand } from './spawn.js';

/**
 * Write source code to a temporary file and return the path
 */
// Create a unique temp directory for this process
const tempDir: string = mkdtempSync(join(os.tmpdir(), 'oxc-mcp-'));

function writeTempFile(sourceCode: string, filename: string): string {
  // Write the file to the unique temp directory
  const tempPath = join(tempDir, filename);
  writeFileSync(tempPath, sourceCode, 'utf8');
  return tempPath;
}

/**
 * Execute a tool with temporary file cleanup
 */
async function executeWithTempFile<T extends { sourceCode: string; filename?: string }>(
  options: T,
  commandBuilder: (tempPath: string, options: T) => { command: string; args: string[] },
): Promise<string> {
  const { sourceCode, filename = 'input.js' } = options;

  if (typeof sourceCode !== 'string') {
    throw new Error('sourceCode must be a string');
  }

  const tempPath = writeTempFile(sourceCode, filename);

  try {
    const { command, args } = commandBuilder(tempPath, options);
    const result = await spawnCommand(command, args);
    return result;
  } finally {
    // Clean up temporary file
    try {
      unlinkSync(tempPath);
    } catch {
      // Ignore cleanup errors
    }
  }
}

// Parser Tool
export interface ParseOptions {
  sourceCode: string;
  filename?: string;
  showAst?: boolean;
  showEstree?: boolean;
  showComments?: boolean;
}

export async function parseCode(options: ParseOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath, opts) => {
    const args = ['run', '-p', 'oxc_linter', '--example', 'linter', tempPath];
    if (opts.showAst) {
      args.push('--ast');
    }
    if (opts.showEstree) {
      args.push('--estree');
    }
    if (opts.showComments) {
      args.push('--comments');
    }
    return { command: 'cargo', args };
  });
}

// Linter Tool
export interface LinterOptions {
  sourceCode: string;
  filename?: string;
}

export async function lintCode(options: LinterOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_linter', '--example', 'linter', tempPath],
  }));
}

// Formatter Tool
export interface FormatterOptions {
  sourceCode: string;
  filename?: string;
}

export async function formatCode(options: FormatterOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_formatter', '--example', 'formatter', tempPath],
  }));
}

// Semantic Analysis Tool
export interface SemanticOptions {
  sourceCode: string;
  filename?: string;
  showSymbols?: boolean;
}

export async function analyzeCode(options: SemanticOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath, opts) => {
    const args = ['run', '-p', 'oxc_semantic', '--example', 'semantic', tempPath];
    if (opts.showSymbols) {
      args.push('--symbols');
    }
    return { command: 'cargo', args };
  });
}

// Transformer Tool
export interface TransformerOptions {
  sourceCode: string;
  filename?: string;
  targets?: string;
}

export async function transformCode(options: TransformerOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath, opts) => {
    const args = ['run', '-p', 'oxc_transformer', '--example', 'transformer', tempPath];
    if (opts.targets) {
      args.push('--targets', opts.targets);
    }
    return { command: 'cargo', args };
  });
}

// Compiler Tool
export interface CompilerOptions {
  sourceCode: string;
  filename?: string;
}

export async function compileCode(options: CompilerOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc', '--example', 'compiler', '--features=full', tempPath],
  }));
}

// Codegen Tool
export interface CodegenOptions {
  sourceCode: string;
  filename?: string;
}

export async function generateCode(options: CodegenOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_codegen', '--example', 'codegen', tempPath],
  }));
}

// Minifier Tool
export interface MinifierOptions {
  sourceCode: string;
  filename?: string;
}

export async function minifyCode(options: MinifierOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_minifier', '--example', 'minifier', tempPath],
  }));
}

// Dead Code Elimination Tool
export interface DCEOptions {
  sourceCode: string;
  filename?: string;
  nospace?: boolean;
  twice?: boolean;
}

export async function eliminateDeadCode(options: DCEOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath, opts) => {
    const args = ['run', '-p', 'oxc_minifier', '--example', 'dce', tempPath];
    if (opts.nospace) args.push('--nospace');
    if (opts.twice) args.push('--twice');
    return { command: 'cargo', args };
  });
}

// Mangler Tool
export interface ManglerOptions {
  sourceCode: string;
  filename?: string;
}

export async function mangleCode(options: ManglerOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_minifier', '--example', 'mangler', tempPath],
  }));
}

// Control Flow Graph Tool
export interface CFGOptions {
  sourceCode: string;
  filename?: string;
}

export async function generateCFG(options: CFGOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_semantic', '--example', 'cfg', tempPath],
  }));
}

// Isolated Declarations Tool
export interface IsolatedDeclarationsOptions {
  sourceCode: string;
  filename?: string;
}

export async function generateIsolatedDeclarations(options: IsolatedDeclarationsOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_isolated_declarations', '--example', 'isolated_declarations', tempPath],
  }));
}

// Define Tool
export interface DefineOptions {
  sourceCode: string;
  filename?: string;
  sourcemap?: boolean;
}

export async function defineCode(options: DefineOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath, opts) => {
    const args = ['run', '-p', 'oxc_transformer_plugins', '--example', 'define', tempPath];
    if (opts.sourcemap) args.push('--sourcemap');
    return { command: 'cargo', args };
  });
}

// Visitor Tool
export interface VisitorOptions {
  sourceCode: string;
  filename?: string;
}

export async function visitCode(options: VisitorOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_parser', '--example', 'visitor', tempPath],
  }));
}

// Parser TSX Tool
export interface ParserTSXOptions {
  sourceCode: string;
  filename?: string;
}

export async function parseTSXCode(options: ParserTSXOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_parser', '--example', 'parser_tsx', tempPath],
  }));
}

// Regular Expression Tool
export interface RegularExpressionOptions {
  sourceCode: string;
  filename?: string;
}

export async function parseRegularExpressions(options: RegularExpressionOptions): Promise<string> {
  return executeWithTempFile(options, (tempPath) => ({
    command: 'cargo',
    args: ['run', '-p', 'oxc_parser', '--example', 'regular_expression', tempPath],
  }));
}

// Regex Visitor Tool
export interface RegexVisitorOptions {
  pattern: string;
  flags?: string;
}

export async function visitRegex(options: RegexVisitorOptions): Promise<string> {
  const { pattern, flags = '' } = options;

  if (typeof pattern !== 'string') {
    throw new Error('pattern must be a string');
  }

  const regexContent = `/${pattern}/${flags}`;
  const tempPath = writeTempFile(regexContent, 'regex.txt');

  try {
    const result = await spawnCommand('cargo', [
      'run',
      '-p',
      'oxc_regular_expression',
      '--example',
      'regex_visitor',
      tempPath,
    ]);
    return result;
  } finally {
    try {
      unlinkSync(tempPath);
    } catch {
      // Ignore cleanup errors
    }
  }
}

// Parse Literal Tool
export interface ParseLiteralOptions {
  pattern: string;
  flags?: string;
}

export async function parseLiteral(options: ParseLiteralOptions): Promise<string> {
  const { pattern, flags = '' } = options;

  if (typeof pattern !== 'string') {
    throw new Error('pattern must be a string');
  }

  const literalContent = `/${pattern}/${flags}`;
  const tempPath = writeTempFile(literalContent, 'regex.txt');

  try {
    const result = await spawnCommand('cargo', [
      'run',
      '-p',
      'oxc_regular_expression',
      '--example',
      'parse_literal',
      tempPath,
    ]);
    return result;
  } finally {
    try {
      unlinkSync(tempPath);
    } catch {
      // Ignore cleanup errors
    }
  }
}
