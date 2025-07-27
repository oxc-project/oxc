// This file should fail to import @app because lib's tsconfig doesn't define it
import { appHelper } from '@app/helper';

export function badImport() {
  return appHelper();
}