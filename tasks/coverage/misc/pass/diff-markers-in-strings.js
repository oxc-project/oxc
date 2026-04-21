// Test that conflict markers in strings and comments don't trigger false positives
// These should all parse successfully as they are valid JavaScript code

// Conflict markers in strings (should NOT trigger)
const validString = "<<<<<<< HEAD";
const anotherString = "=======";
const endMarker = ">>>>>>> branch";
const diff3Marker = "|||||||";

// Conflict markers in template literals (should NOT trigger)
const validTemplate = `
<<<<<<< not a marker
this is fine
=======
still fine
>>>>>>> also fine
`;

// Conflict markers in comments (should NOT trigger)
// <<<<<<< HEAD - this is a comment about conflicts
// ======= separator
// >>>>>>> branch

/* 
Multi-line comment with markers:
<<<<<<< HEAD
=======
>>>>>>> branch
These are all just text in a comment
*/

// Edge case: markers that look similar but aren't exactly 7 characters
// These should NOT trigger because they're not conflict markers
const tooShort = "<<<<<<";
const tooLong = "<<<<<<<<";

// All of the above should parse successfully
export { validString, validTemplate };
