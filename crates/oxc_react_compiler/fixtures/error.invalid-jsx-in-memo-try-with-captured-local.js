// @validateNoJSXInTryStatements @outputMode:"lint"
import { useMemo } from 'react';

function Component() {
  return useMemo(() => {
    let cached;
    try {
      return <div onClick={() => { cached = 1; }} />;
    } catch {
      return null;
    }
  }, []);
}

function Fragment() {
  return useMemo(() => {
    let cached;
    try {
      return <>{() => (cached = 1)}</>;
    } catch {
      return null;
    }
  }, []);
}

function AllowedCatch() {
  return useMemo(() => {
    let cached;
    try {
      return read();
    } catch {
      return <div onClick={() => { cached = 1; }} />;
    }
  }, []);
}

function EnclosingTry() {
  try {
    return useMemo(() => {
      let cached;
      return <div onClick={() => { cached = 1; }} />;
    }, []);
  } catch {
    return null;
  }
}

function ReturnedCallback() {
  return useMemo(() => {
    let cached;
    return () => {
      try {
        return (cached ??= <div />);
      } catch {
        return null;
      }
    };
  }, []);
}

function Unreachable() {
  if (false) {
    return useMemo(() => {
      let cached;
      try {
        return <div onClick={() => { cached = 1; }} />;
      } catch {
        return null;
      }
    }, []);
  }
  return null;
}
