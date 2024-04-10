// Types matching tsserver: https://github.com/microsoft/TypeScript/blob/25a708cf633c6c8a66c86ca9e664c31bd8d145d0/src/server/protocol.ts#L182-L276

export interface Request {
  command: string;
  seq: number;
  arguments?: {};
}

export interface Message {
  seq: number;
  type: 'response' | 'event';
}

export interface Response extends Message {
  seq: number;
  type: 'response';
  command: string;
  request_seq: number;
  success: boolean;
  body?: {};
  message?: string;
}

export interface Event extends Message {
  seq: number;
  type: 'event';
  event: string;
  body: {};
}

export interface OpenRequest extends Request {
  command: 'open';
  arguments: {
    file: string;
    fileContent?: string;
  };
}

export interface FileRequest extends Request {
  arguments: {
    file: string;
  };
}

export interface NodeRequest extends Request {
  command: 'node';
  arguments: {
    file: string;
    line: number;
    col: number;
    kind: string;
  };
}

export interface LocationRequest extends Request {
  arguments: {
    file: string;
    line: number;
    col: number;
  };
}
