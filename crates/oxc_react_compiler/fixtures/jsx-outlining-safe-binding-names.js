// @enableJsxOutlining
function Component({arr}) {
  return (
    <ul>
      {arr.map(item => (
        <li p="existing" key-={item.keyValue} eval-={item.evalValue} arguments-={item.argumentsValue} class={item.classValue} key={item.id}>
          <span p1="existing1" data-id={item.id}>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{arr: [{id: 1, label: 'a', keyValue: 'key', evalValue: 'eval', argumentsValue: 'arguments', classValue: 'row'}]}],
};
