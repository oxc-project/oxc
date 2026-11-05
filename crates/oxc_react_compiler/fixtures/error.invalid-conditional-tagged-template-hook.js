import {useMotionTemplate} from 'framer-motion';

function Component({cond}) {
  let value;
  if (cond) {
    value = useMotionTemplate`static`;
  }
  return value;
}
