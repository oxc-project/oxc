// Regression test: a hook used as a tagged template tag is a *call*, not a
// value reference, so it must not trigger "Hooks may not be referenced as
// normal values". Idiomatic framer-motion `useMotionTemplate` usage.
// https://github.com/oxc-project/oxc/issues/24473
import {useMotionValue, useMotionTemplate} from 'framer-motion';

function Transform() {
  const x = useMotionValue(100);
  const transform = useMotionTemplate`transform(${x}px)`;
  return <div style={{transform}} />;
}
