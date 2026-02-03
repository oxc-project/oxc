function definePlugin(plugin) {
	return plugin;
}
function defineRule(rule) {
	return rule;
}
const EMPTY_VISITOR = {};
function eslintCompatPlugin(plugin) {
	if (typeof plugin != "object" || !plugin) throw Error("Plugin must be an object");
	let { rules } = plugin;
	if (typeof rules != "object" || !rules) throw Error("Plugin must have an object as `rules` property");
	for (let ruleName in rules) Object.hasOwn(rules, ruleName) && convertRule(rules[ruleName]);
	return plugin;
}
function convertRule(rule) {
	if (typeof rule != "object" || !rule) throw Error("Rule must be an object");
	if ("create" in rule) return;
	let context = null, visitor, beforeHook;
	rule.create = (eslintContext) => (context === null && ({context, visitor, beforeHook} = createContextAndVisitor(rule)), Object.defineProperties(context, {
		id: { value: eslintContext.id },
		options: { value: eslintContext.options },
		report: { value: eslintContext.report }
	}), Object.setPrototypeOf(context, Object.getPrototypeOf(eslintContext)), beforeHook !== null && beforeHook() === !1 ? EMPTY_VISITOR : visitor);
}
const FILE_CONTEXT = Object.freeze({
	get filename() {
		throw Error("Cannot access `context.filename` in `createOnce`");
	},
	getFilename() {
		throw Error("Cannot call `context.getFilename` in `createOnce`");
	},
	get physicalFilename() {
		throw Error("Cannot access `context.physicalFilename` in `createOnce`");
	},
	getPhysicalFilename() {
		throw Error("Cannot call `context.getPhysicalFilename` in `createOnce`");
	},
	get cwd() {
		throw Error("Cannot access `context.cwd` in `createOnce`");
	},
	getCwd() {
		throw Error("Cannot call `context.getCwd` in `createOnce`");
	},
	get sourceCode() {
		throw Error("Cannot access `context.sourceCode` in `createOnce`");
	},
	getSourceCode() {
		throw Error("Cannot call `context.getSourceCode` in `createOnce`");
	},
	get languageOptions() {
		throw Error("Cannot access `context.languageOptions` in `createOnce`");
	},
	get settings() {
		throw Error("Cannot access `context.settings` in `createOnce`");
	},
	extend(extension) {
		return Object.freeze(Object.assign(Object.create(this), extension));
	},
	get parserOptions() {
		throw Error("Cannot access `context.parserOptions` in `createOnce`");
	},
	get parserPath() {
		throw Error("Cannot access `context.parserPath` in `createOnce`");
	}
});
function createContextAndVisitor(rule) {
	let { createOnce } = rule;
	if (createOnce == null) throw Error("Rules must define either a `create` or `createOnce` method");
	if (typeof createOnce != "function") throw Error("Rule `createOnce` property must be a function");
	let context = Object.create(FILE_CONTEXT, {
		id: {
			value: "",
			enumerable: !0,
			configurable: !0
		},
		options: {
			value: null,
			enumerable: !0,
			configurable: !0
		},
		report: {
			value: null,
			enumerable: !0,
			configurable: !0
		}
	}), { before: beforeHook, after: afterHook, ...visitor } = createOnce.call(rule, context);
	if (beforeHook === void 0) beforeHook = null;
	else if (beforeHook !== null && typeof beforeHook != "function") throw Error("`before` property of visitor must be a function if defined");
	if (afterHook != null) {
		if (typeof afterHook != "function") throw Error("`after` property of visitor must be a function if defined");
		let programExit = visitor["Program:exit"];
		visitor["Program:exit"] = programExit == null ? (_node) => afterHook() : (node) => {
			programExit(node), afterHook();
		};
	}
	return {
		context,
		visitor,
		beforeHook
	};
}
export { definePlugin, defineRule, eslintCompatPlugin };
