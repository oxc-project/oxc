import { spawn } from 'child_process';

export function spawnCommand(command: string, args: string[]): Promise<string> {
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
