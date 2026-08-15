// @compilationMode:"infer" @panicThreshold:"none"
import {useState} from 'react';

export function Component() {
  const [value, setValue] = useState(BigInt('123'));
  return <input value={value} onChange={setValue} step={BigInt(10)} />;
}
