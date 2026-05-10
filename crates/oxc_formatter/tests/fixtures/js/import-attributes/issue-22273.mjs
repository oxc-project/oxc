// Issue #22273 - Should retain space after comma in import attributes
import myEmbeddedDb from './my.db' with { type: 'sqlite', embed: 'true' }

// Should be spread over multiple lines
import myEmbeddedDb from './my.db' with { type: 'sqlite', embed: 'true', veryLongAttribute: "true" }

// Shouldn't have a comma for a single attribute
import myEmbeddedDb from './my.db' with { type: 'sqlite' }


