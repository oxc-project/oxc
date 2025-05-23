const A = {
	get state(): number {
		return 0;
	},
};

const B = {
	set state(v: string) {
		// do something
	},
};

const C = {
	get state(): number {
		return 0;
	},
	set state(v: number) {
		// do something
	},
};

const D = {
	get state(): number {
		return 0;
	},
	set state(v: string) {
		// do something
	},
};

const E = {
	get state() {
		return A;
	},
	set state(v) {
		// do something
	},
};
