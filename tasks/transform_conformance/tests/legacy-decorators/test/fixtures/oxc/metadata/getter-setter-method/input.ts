declare function dec(
	target: any,
	propertyKey: string,
	descriptor: PropertyDescriptor,
): void;

class Getter {
	@dec
	get address(): string {
		return "test";
	}

	@dec
	regularMethod(): string {
		return "test";
	}
}

class UntypedGetter {
	@dec
	get myProp() {
		return "hello";
	}
}

class UntypedSetter {
	@dec
	set myProp(value) {}
}

class Setter {
	@dec
	set address(value: number) {}

	@dec
	regularMethod(): string {
		return "test";
	}
}
