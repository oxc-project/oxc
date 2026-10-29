// @validateNoImpureFunctionsInRender @outputMode:"lint" @compilationMode:"infer" @panicThreshold:"none"
// Async function components are Server Components: they render once per request and never
// re-render, so impure calls are not reported in them or in their nested callbacks.
export default async function Posts() {
  const posts = await getPosts();
  return posts.map((p) => <p key={p.id}>{Date.now() - p.createdAt < 86_400_000 ? "new" : ""}</p>);
}

async function AsyncDeclaration() {
  const date = Date.now();
  const now = new Date();
  const rand = Math.random();
  return <Foo date={date} now={now} rand={rand} />;
}

const AsyncArrow = async () => {
  return <p>{Date.now()}</p>;
};

const AsyncFunctionExpression = async function () {
  return <p>{new Date().getTime()}</p>;
};

async function AsyncMapCallback({ posts }) {
  return (
    <ul>
      {posts.map((p) => (
        <li key={p.id}>{Date.now() - p.createdAt}</li>
      ))}
    </ul>
  );
}

async function AsyncHelperCallback({ posts }) {
  const label = (post) => (Date.now() - post.createdAt < 86400000 ? "new" : "");
  return (
    <ul>
      {posts.map((p) => (
        <li key={p.id}>{label(p)}</li>
      ))}
    </ul>
  );
}

async function AsyncNestedComponent() {
  function Badge() {
    return <span>{Date.now()}</span>;
  }
  return <Badge />;
}

// Non-async components are still reported, including after an async component.
function ClientAfterServer() {
  return <p>{Date.now()}</p>;
}

const ClientArrow = () => {
  return <p>{Math.random()}</p>;
};

// Nested callbacks of a non-async component still run on every render.
function ClientMapCallback({ posts }) {
  return (
    <ul>
      {posts.map((p) => (
        <li key={p.id}>{Date.now()}</li>
      ))}
    </ul>
  );
}

// Hooks stay checked, including async hooks.
function useNow() {
  return <span>{Date.now()}</span>;
}

async function useAsyncNow() {
  return <span>{new Date().getTime()}</span>;
}
