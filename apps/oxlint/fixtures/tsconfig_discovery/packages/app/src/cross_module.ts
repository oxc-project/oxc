// This file tests cross-module imports
// Should resolve correctly with app's tsconfig
import { libFunction } from '@lib/index';
import { appHelper } from '@app/helper';

// This should fail - @root is not defined in app's tsconfig
import { rootUtil } from '@root/utils';

export function crossModuleTest() {
  console.log(libFunction());
  console.log(appHelper());
  console.log(rootUtil());
}