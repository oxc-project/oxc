import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { glob } from 'tinyglobby';

const TS_REPO_DIR = process.env.TS_REPO_DIR ?? `${import.meta.dirname}/../typescript`;
const OUTPUT_DIR = process.env.OUTPUT_DIR ?? `${import.meta.dirname}/..`;
const FILE_NAME = 'reviewed-diagnostic-error-codes';
const DEBUG = process.env.DEBUG ?? false;

// First, collect all `.errors.txt` files from `tests/baselines/reference`.
// At this point, snapshots for tests other than `compiler` and `conformance` are also included.
console.log(`Collecting all \`.errors.txt\` files from \`${TS_REPO_DIR}/tests/baselines/reference\`...`);
// NOTE: Some `.errors.txt` files are located in subdirectories, but we do not need them
const allErrorsTextPaths = await glob('*.errors.txt', {
  cwd: `${TS_REPO_DIR}/tests/baselines/reference`,
});
console.log('🍀', `Found ${allErrorsTextPaths.length} files.`);

// Each test case refers to a `.errors.txt` file with the same name as the test file.
// The `.errors.txt` file may be generated multiple times depending on the variations of `@option`.
const errorsMap = new Map<string, string[]>();
for (const errorsTextPath of allErrorsTextPaths) {
  const testId = errorsTextPathToTestId(errorsTextPath);

  const paths = errorsMap.get(testId) ?? [];
  paths.push(errorsTextPath);

  errorsMap.set(testId, paths);
}

// Now, collect all test files from `tests/cases/compiler` and `tests/cases/conformance`.
console.log('Collecting all test files from `tests/cases/compiler|conformance`...');
const allTestPaths = await glob([`compiler/**/*`, `conformance/**/*`], {
  cwd: `${TS_REPO_DIR}/tests/cases`,
});
console.log('🍀', `Found ${allTestPaths.length} files.`);

// If the test is expected to produce an error, a `.errors.txt` file should exist.
// Keep `.errors.txt` files that have a corresponding test file.
console.log('Checking each test file has `.errors.txt` files...');
const targetErrorsTextPaths = new Set<string>();
for (const testPath of allTestPaths) {
  const testId = testPathToTestId(testPath);

  const errorsTextPaths = errorsMap.get(testId);
  // These are "Expect to parse" cases
  if (!errorsTextPaths) continue;

  for (const errorsTextPath of errorsTextPaths) {
    if (!isTargetErrorsTextPath(errorsTextPath)) continue;
    targetErrorsTextPaths.add(errorsTextPath);
  }
}
console.log('🍀', `Found ${targetErrorsTextPaths.size} \`.errors.txt\` files to be checked.`);

// Finally, extract diagnostic error codes from the target `.errors.txt` files.
// Store error codes with the paths to the `.errors.txt` files they were found in.
console.log('Extracting diagnostic error codes from target `.errors.txt` files...');
const errorDiagnosticsMap = new Map<number, string[]>();
for (const errorsTextPath of targetErrorsTextPaths) {
  let errorsText: string;
  try {
    errorsText = await readFile(`${TS_REPO_DIR}/tests/baselines/reference/${errorsTextPath}`, 'utf8');
  } catch {
    console.error('💥', `Failed to read file: ${errorsTextPath}`);
    process.exit(1);
  }

  const errorCodes = extractErrorCodes(errorsText);
  for (const errorCode of errorCodes) {
    const errorsTextPaths = errorDiagnosticsMap.get(errorCode) ?? [];
    errorsTextPaths.push(errorsTextPath);

    errorDiagnosticsMap.set(errorCode, errorsTextPaths);
  }
}
console.log(
  '🍀',
  `Extracted ${errorDiagnosticsMap.size} unique diagnostic error codes from target \`.errors.txt\` files.`,
);

