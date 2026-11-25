// Prettier keeps a trailing double space on JSDoc lines (prettier/prettier#18594).
/**
 * Two spaces stay.  
 * It's on another line
 */
function two() {}

/**
 * Three spaces collapse to two.   
 * next
 */
function three() {}

/**
 * One space goes. 
 * next
 */
function one() {}

/**
 *  
 * A bare star line drops its spaces.
 */
function bare() {}

/**  
 * Spaces on the opening line go, spaces here stay.  
 */
function opening() {}

function nested() {
        /**
         * Misaligned and indented, spaces stay.  
        */
  return 1;
}

// DIVERGENCES.md#triple-star-jsdoc-hard-break
/***
 * Also JSDoc, spaces stay.  
 */
function tripleStar() {}

/*
 * Plain block comment, spaces go.  
 */
function plain() {}
