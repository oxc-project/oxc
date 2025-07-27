// This file should use the root tsconfig
import { rootUtil } from '@root/utils';

// This import should fail because @lib is not defined in root tsconfig
import { libFunction } from '@lib/index';

export function testRoot() {
  return rootUtil();
}