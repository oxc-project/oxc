class C {
  #method<T>(value: T) { return [this, value]; }
  #field = function(value: number) { return [this, value]; };
  #missing;
  get #getter() { return this.#field; }
  static #staticMethod(value: number) { return [this, value]; }
  static #staticField = function(value: number) { return [this, value]; };
  static get #staticGetter() { return this.#staticField; }
  static get #staticValue() { return () => 42; }

  run() {
    let calls = 0;
    const receiver = () => { calls++; return this; };
    expect((receiver().#method satisfies Function)(1)).toEqual([this, 1]);
    expect((<any>receiver().#method)(2)).toEqual([this, 2]);
    expect(((<any>(receiver().#method as any))! satisfies Function)(3)).toEqual([this, 3]);
    expect((receiver().#method<number>)(4)).toEqual([this, 4]);
    expect((receiver().#field as any)(5)).toEqual([this, 5]);
    expect(receiver().#getter!(6)).toEqual([this, 6]);
    expect((receiver().#method satisfies Function)?.(7)).toEqual([this, 7]);
    expect((<any>receiver().#method)?.(8)).toEqual([this, 8]);
    expect(((receiver().#method as any)!)?.(9)).toEqual([this, 9]);
    expect((receiver().#method<number>)?.(10)).toEqual([this, 10]);
    expect((receiver().#field as any)?.(11)).toEqual([this, 11]);
    expect(receiver().#getter!?.(12)).toEqual([this, 12]);
    let args = 0;
    expect((receiver().#missing as any)?.(++args)).toBeUndefined();
    expect(args).toBe(0);
    expect(calls).toBe(13);
    const detached = this.#method as any;
    expect(detached(13)).toEqual([undefined, 13]);
  }

  static run() {
    let calls = 0;
    const receiver = () => { calls++; return this; };
    expect((receiver().#staticMethod as any)(1)).toEqual([this, 1]);
    expect(receiver().#staticField!(2)).toEqual([this, 2]);
    expect((receiver().#staticGetter satisfies Function)(3)).toEqual([this, 3]);
    expect((<any>receiver().#staticMethod)?.(4)).toEqual([this, 4]);
    expect((receiver().#staticField as any)?.(5)).toEqual([this, 5]);
    expect(receiver().#staticGetter!?.(6)).toEqual([this, 6]);
    expect((C.#staticMethod as any)(7)).toEqual([C, 7]);
    expect(C.#staticField!(8)).toEqual([C, 8]);
    expect((C.#staticGetter satisfies Function)(9)).toEqual([C, 9]);
    expect((C.#staticValue as any)()).toBe(42);
    expect(calls).toBe(6);
  }
}

new C().run();
C.run();
