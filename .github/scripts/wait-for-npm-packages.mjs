#!/usr/bin/env node
/* eslint-disable no-await-in-loop, no-console */

import fs from "node:fs";
import path from "node:path";

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

async function waitForPackage(packageDir) {
  const { name, version } = JSON.parse(
    fs.readFileSync(path.join(packageDir, "package.json"), "utf8"),
  );
  const spec = `${name}@${version}`;
  const metadataUrl = `https://registry.npmjs.org/${encodeURIComponent(name)}/${encodeURIComponent(version)}`;
  const deadline = Date.now() + 24 * 60 * 60 * 1000;

  while (Date.now() < deadline) {
    try {
      const metadata = await fetch(metadataUrl, {
        headers: { "cache-control": "no-cache" },
        signal: AbortSignal.timeout(30_000),
      });
      if (!metadata.ok) throw new Error();

      const tarball = await fetch((await metadata.json()).dist.tarball, {
        headers: { "cache-control": "no-cache", range: "bytes=0-0" },
        signal: AbortSignal.timeout(30_000),
      });
      await tarball.body?.cancel();
      if (!tarball.ok) throw new Error();

      console.log(`✓ ${spec} is available.`);
      return;
    } catch {
      await sleep(10_000);
    }
  }

  throw new Error(`Timed out waiting for ${spec}.`);
}

Promise.all(process.argv.slice(2).map(waitForPackage)).catch((error) => {
  console.error(`::error::${error.message}`);
  process.exit(1);
});
