// this test is passing with Babel 6
// but would fail for Babel 7 _without_ custom hook node being cloned for signature
import {useFancyState} from './hooks';

export default function App() {
  const bar = useFancyState();
  return <h1>{bar}</h1>;
}
