import * as noFloatingPromises from './rules/no-floating-promises.js';
import { FileRequest, NodeRequest, OpenRequest } from './protocol.js';
import { service } from './typecheck/createProjectService.js';
import { useProgramFromProjectService } from './typecheck/useProgramFromProjectService.js';
import {
  deleteNodeCache,
  getNodeAtPosition,
} from './typecheck/getNodeAtPosition.js';
import ts from 'typescript';
import { stats } from './stats.js';

export const handlers: Record<string, (req: any) => Result> = {
  status: () => {
    const response = { version: '0.1.0' };
    return requiredResponse(response);
  },
  exit: () => {
    process.exit(0);
  },
  open: ({ arguments: { file, fileContent } }: OpenRequest) => {
    measure(() => service.openClientFile(file, fileContent, undefined), 'open');
    return notRequired();
  },
  close: ({ arguments: { file } }: FileRequest) => {
    measure(() => {
      service.closeClientFile(file);
      deleteNodeCache(file);
    }, 'close');
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
    const program = measure(
      () => useProgramFromProjectService(service, file),
      'getProgram',
    );
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = measure(() => getNodeAtPosition(program.ast, span), 'getNode');
    const checker = measure(
      () => program.program.getTypeChecker(),
      'getTypechecker',
    );

    const result = measure(
      () => noFloatingPromises.isPromiseArray(checker, node),
      'isPromiseArray',
    );
    return requiredResponse({ result });
  },
  'noFloatingPromises::isPromiseLike': ({
    arguments: { file, span },
  }: NodeRequest) => {
    const program = measure(
      () => useProgramFromProjectService(service, file),
      'getProgram',
    );
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = measure(() => getNodeAtPosition(program.ast, span), 'getNode');
    const checker = measure(
      () => program.program.getTypeChecker(),
      'getTypechecker',
    );

    const result = measure(
      () => noFloatingPromises.isPromiseLike(checker, node),
      'isPromiseLike',
    );
    return requiredResponse({ result });
  },
  'noFloatingPromises::isValidRejectionHandler': ({
    arguments: { file, span },
  }: NodeRequest) => {
    const program = measure(
      () => useProgramFromProjectService(service, file),
      'getProgram',
    );
    if (!program) {
      throw new Error('failed to create TS program');
    }

    const node = measure(() => getNodeAtPosition(program.ast, span), 'getNode');
    const checker = measure(
      () => program.program.getTypeChecker(),
      'getTypechecker',
    );

    const result = measure(
      () => noFloatingPromises.isValidRejectionHandler(checker, node),
      'isValidRejectionHandler',
    );
    return requiredResponse({ result });
  },
};

function measure<R>(f: () => R, key: keyof typeof stats): R {
  const start = process.hrtime.bigint();

  const result = f();

  const duration = process.hrtime.bigint() - start;
  stats[key].total += Number(duration);
  stats[key].count += 1;

  return result;
}

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
