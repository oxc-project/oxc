// @validateRefAccessDuringRender
function Component({cond}) {
  const ref = useRef(null);
  const x = cond ? ref.current : ref;
  return <Foo value={x} />;
}
