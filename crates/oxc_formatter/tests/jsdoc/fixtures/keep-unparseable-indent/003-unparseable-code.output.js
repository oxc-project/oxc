/**
 * @example
 *   Prism.languages['css-with-colors'] = Prism.languages.extend('css', {
 *       // Prism.languages.css already has a 'comment' token, so this token will overwrite CSS' 'comment' token
 *       // at its original position
 *       'comment': { ... },
 *       // CSS doesn't have a 'color' token, so this token will be appended
 *       'color': /\b(?:red|green|blue)\b/
 *   });
 */
