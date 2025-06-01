function Bar(): ClassDecorator {
  return (_target) => {
    console.log(Bar.name)
  }
}

@Bar()
class Foo {
  static foo = `${Foo.name}`;
}