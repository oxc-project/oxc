// DIVERGENCES.md#binary-keyword-class-field-boundary
class QuotedKeys {
  x = 1;
  "in";
  y = 1;
  "instanceof"() {}
}
class ConsistentKeys {
  "not-an-identifier";
  x = 1;
  "in";
  y = 1;
  "instanceof"() {}
}
class ConsistentIdentifiers {
  "not-an-identifier";
  x = 1;
  in;
  y = 1;
  instanceof() {}
}
