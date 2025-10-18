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

class Setter {
	@dec
	set address(value: number) {}

	@dec
	regularMethod(): string {
		return "test";
	}
}
