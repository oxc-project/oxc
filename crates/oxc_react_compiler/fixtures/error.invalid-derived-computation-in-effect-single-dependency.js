// @validateNoDerivedComputationsInEffects
import {useEffect, useState} from 'react';

function Component({value}) {
  const [normalized, setNormalized] = useState('');
  useEffect(() => {
    setNormalized(value.trim());
  }, [value]);
  return <div>{normalized}</div>;
}
