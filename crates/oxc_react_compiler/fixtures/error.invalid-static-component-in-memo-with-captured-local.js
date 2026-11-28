// @validateStaticComponents @outputMode:"lint"
import { useMemo } from 'react';

function Component() {
  return useMemo(() => {
    let cached;
    const Dynamic = () => <div onClick={() => { cached = 1; }} />;
    return <Dynamic />;
  }, []);
}

function CapturedComponent() {
  const Dynamic = () => <div />;
  return useMemo(() => {
    let cached;
    return <Dynamic onClick={() => { cached = 1; }} />;
  }, [Dynamic]);
}

function Stable({ onClick }) {
  return <div onClick={onClick} />;
}

function AllowedGlobal() {
  return useMemo(() => {
    let cached;
    return <Stable onClick={() => { cached = 1; }} />;
  }, []);
}

function ReturnedCallback() {
  return useMemo(() => {
    let cached;
    return () => {
      const Dynamic = () => <div onClick={() => { cached = 1; }} />;
      return <Dynamic />;
    };
  }, []);
}
