// Basic pattern modifiers
const r1 = /^(?i:[a-z])$/;

// Pattern modifier with other groups
const r2 = /(?i:hello)(?-i:WORLD)/;

// Multiple modifiers
const r3 = /(?im:test)/;

// Pattern modifier in nested groups
const r4 = /a(?i:b(?-i:c))d/;

// Pattern modifier with flags
const r5 = /(?i:test)/g;

// Case insensitive modifier
const r6 = /(?i:abc)/;

// Multiple flags modifier
const r7 = /(?-i:test)/m;
