import { useEffect, useRef } from "@wordpress/element";

export function useLatest<T>(value: T) {
  const latest = useRef(value);
  useEffect(() => {
    latest.current = value;
  });
  return () => latest.current;
}

export function Probe() {
  const clicks = useRef(0);
  return (
    <button
      onClick={() => {
        clicks.current += 1;
      }}
    />
  );
}

export function useLatestRef<T>(value: T) {
  const latestRef = useRef(value);
  useEffect(() => {
    latestRef.current = value;
  });
  return () => latestRef.current;
}

export function ProbeRef() {
  const clicksRef = useRef(0);
  return (
    <button
      onClick={() => {
        clicksRef.current += 1;
      }}
    />
  );
}

export function OtherProperty({ value }: { value: { other: boolean } }) {
  value.other = true;
  return <div />;
}
