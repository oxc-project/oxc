// prettier-ignore on class property: the value prints verbatim, the `;` is the formatter's
// (never doubled, added per `semi` when the source has none)
export class Counter {
	// prettier-ignore
	'count' = $state(0);
	constructor() {
		this['count'] = $state(0);
	}
}

export class Counter2 {
	// prettier-ignore
	'count' = $state(0)
	constructor() {
		this['count'] = $state(0);
	}
}

export class Counter3 {
	'count' = $state(0)
	constructor() {
		this['count'] = $state(0);
	}
}

export class Counter4 {
	'count' = $state(0);;
	constructor() {
		this['count'] = $state(0);
	}
}
