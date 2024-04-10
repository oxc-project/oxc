// Source: https://github.com/typescript-eslint/typescript-eslint/blob/a41ad155b5fee9177651439adb1c5131e7e6254f/packages/typescript-estree/src/create-program/createProjectService.ts

import ts from 'typescript';

const noop = (): void => {};

const createStubFileWatcher = (): ts.FileWatcher => ({
  close: noop,
});

export type TypeScriptProjectService = ts.server.ProjectService;

export function createProjectService(): TypeScriptProjectService {
  const system: ts.server.ServerHost = {
    ...ts.sys,
    clearImmediate,
    clearTimeout,
    setImmediate,
    setTimeout,
    watchDirectory: createStubFileWatcher,
    watchFile: createStubFileWatcher,
  };

  const service = new ts.server.ProjectService({
    host: system,
    cancellationToken: { isCancellationRequested: (): boolean => false },
    useSingleInferredProject: false,
    useInferredProjectPerProjectRoot: false,
    logger: {
      close: noop,
      endGroup: noop,
      getLogFileName: () => undefined,
      ...(process.env.DEBUG === 'true'
        ? {
            hasLevel: () => true,
            info: (...args) => console.error(...args),
            loggingEnabled: () => true,
            msg: (...args) => console.error(...args),
          }
        : {
            hasLevel: () => false,
            info: noop,
            loggingEnabled: () => false,
            msg: noop,
          }),
      perftrc: noop,
      startGroup: noop,
    },
    session: undefined,
    jsDocParsingMode: ts.JSDocParsingMode.ParseForTypeInfo,
  });

  return service;
}

export const service = createProjectService();
