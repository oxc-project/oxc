// js-in-html-in-js: JS inside <script> makes Prettier emit `conditionalGroup`,
// which maps to `BestFitting` in the Doc->IR conversion (from_prettier_doc.rs).

// Member chain (member-chain.js) with template expressions inside the <script>,
// so placeholders sit inside BestFitting variants (placeholders_are_sequential + clone-shared substitution).
const withChain = html`
	<script>
		window.app.store.getState().users.filter((user) => user.active).map((user) => user.name).join(", ");
		document.querySelector("#root").addEventListener("click", ${handler});
	</script>
	<p>${label}</p>
`;

// Last-argument expansion (call-arguments.js): callback as last argument.
const withCallback = html`
	<script>
		setup(${config}, () => { console.log("ready"); start(); });
	</script>
`;
