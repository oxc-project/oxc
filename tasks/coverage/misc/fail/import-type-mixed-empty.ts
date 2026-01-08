// https://github.com/oxc-project/oxc/issues/17609
// TS1363: A type-only import can specify a default import or named bindings, but not both.
import type foo, {} from 'bar';
