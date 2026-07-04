// The variable is declared after the template referencing it. Rules which read
// `parent` of a variable's declaration (e.g. `ember/template-no-let-reference` reads
// `identifier.parent.parent.kind`) require all parents to be set before any rule
// runs, like ESLint does - not while the AST is being walked.
class LateDeclaration {
  <template>
    <p>{{lateMessage}}</p>
  </template>
}

let lateMessage = "declared after the template";

export { LateDeclaration, lateMessage };
