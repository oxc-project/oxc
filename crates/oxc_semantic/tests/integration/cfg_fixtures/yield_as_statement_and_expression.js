var MyClass = class {
    static async *#Generate() {
        callCount += 1;
        yield [...yield];
    }
}
