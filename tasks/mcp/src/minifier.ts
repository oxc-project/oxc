import { executeWithTempFile } from './exec.js';

export interface MinifyOptions {
  sourceCode: string;
  filename?: string;
  mangle?: boolean;
  nospace?: boolean;
}

export async function minify(options: MinifyOptions): Promise<string> {
  const args = ['run', '-p', 'oxc_minifier', '--example', 'minifier'];
  if (options.mangle) {
    args.push('--mangle');
  }
  if (options.nospace) {
    args.push('--nospace');
  }
  return executeWithTempFile('cargo', options, args);
}
