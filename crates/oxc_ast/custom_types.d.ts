export interface FormalParameterRest extends Span {
  type: 'RestElement';
  argument: BindingPatternKind;
  typeAnnotation: TSTypeAnnotation | null;
  optional: boolean;
}

export type RegExpFlags = {
  /** Global flag */
  G: 1;
  /** Ignore case flag */
  I: 2;
  /** Multiline flag */
  M: 4;
  /** DotAll flag */
  S: 8;
  /** Unicode flag */
  U: 16;
  /** Sticky flag */
  Y: 32;
  /** Indices flag */
  D: 64;
  /** Unicode sets flag */
  V: 128;
};

export type JSXElementName =
  | JSXIdentifier
  | JSXNamespacedName
  | JSXMemberExpression;

export type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;
