// When in doubt, register variables that were used in JSX.
// Foo, Header, and B get registered.
// A doesn't get registered because it's not declared locally.
// Alias doesn't get registered because its definition is just an identifier.

import A from './A';
import Store from './Store';

Store.subscribe();

const Header = styled.div`color: red`
const StyledFactory1 = styled('div')`color: hotpink`
const StyledFactory2 = styled('div')({ color: 'hotpink' })
const StyledFactory3 = styled(A)({ color: 'hotpink' })
const FunnyFactory = funny.factory``;

let Alias1 = A;
let Alias2 = A.Foo;
const Dict = {};

function Foo() {
  return (
    <div><A /><B /><StyledFactory1 /><StyledFactory2 /><StyledFactory3 /><Alias1 /><Alias2 /><Header /><Dict.X /></div>
  );
}

const B = hoc(A);
// This is currently registered as a false positive:
const NotAComponent = wow(A);
// We could avoid it but it also doesn't hurt.
