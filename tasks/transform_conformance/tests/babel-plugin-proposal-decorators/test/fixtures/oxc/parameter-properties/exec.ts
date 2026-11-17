const log: string[] = [];
function logged(fn: Function, ctx: ClassMethodDecoratorContext) {
  return function (this: unknown, ...args: unknown[]) {
    log.push(String(ctx.name));
    return fn.apply(this, args);
  };
}

class Base {
  base = "base";
}

class Service extends Base {
  @logged greet(): string {
    return `${this.id}:${this.owner}:${this.count}`;
  }
  count = 1;
  constructor(public readonly id: number, private owner: string = "me") {
    super();
  }
}

const s = new Service(7);
expect(s.greet()).toBe("7:me:1");
expect(s.id).toBe(7);
expect(log).toEqual(["greet"]);
