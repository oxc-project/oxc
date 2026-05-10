// Should retain space after comma
import myEmbeddedDb from './my.db' with { type: 'sqlite', embed: 'true' }

// Should be spread over multiple lines
import myEmbeddedDb from './my.db' with { type: 'sqlite', embed: 'true', veryLongAttribute: "true" }

