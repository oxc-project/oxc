import * as noFloatingPromises from './rules/no-floating-promises.js';
import { FileRequest, NodeRequest, OpenRequest } from './protocol.js';
import { service } from './typecheck/createProjectService.js';
import { useProgramFromProjectService } from './typecheck/useProgramFromProjectService.js';
import { getNodeAtPosition } from './typecheck/getNodeAtPosition.js';
import ts from 'typescript';

export const handlers: Record<string, (req: any) => Result> = {
  status: () => {
    const response = { version: '0.1.0' };
    return requiredResponse(response);
  },
  exit: () => {
    process.exit(0);
  },
  open: ({ arguments: { file, fileContent } }: OpenRequest) => {
    service.openClientFile(file, fileContent, undefined);
    return notRequired();
  },
  close: ({ arguments: { file } }: FileRequest) => {
    service.closeClientFile(file);
    return notRequired();
  },
  getNode: ({ arguments: { file, span } }: NodeRequest) => {
    const program = useProgramFromProjectService(service, file);
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = getNodeAtPosition(program.ast, span);

    const checker = program.program.getTypeChecker();
    const type = checker.getTypeAtLocation(node);

    return requiredResponse({
      kind: ts.SyntaxKind[node.kind],
      text: node.getText(),
      type: checker.typeToString(type),
      symbol: type.symbol?.name,
    });
  },
  'noFloatingPromises::isPromiseArray': ({
    arguments: { file, span },
  }: NodeRequest) => {
    const program = useProgramFromProjectService(service, file);
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = getNodeAtPosition(program.ast, span);
    const checker = program.program.getTypeChecker();

    const result = noFloatingPromises.isPromiseArray(checker, node);
    return requiredResponse({ result });
  },
  'noFloatingPromises::isPromiseLike': ({
    arguments: { file, span },
  }: NodeRequest) => {
    const program = useProgramFromProjectService(service, file);
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = getNodeAtPosition(program.ast, span);
    const checker = program.program.getTypeChecker();

    const result = noFloatingPromises.isPromiseLike(checker, node);
    return requiredResponse({
      result,
    });
  },
};

export interface Result {
  response?: {};
  responseRequired: boolean;
}

function requiredResponse(response: {}): Result {
  return { response, responseRequired: true };
}

function notRequired(): Result {
  return { responseRequired: false };
}
