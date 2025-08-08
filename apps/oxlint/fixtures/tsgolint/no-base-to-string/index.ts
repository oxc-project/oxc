// Examples of incorrect code for no-base-to-string rule

// These will evaluate to '[object Object]'
({}).toString();
({foo: 'bar'}).toString();
({foo: 'bar'}).toLocaleString();

// This will evaluate to 'Symbol()'
Symbol('foo').toString();