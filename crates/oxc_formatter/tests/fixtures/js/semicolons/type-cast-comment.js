;/** @type {string[]} */ (['foo', 'bar']).forEach(doStuff)
;/** @type {(token: Token)=>void} */ (onToken)(token)
;/** @type {string} */ (foo).length
;/* 2 */ /** @type {{bar: string[]}} */ ({}).bar.forEach(doStuff)
;/* not a type cast comment */ ([])(token)
;/* don't need leading semicolon */ (foo)(token)
