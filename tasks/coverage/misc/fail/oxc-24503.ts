class Basic {
    accessor noType!;
    accessor initialized! = 1;
    accessor typed!: number;
    static accessor staticTyped!: number;
    static accessor staticNoType!;
    static accessor staticInitialized! = 1;
}

abstract class Abstract {
    abstract accessor value!: number;
}

declare class Ambient {
    accessor value!: number;
}

class DeclareMember {
    declare accessor typed!: number;
    declare accessor untyped!;
    declare accessor initialized! = 1;
}
