// Test case for declare fields issue #13733
class B {
  public value: number = 3;
  
  constructor(value?: number) {
    if (value !== undefined) {
      this.value = value;
    }
  }
}

class C extends B {
  declare public value: number;

  log() {
    return "C " + this.value
  }
}

// This should log "C 6", not "C undefined"
expect(new C(6).log()).toBe("C 6");
