import fs from 'node:fs/promises';
import { join as pathJoin } from 'node:path';

const FILE_PATH = pathJoin(import.meta.dirname, '../test_files/antd.js');

export async function getSourceBuffer() {
  const data = await fs.readFile(FILE_PATH);
  const dataUint8 = new Uint8Array(data.buffer);

  const buffer = new SharedArrayBuffer(dataUint8.length);
  const uint8 = new Uint8Array(buffer);
  uint8.set(dataUint8);
  return buffer;
}
