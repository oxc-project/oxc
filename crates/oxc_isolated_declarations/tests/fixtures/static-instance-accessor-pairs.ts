// Instance and static accessors with the same key are distinct properties.
// Their types must not be used to infer each other.
export class InstanceGetterStaticSetter {
  get value() {
    return;
  }

  static set value(value: string) {}
}

export class StaticGetterInstanceSetter {
  static get value() {
    return;
  }

  set value(value: number) {}
}

export class InstanceGetterStaticUntypedSetter {
  get value(): string {
    return "";
  }
  static set value(value) {}
}

export class StaticGetterInstanceUntypedSetter {
  static get value(): number {
    return 0;
  }
  set value(value) {}
}
