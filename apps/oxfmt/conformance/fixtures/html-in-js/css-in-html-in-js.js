// CSS in a <style> tag is formatted (sub-formatter runs regardless of `parentParser`);
// CSS in a style="" attribute is NOT (`parentParser` blocks attribute-level sub-formatters).
const both = /* HTML */ `<div style="color: red; font-size: 16px;"><style>.bar { background: blue; margin: 0; padding: 10px 20px; }</style><p>hello</p></div>`;

// With expressions
function d(color) {
	return /* HTML */ `<style>.dynamic { color: ${color}; }</style><p style="color: ${color};">${color}</p>`;
}
