import { glob } from 'glob';
import type { RolldownOptions } from 'rolldown';

export default async (): Promise<RolldownOptions> => {
  const input = await glob('tests/*.spec.ts');
  return {
    input,
    output: {
      dir: 'out',
      sourcemap: true,
      format: 'cjs',
      banner: `"use strict";\n`,
    },
    external: ['vscode'],
    platform: 'node',
    transform: {
      target: 'node16',
    },
  };
};
