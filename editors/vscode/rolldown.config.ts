import type { RolldownOptions } from 'rolldown';

export default (): RolldownOptions => {
  return {
    input: 'client/extension.ts',
    output: {
      file: 'out/main.js',
      sourcemap: true,
      format: 'cjs',
      banner: `"use strict";\n`,
      minify: true,
    },
    external: ['vscode'],
    platform: 'node',
    transform: {
      target: 'node16',
    },
  };
};
