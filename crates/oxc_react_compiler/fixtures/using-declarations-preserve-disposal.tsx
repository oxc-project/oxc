class Disposer {
  [Symbol.dispose]() {}
}

class AsyncDisposer {
  async [Symbol.asyncDispose]() {}
}

export function UnusedResource() {
  using resource = new Disposer();
  return <p>hello world</p>;
}

export async function AwaitUsingResource(props) {
  await using resource = await props.acquire();
  return <p>{props.text}</p>;
}

export function UsedResource(props) {
  using resource = props.resource;
  return <button onClick={() => resource.use()}>{props.text}</button>;
}

export function BlockScopedResource(props) {
  if (props.enabled) {
    using resource = props.acquire();
    if (props.early) {
      return <p>{resource.value}</p>;
    }
  }
  return <p>{props.text}</p>;
}

export function ForUsingResource(props) {
  for (using resource of props.resources) {
    resource.use();
  }
  return <p>{props.text}</p>;
}
