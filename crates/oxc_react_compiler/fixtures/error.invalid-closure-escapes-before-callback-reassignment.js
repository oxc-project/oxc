import {useEffect} from 'react';

function Component() {
  let count = 0;
  let cb = () => null;
  useEffect(() => cb(), []);
  cb = () => count++;
  return null;
}
