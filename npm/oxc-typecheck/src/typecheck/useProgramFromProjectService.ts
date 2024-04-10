// Source: https://github.com/typescript-eslint/typescript-eslint/blob/a41ad155b5fee9177651439adb1c5131e7e6254f/packages/typescript-estree/src/useProgramFromProjectService.ts

import type ts from 'typescript';
import { TypeScriptProjectService } from './createProjectService.js';

export interface ASTAndDefiniteProgram {
  ast: ts.SourceFile;
  program: ts.Program;
}

export function useProgramFromProjectService(
  service: TypeScriptProjectService,
  filePath: string,
): ASTAndDefiniteProgram | undefined {
  const scriptInfo = service.getScriptInfo(filePath);
  const program = service
    .getDefaultProjectForFile(scriptInfo!.fileName, true)!
    .getLanguageService(/*ensureSynchronized*/ true)
    .getProgram();

  if (!program) {
    return undefined;
  }

  const ast = program.getSourceFile(filePath);
  return ast && { ast, program };
}
