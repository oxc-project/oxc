import { appendFile, appendFileSync, writeFileSync } from 'node:fs';

export const stats = {
  parse: { total: 0, count: 0 },
  stringify: { total: 0, count: 0 },
  open: { total: 0, count: 0 },
  close: { total: 0, count: 0 },
  isPromiseArray: { total: 0, count: 0 },
  isPromiseLike: { total: 0, count: 0 },
  isValidRejectionHandler: { total: 0, count: 0 },
  getProgram: { total: 0, count: 0 },
  getTypechecker: { total: 0, count: 0 },
  getNode: { total: 0, count: 0 },
  channelOverhead: { total: 0, count: 0 },
  idle: { total: 0, count: 0 },
};

const statEntries = Object.entries(stats);

writeFileSync('stats.csv', statEntries.map(([k]) => k).join(';') + '\n');

function formatDuration(x: number, scale: number) {
  return (x / scale).toFixed(3).padStart(6, ' ');
}

function formatStats() {
  const result: string[] = [];
  for (const [k, { total, count }] of statEntries) {
    result.push(
      `${formatDuration(count && total / count, 1e6)}ms / ${formatDuration(
        total,
        1e9,
      )}s`,
    );
  }

  return result.join(' ; ') + '\n';
}

setInterval(() => {
  appendFile('stats.csv', formatStats(), () => {});
}, 1000).unref();

process.on('exit', () => {
  appendFileSync('stats.csv', formatStats());
});
