type EnthusiasticGreeting<T extends string> =
  `${Uppercase<T>} - ${Lowercase<T>} - ${Capitalize<T>} - ${Uncapitalize<T>}`;
type HELLO = EnthusiasticGreeting<'heLLo'>;
