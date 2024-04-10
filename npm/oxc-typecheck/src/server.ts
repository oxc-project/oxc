// Closely mimics tsserver: https://github.com/microsoft/TypeScript/blob/e2bf8b437d063392264ef20c55076cf0922ea2b6/src/server/session.ts#L3631

import { createInterface } from 'node:readline';
import { EOL } from 'node:os';
import type { Message, Request, Response, Event } from './protocol.js';
import { Queue } from './queue.js';
import { Result, handlers } from './handlers.js';

const writeQueue = new Queue<Buffer>();
let canWrite = true;

export function startServer() {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  rl.on('line', (input: string) => {
    const message = input.trim();
    onMessage(message);
  });
  rl.on('close', () => {
    process.exit(0);
  });
}

function onMessage(message: string): void {
  let request: Request | undefined;
  try {
    request = JSON.parse(message) as Request;
    const { response, responseRequired } = executeCommand(request);
    if (response) {
      doOutput(response, request.command, request.seq, true);
    } else if (responseRequired) {
      doOutput(
        undefined,
        request.command,
        request.seq,
        false,
        'No content available.',
      );
    }
  } catch (err) {
    doOutput(
      undefined,
      request ? request.command : 'unknown',
      request ? request.seq : 0,
      false,
      'Error processing request. ' +
        (err as Error).message +
        '\n' +
        (err as Error).stack,
    );
  }
}

function executeCommand(request: Request): Result {
  const handler = handlers[request.command];
  if (handler) {
    const response = handler(request);
    return response;
  } else {
    doOutput(
      undefined,
      'unknown',
      request.seq,
      false,
      `Unrecognized JSON command: ${request.command}`,
    );
    return { responseRequired: false };
  }
}

function doOutput(
  response: {} | undefined,
  command: string,
  seq: number,
  success: boolean,
  message?: string,
): void {
  const res: Response = {
    seq: 0,
    type: 'response',
    command,
    request_seq: seq,
    success,
  };

  if (success) {
    res.body = response;
  }

  if (message) {
    res.message = message;
  }

  send(res);
}

function emitEvent(eventName: string, body: {}): void {
  const event: Event = {
    seq: 0,
    type: 'event',
    event: eventName,
    body,
  };

  send(event);
}

function send(msg: Message): void {
  const json = JSON.stringify(msg);
  const len = Buffer.byteLength(json, 'utf8');
  const msgString = `Content-Length: ${1 + len}\r\n\r\n${json}${EOL}`;
  writeMessage(Buffer.from(msgString, 'utf8'));
}

function writeMessage(buf: Buffer): void {
  if (!canWrite) {
    writeQueue.enqueue(buf);
  } else {
    canWrite = false;
    process.stdout.write(buf, writeMessageCallback);
  }
}

function writeMessageCallback() {
  canWrite = true;
  if (!writeQueue.isEmpty()) {
    writeMessage(writeQueue.dequeue());
  }
}

startServer();
