// This file is in tests/ but should still use lib's tsconfig
import { libFunction } from '@lib/index';
import { helper } from '@lib/helper';

// Test that would fail if wrong tsconfig is used
const result = libFunction();
const helperResult = helper();