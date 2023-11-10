import path from 'path';
import resolve, { ResolverFactory } from './index.js';
import assert from 'assert';

console.log(`Testing on ${process.platform}-${process.arch}`)

const cwd = process.cwd();

// `resolve`
assert(resolve.sync(cwd, "./index.js").path, path.join(cwd, 'index.js'));

// `ResolverFactory`
const resolver = new ResolverFactory();
assert(resolver.sync(cwd, "./index.js").path, path.join(cwd, 'index.js'));
