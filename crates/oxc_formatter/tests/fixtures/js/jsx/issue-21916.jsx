// A group in a fill item must not expand when the item and its flat-decided separator fit;
// the re-measure after the `<x />` hard line has to see the separator as `" "`, not `{" "}`.
const App = () => {
  return (
    <O>
      <I>
        <x />
        {f(xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx).y} (
        {label})
      </I>
    </O>
  );
};

// From the issue comments: same shape at printWidth 100 with a call argument that would
// otherwise be broken apart.
export const Foo = () => (
  <div>
    <div>
      <div>
        <div>
          {cond ? (
            <>a</>
          ) : (
            <>
              foo bar baz:{' '}
              {formatBytes(x)} · qux:{' '}
              {formatBytes(y)}
              <br />
              Total: {formatBytes(subject.value.context.numberA + subject.value.context.amountAB)} (
              {compressed ? 'compressed' : 'uncompressed'})
            </>
          )}
        </div>
      </div>
    </div>
  </div>
);
