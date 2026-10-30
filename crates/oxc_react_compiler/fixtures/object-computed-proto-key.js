// A computed `["__proto__"]` key defines an own property, while the
// non-computed spellings set the object's prototype.
function Component({value}) {
  const own = {['__proto__']: value};
  const prototype = {'__proto__': value};
  const shorthandProto = {__proto__: value};
  return (
    <div>
      {Object.keys(own).join(',')}|{String(Object.getPrototypeOf(own) === value)}|
      {Object.keys(prototype).join(',')}|
      {String(Object.getPrototypeOf(prototype) === value)}|
      {String(Object.getPrototypeOf(shorthandProto) === value)}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: {x: 1}}],
};
