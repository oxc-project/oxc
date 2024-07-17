declare const React;

class Foo {
  foo: any;
  Div = {
    Element: () => <div />,
  };
  method() {
    <this.foo />;
    (<Div.Element />)(<this />);
  }
}
