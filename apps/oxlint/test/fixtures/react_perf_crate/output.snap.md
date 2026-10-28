# Exit code
1

# stdout
```
  x react-perf(jsx-no-jsx-as-prop): JSX attribute values should not contain other JSX.
   ,-[files/index.jsx:2:22]
 1 | export function Example() {
 2 |   return <Item node={<Child />} items={[]} callback={() => {}} options={{}} />;
   :                      ^^^^^^^^^
 3 | }
   `----
  help: simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).

  x react-perf(jsx-no-new-array-as-prop): JSX attribute values should not contain Arrays created in the same scope.
   ,-[files/index.jsx:2:40]
 1 | export function Example() {
 2 |   return <Item node={<Child />} items={[]} callback={() => {}} options={{}} />;
   :                                        ^^
 3 | }
   `----
  help: simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).

  x react-perf(jsx-no-new-function-as-prop): JSX attribute values should not contain functions created in the same scope.
   ,-[files/index.jsx:2:54]
 1 | export function Example() {
 2 |   return <Item node={<Child />} items={[]} callback={() => {}} options={{}} />;
   :                                                      ^^^^^^^^
 3 | }
   `----
  help: simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).

  x react-perf(jsx-no-new-object-as-prop): JSX attribute values should not contain objects created in the same scope.
   ,-[files/index.jsx:2:73]
 1 | export function Example() {
 2 |   return <Item node={<Child />} items={[]} callback={() => {}} options={{}} />;
   :                                                                         ^^
 3 | }
   `----
  help: simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array).

Found 0 warnings and 4 errors.
Finished in Xms on 1 file with 4 rules using X threads.
```

# stderr
```
```
