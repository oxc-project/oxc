// CSS in <style> tags is formatted (sub-formatter runs regardless of `parentParser`).
const styleTag = /* HTML */ `<style>.foo { color: red; font-size: 16px; display: flex; align-items: center; justify-content: center; }</style>`;

// CSS in style="" attributes is NOT formatted (`parentParser` blocks attribute-level sub-formatters).
const styleAttr = /* HTML */ `<div style="color: red; font-size: 16px; display: flex; align-items: center; justify-content: center;">hello</div>`;

// Both combined: <style> tag is formatted, style="" attribute is left as-is.
const both = /* HTML */ `<div style="color: red; font-size: 16px;"><style>.bar { background: blue; margin: 0; padding: 10px 20px; }</style><p>hello</p></div>`;

// With expressions
function d(color) {
	return /* HTML */ `<style>.dynamic { color: ${color}; }</style><p style="color: ${color};">${color}</p>`;
}
