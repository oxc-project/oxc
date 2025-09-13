import type { RolldownOptions } from 'rolldown';
import * as glob from 'glob';

export default (): RolldownOptions => {
  return {
    input: glob.sync('tests/*.spec.ts'),
    output: {
      dir: 'out',
      sourcemap: true,
      format: 'cjs',
      banner: `"use strict";\n`,
    },
    external: ['vscode'],
    platform: 'node',
    transform: {
      target: 'node16'
    }
  }
}