// Print the sorted error codes as `.txt` format.
console.log('Writing the error codes to the output file...');
const sortedErrorDiagnosticsMap = new Map(Array.from(errorDiagnosticsMap.entries()).sort(([a], [b]) => a - b));
let output = '';
let outputFile = FILE_NAME;
if (DEBUG) {
  // `.json` format for verbose output
  output = JSON.stringify(
    Object.fromEntries(Array.from(sortedErrorDiagnosticsMap.entries()).map(([code, paths]) => [code, paths.sort()])),
    null,
    2,
  );
  outputFile += '.json';
} else {
  // `.txt` format to avoid trailing commas
  output = [
    'If you are seeing this file with Git diffs, please follow the instructions below to review the changes.',
    '',
    'Q. Is it possible for `oxc_parser` to detect and report this error during static parsing?',
    '- Yes: No action required, just commit the change. (Create an issue for that will be helpful.)',
    '- No: Please add it to the `NOT_SUPPORTED_ERROR_CODES` list in `constants.rs` with an example error message, and commit the change.',
    '-----',
    ...sortedErrorDiagnosticsMap.keys(),
  ].join('\n');
  outputFile += '.txt';
}
const outputPath = `${OUTPUT_DIR}/${outputFile}`;
await writeFile(outputPath, output, 'utf8');
console.log('🍀', `Saved the output to ${outputPath}`);

// ---

// Extracts a test ID from a given test path.
// Path can be like:
// - compiler/jsxContainsOnlyTriviaWhiteSpacesNotCountedAsChild.tsx
// - conformance/parser/ecmascript5/Statements/BreakStatements/parser_breakTarget2.ts
// - conformance/es6/decorators/class/property/decoratorOnClassProperty1.es6.ts
function testPathToTestId(testPath: string): string {
  const basename = path.basename(testPath);
  const [testId] = basename.split(path.extname(basename));
  return testId;
}

// Extracts a test ID from a given text path.
// Path can be like:
// - importDeferTypeConflict2.errors.txt
// - asyncGeneratorParameterEvaluation(target=es2018).errors.txt
// - project/maprootUrlModuleSimpleSpecifyOutputFile/node/maprootUrlModuleSimpleSpecifyOutputFile.errors.txt
function errorsTextPathToTestId(errorsTextPath: string): string {
  const basename = path.basename(errorsTextPath);
  const [testIdPart] = basename.split('.errors.txt');
  const [testId] = testIdPart.split('(');
  return testId;
}

// If path contains variations, we want to keep specific variations only.
// module, target, jsx, experimentaldecorators
// These are the same as OXC's typescript coverage tests.
function isTargetErrorsTextPath(errorsTextPath: string) {
  const hasVariations = errorsTextPath.endsWith(').errors.txt');
  if (!hasVariations) return true;

  // This is synced with:
  // https://github.com/oxc-project/oxc/blob/ee035b49c6e60902939679dae53203192f080a66/tasks/coverage/src/typescript/meta.rs#L231-L241
  const supportedVariant = [
    'module=',
    'target=',
    'jsx=',
    'preserveconstenums=',
    'usedefineforclassfields=',
    'experimentaldecorators=',
  ];
  if (supportedVariant.some((option) => errorsTextPath.includes(option))) return true;

  return false;
}

// Extracts error diagnostic codes from the content of `.errors.txt` file.
// The file is expected to contain a summary and details of errors.
//
// The summary contains lines like:
// - ArrowFunction3.ts(1,12): error TS1005: ',' expected.
// - error TS2688: Cannot find type definition file for 'react'.
// The details contain lines like:
// - !!! error TS1005: '}' expected.
function extractErrorCodes(errorsText: string): Set<number> {
  const errorDiagnosticsCodes = new Set<number>();

  // @ts-expect-error: `v` flag
  const matches = errorsText.matchAll(/error TS(?<code>\d{4,5}): /gv);
  for (const match of matches) {
    const code = match.groups?.code;
    if (code) errorDiagnosticsCodes.add(Number(code));
  }

  return errorDiagnosticsCodes;
}
