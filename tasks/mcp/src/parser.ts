import { writeFileSync, unlinkSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

import { spawnCommand } from './spawn'

export interface ParseOptions {
  sourceCode: string;
  filename?: string;
  showAst?: boolean;
  showEstree?: boolean;
  showComments?: boolean;
  additionalArgs?: string[];
}

export async function parseCode(options: ParseOptions): Promise<string> {
  const {
    sourceCode,
    filename = 'input.js',
    showAst = false,
    showEstree = false,
    showComments = false,
  } = options;

  if (typeof sourceCode !== 'string') {
    throw new Error('sourceCode must be a string');
  }

  try {
    // Create a temporary file with the source code
    const tmpFile = join(tmpdir(), `oxc-mcp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}-${filename}`);
    writeFileSync(tmpFile, sourceCode, 'utf8');

    try {
      // Build the cargo command arguments
      const cargoArgs = ['run', '-p', 'oxc_parser', '--example', 'parser', tmpFile];

      if (showAst) {
        cargoArgs.push('--ast');
      }
      if (showEstree) {
        cargoArgs.push('--estree');
      }
      if (showComments) {
        cargoArgs.push('--comments');
      }

      // Spawn the cargo command
      const result = await spawnCommand('cargo', cargoArgs);

      return result;
    } finally {
      // Clean up temporary file
      try {
        unlinkSync(tmpFile);
      } catch {
        // Ignore cleanup errors
      }
    }
  } catch (error) {
    throw new Error(`Parse error: ${error instanceof Error ? error.message : String(error)}`);
  }
}

