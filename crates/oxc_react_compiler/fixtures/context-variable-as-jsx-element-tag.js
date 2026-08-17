// @enablePreserveExistingMemoizationGuarantees:false
import {useMemo} from 'react';
import {Stringify} from 'shared-runtime';

function Component(props) {
  let _Component = Stringify;

  _Component = useMemo(() => {
    return _Component;
  }, [_Component]);

  return <_Component {...props} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{name: 'Sathya'}],
};
