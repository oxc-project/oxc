// @compilationMode:"infer" @panicThreshold:"none"
import {useEffect, useState} from 'react';

export function Component({source}) {
  const callback = (setData: (data: string) => void) => setData(source);
  const data = useState(source)[0];
  useEffect(() => callback(() => data), [data]);
  return data;
}
