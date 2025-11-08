// Basic pattern modifiers
const r1 = new RegExp("^(?i:[a-z])$", "");

// Pattern modifier with other groups
const r2 = new RegExp("(?i:hello)(?-i:WORLD)", "");

// Multiple modifiers
const r3 = new RegExp("(?im:test)", "");

// Pattern modifier in nested groups
const r4 = new RegExp("a(?i:b(?-i:c))d", "");

// Pattern modifier with flags
const r5 = new RegExp("(?i:test)", "g");

// Case insensitive modifier
const r6 = new RegExp("(?i:abc)", "");

// Multiple flags modifier
const r7 = new RegExp("(?-i:test)", "m");
