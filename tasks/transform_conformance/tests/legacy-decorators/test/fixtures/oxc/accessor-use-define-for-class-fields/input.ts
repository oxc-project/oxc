// accessor backing field should be hoisted when other public fields are hoisted
class Hello {
    private input = { foo };
    accessor util = this.input.foo();
}

// accessor backing field should NOT be hoisted when no public fields exist
class AccessorOnly {
    accessor y = 2;
}
