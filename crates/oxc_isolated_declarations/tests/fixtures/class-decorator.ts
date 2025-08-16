declare const decorator: any;

export class Test1 {
  /** This method will trigger the feature highlight dialog load/show based on dialogId and analyticsId */
  @decorator
  property: (() => any) | undefined;
}

export class Test2 {
  /** This method will trigger the feature highlight dialog load/show based on dialogId and analyticsI */
  @decorator
  property: ((arg: any) => any) | undefined;
}

export class Test3 {
  @decorator
  property?: {
    property?: (arg: any) => any;
  };
}
