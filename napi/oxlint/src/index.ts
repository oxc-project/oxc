import { lint } from './binding';

const result = lint();

if (!result) {
  process.exit(1);
}
