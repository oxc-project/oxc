import fbt from 'fbt';
import {identity} from 'shared-runtime';

/**
 * The fbt transform looks for callsites with an `fbt`-named callee, so the
 * compiler must preserve local `fbt` binding names across nested scopes.
 */

function Foo(props) {
  const getText1 = fbt =>
    fbt(
      `Hello, ${fbt.param('(key) name', identity(props.name))}!`,
      '(description) Greeting'
    );

  const getText2 = fbt =>
    fbt(
      `Goodbye, ${fbt.param('(key) name', identity(props.name))}!`,
      '(description) Greeting2'
    );

  return (
    <div>
      {getText1(fbt)}
      {getText2(fbt)}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Foo,
  params: [{name: 'Sathya'}],
};
