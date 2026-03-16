declare function dec(): ClassDecorator;

@dec()
class MyService {
  constructor(a: string);
  constructor(a: string, b: string);
  constructor(a?: string, b?: string) {}
}
