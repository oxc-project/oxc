#!/usr/bin/env node

import { format } from './bindings.js';
import { formatEmbeddedCode } from './embedded.js';

const args = process.argv.slice(2);

// Call the Rust formatter with our JS callback
const success = await format(args, formatEmbeddedCode);

process.exit(success ? 0 : 1);
