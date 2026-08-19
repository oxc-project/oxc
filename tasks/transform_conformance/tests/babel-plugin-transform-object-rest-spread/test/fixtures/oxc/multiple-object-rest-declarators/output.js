const _excluded = ["ariaAttributes", "style"], _excluded2 = ["ariaAttributes", "style"];
function merge(e, t) {
	const { ariaAttributes: s, style: r } = e, n = babelHelpers.objectWithoutProperties(e, _excluded), { ariaAttributes: i, style: l } = t, f = babelHelpers.objectWithoutProperties(t, _excluded2);
	return [s, r, n, i, l, f];
}
