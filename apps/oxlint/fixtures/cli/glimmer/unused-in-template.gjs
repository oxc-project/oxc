// `greeting` is only used inside the template, which the partial loader blanks
// out, so `no-unused-vars` must not run on loader-derived Glimmer sources.
// The `debugger` statement proves other native rules still run.
const greeting = "hello";
debugger;

export default <template>{{greeting}}</template>;
