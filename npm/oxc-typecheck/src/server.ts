// Closely mimics tsserver: https://github.com/microsoft/TypeScript/blob/e2bf8b437d063392264ef20c55076cf0922ea2b6/src/server/session.ts#L3631

import { createInterface } from 'node:readline';
import { EOL } from 'node:os';
import type { Message, Request, Response } from './protocol.js';
import { Queue } from './queue.js';
import { Result, handlers } from './handlers.js';
import { stats } from './stats.js';

const writeQueue = new Queue<Buffer>();
let canWrite = true;
let previousDuration: number = 0;
let idleStart = 0n;

export function startServer() {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  rl.on('line', (input: string) => {
    const start = process.hrtime.bigint();

    stats.idle.count++;
    stats.idle.total += Number(start - idleStart);

    const message = input.trim();
    onMessage(message, start);
    idleStart = process.hrtime.bigint();
  });
  rl.on('close', () => {
    process.exit(0);
  });
  idleStart = process.hrtime.bigint();
}

function onMessage(message: string, start: bigint): void {
  let request: Request | undefined;
  try {
    request = JSON.parse(message) as Request;
    if (previousDuration) {
      stats.channelOverhead.count++;
      stats.channelOverhead.total +=
        request.previousDuration - previousDuration;
    }

    const { response, responseRequired } = executeCommand(request, start);
    const strStart = process.hrtime.bigint();
    if (response) {
      doOutput(response, request.command, request.seq, true, start, strStart);
    } else if (responseRequired) {
      doOutput(
        undefined,
        request.command,
        request.seq,
        false,
        start,
        strStart,
        'No content available.',
      );
    } else {
      doOutput({}, request.command, request.seq, true, start, strStart);
    }
  } catch (err) {
    const strStart = process.hrtime.bigint();
    doOutput(
      undefined,
      request ? request.command : 'unknown',
      request ? request.seq : 0,
      false,
      start,
      strStart,
      'Error processing request. ' +
        (err as Error).message +
        '\n' +
        (err as Error).stack,
    );
  }
}

function executeCommand(request: Request, start: bigint): Result {
  const handler = handlers[request.command];
  if (handler) {
    stats.parse.total += Number(process.hrtime.bigint() - start);
    stats.parse.count++;
    const response = handler(request);
    return response;
  } else {
    const strStart = process.hrtime.bigint();
    doOutput(
      undefined,
      'unknown',
      request.seq,
      false,
      start,
      strStart,
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
  start: bigint,
  strStart: bigint,
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

  send(res, start, strStart);
}

function send(msg: Message, start: bigint, strStart: bigint): void {
  const json = JSON.stringify(msg);
  const len = Buffer.byteLength(json, 'utf8');
  const msgString = `Content-Length: ${1 + len}\r\n\r\n${json}${EOL}`;
  writeMessage(Buffer.from(msgString, 'utf8'));
  const end = process.hrtime.bigint();
  stats.stringify.count++;
  stats.stringify.total += Number(end - strStart);
  previousDuration = Number(end - start);
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
