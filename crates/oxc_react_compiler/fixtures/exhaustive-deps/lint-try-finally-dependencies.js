// @validateExhaustiveEffectDependencies:"all" @validateNoVoidUseMemo @outputMode:"lint" @panicThreshold:"none"
import {useCallback, useEffect, useMemo, useState} from 'react';

// A `finally` clause is not compiled, but lint rules still see the values that
// the `try` and `finally` blocks use. Only `MissingFinallyDependency` is invalid.
function TryBody() {
  const [foo] = useState('bar');
  useEffect(() => {
    try {
      report(foo);
    } finally {
    }
  }, [foo]);
}

function FinallyBody() {
  const [foo] = useState('bar');
  const cb = useCallback(() => {
    try {
      report('x');
    } finally {
      report(foo);
    }
  }, [foo]);
  return <button onClick={cb} />;
}

function FinallyWithCatch() {
  const [foo] = useState('bar');
  const cb = useCallback(() => {
    try {
      report('x');
    } catch {
      report('y');
    } finally {
      report(foo);
    }
  }, [foo]);
  return <button onClick={cb} />;
}

function ReturnInTry({foo}) {
  const value = useMemo(() => {
    try {
      return foo;
    } finally {
      report('z');
    }
  }, [foo]);
  return <div>{value}</div>;
}

function MissingFinallyDependency() {
  const [foo] = useState('bar');
  useEffect(() => {
    try {
      report('x');
    } finally {
      report(foo);
    }
  }, []);
}
