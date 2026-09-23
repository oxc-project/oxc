async function f(...[x = async () => await 0]) {}
function* g(...[x = function* () { yield 0; }]) {}
