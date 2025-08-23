// Examples of incorrect code for only-throw-error rule

throw 'error'; // throwing string

throw 42; // throwing number

throw true; // throwing boolean

throw { message: 'error' }; // throwing plain object

throw null; // throwing null