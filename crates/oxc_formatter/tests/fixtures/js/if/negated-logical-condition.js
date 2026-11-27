// Hugs the head's parens
if (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (a) {} else if (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
while (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
do {} while (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference"));
with (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (!!(node.type === "ImportExpression" ?? node.type === "TSImportType" ?? node.type === "TSExternalModuleReference")) {}

// Breaks in the head's group
if (!!!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (typeof (node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (!isNodeMatches(node.type === "ImportExpression", node.type === "TSImportType", node.type === "TSExternalModuleReference")) {}
if (!(node.type === "ImportExpression" || node.type === "TSImportType") && node.type === "TSExternalModuleReference") {}
if (!/** @type {boolean} */ (node.type === "ImportExpression" || isImportType)) {}
if (!!/** @type {boolean} */ (node.type === "ImportExpression" || isImportType)) {}
if (!/** @type {boolean} */ (!(node.type === "ImportExpression" || isTypeOf))) {}
// `switch` and `for` never hug (as Prettier)
switch (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
for (; !(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference"); ) {}

// A comment of the condition's own, outside its parens, breaks in the head's group
if (/* c */ !(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (
  // c
  !(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference") /* c */) {}
if (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference") // c
) {}
do {} while (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference") /* c */);

// Comments elsewhere keep the hug
if (!(/* c */ node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (!(node.type === "ImportExpression" || // c
  node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (! // c
!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) {}
if (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference")) /* c */ {}
// The comment keeps its side of `while` (DIVERGENCES.md#head-body-comment-relocation)
do {} // c
while (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference"));
do foo() /* c */; while (!(node.type === "ImportExpression" || node.type === "TSImportType" || node.type === "TSExternalModuleReference"));
