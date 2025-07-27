// This file uses the app's tsconfig paths
import { libFunction } from '@lib/index';
import { appHelper } from '@app/helper';

export function main() {
  console.log(libFunction());
  console.log(appHelper());
}