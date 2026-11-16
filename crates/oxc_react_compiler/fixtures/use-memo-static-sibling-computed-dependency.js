import {useEffect, useMemo} from 'react';

export function A({id}) {
  const items = useMemo(() => [{label: 'Home'}, {label: `#${id}`}], [id]);
  return <List items={items} />;
}

export function Binary({id}) {
  return useMemo(() => [{label: 'Home'}, {label: '#' + id}], [id]);
}

export function Unary({id}) {
  return useMemo(() => [{label: 'Home'}, {label: -id}], [id]);
}

export function ObjectWithMap({id}) {
  return useMemo(() => ({entries: new Map(), key: `k-${id}`}), [id]);
}

export function MapWithEffect({id}) {
  const cache = useMemo(() => ({entries: new Map(), key: `k-${id}`}), [id]);
  useEffect(() => {
    cache.entries.set(cache.key, true);
  }, [cache]);
  return <div>{cache.key}:{cache.entries.size}</div>;
}

export function NestedProperty({id}) {
  return useMemo(() => ({entries: new Map(), key: `k-${id.name}`}), [id.name]);
}

export function MultipleProperties({id}) {
  return useMemo(() => ({entries: new Map(), key: id.prefix + id.name}), [id.prefix, id.name]);
}

export function ComputedProperty({id, field}) {
  return useMemo(() => ({entries: new Map(), key: id[field]}), [id, field]);
}

export function RepeatedOperand({id}) {
  return useMemo(() => [{label: 'Home'}, {label: '#' + (id + id)}], [id]);
}

export function MultipleStaticScopes({id}) {
  return useMemo(() => [{label: 'Home'}, {label: 'Other'}, {label: `#${id}`}], [id]);
}

export function LocalAlias({id}) {
  return useMemo(() => {
    const home = {label: 'Home'};
    const label = `#${id}`;
    return [home, {label}];
  }, [id]);
}

export function Property({id}) {
  return useMemo(() => [{label: 'Home'}, {label: id.name}], [id.name]);
}

export function Reversed({id}) {
  return useMemo(() => [{label: `#${id}`}, {label: 'Home'}], [id]);
}

export const FIXTURE_ENTRYPOINT = {
  fn: MapWithEffect,
  params: [{id: 'a'}],
  sequentialRenders: [{id: 'a'}, {id: 'a'}, {id: 'b'}, {id: 'b'}],
};
