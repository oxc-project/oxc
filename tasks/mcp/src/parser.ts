import { executeWithTempFile } from './exec.js';

export interface ParseOptions {
  sourceCode: string;
  filename?: string;
  showAst?: boolean;
  showEstree?: boolean;
  showComments?: boolean;
}

export async function parse(options: ParseOptions): Promise<string> {
  const args = ['run', '-p', 'oxc_parser', '--example', 'parser'];
  if (options.showAst) {
    args.push('--ast');
  }
  if (options.showEstree) {
    args.push('--estree');
  }
  if (options.showComments) {
    args.push('--comments');
  }
  return executeWithTempFile('cargo', options, args);
}
