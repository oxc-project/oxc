// https://github.com/oxc-project/oxc/issues/26414
// In unambiguous mode, the `await` statement before a line break must not
// duplicate its operand as a separate statement when module syntax later
// commits the file to the Module goal and the statement is reparsed with
// await context enabled.
await
x
export {}

await
import('./x.js')
export {}

await
import.meta
export {}

import { x } from 'y';
await
x
import { z } from 'w';
