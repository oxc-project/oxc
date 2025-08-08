// Examples of incorrect code for no-unnecessary-boolean-literal-compare rule

declare const someCondition: boolean;

if (someCondition === true) {
  // ...
}

if (someCondition === false) {
  // ...
}