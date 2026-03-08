// Examples of incorrect code for no-unnecessary-type-conversion rule

const a = String('asdf');
const b = 'asdf'.toString();
const c = '' + 'asdf';
const d = Number(123);
const e = !!true;
