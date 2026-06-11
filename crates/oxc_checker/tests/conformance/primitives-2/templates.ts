export {};

// No-substitution template literal keeps literal type under const
const tmpl_const = `alpha`;
const ok_tmpl: "alpha" = tmpl_const;

// let widens a template literal to string
let tmpl_let = `beta`;
const bad_tmpl: "beta" = tmpl_let;

// Substituting a literal-typed const keeps a literal template type
const name_part = "world";
const ok_literal_tmpl: "hello world" = `hello ${name_part}`;

// Substituting a plain string makes the template plain string
declare const dynamic_part: string;
const greeting = `hello ${dynamic_part}`;
const ok_greeting: string = greeting;
const bad_greeting: "hello world" = greeting;

// Template literal type with no substitutions acts as a string literal
type ExactGreeting = `hello world`;
const ok_exact: ExactGreeting = "hello world";
const bad_exact: ExactGreeting = "hello there";
