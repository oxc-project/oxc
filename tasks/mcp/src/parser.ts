import { spawn } from 'child_process';
import { writeFileSync, unlinkSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

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
    additionalArgs = []
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

      // Pass all remaining args into the command
      cargoArgs.push(...additionalArgs);

      // Spawn the cargo command
      const result = await spawnCommand('cargo', cargoArgs);
      
      return result;
    } finally {
      // Clean up temporary file
      try {
        unlinkSync(tmpFile);
      } catch (e) {
        // Ignore cleanup errors
      }
    }
  } catch (error) {
    throw new Error(`Parse error: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function spawnCommand(command: string, args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const process = spawn(command, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: '/home/runner/work/oxc/oxc',  // Set working directory to the oxc repository root
    });

    let stdout = '';
    let stderr = '';

    process.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    process.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    process.on('close', (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(`Command failed with exit code ${code}:\n${stderr}`));
      }
    });

    process.on('error', (error) => {
      reject(new Error(`Failed to spawn command: ${error.message}`));
    });
  });
}