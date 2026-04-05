let done = false;

function work() {}

export default function App() {
  while (!done) {
    work();
  }

  done = true;
  return <div />;
}
