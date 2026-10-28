import {useMotionTemplate} from 'framer-motion';

function Component() {
  const callback = () => useMotionTemplate`static`;
  return callback;
}
