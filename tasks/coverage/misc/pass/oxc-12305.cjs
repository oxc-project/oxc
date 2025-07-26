// Sloppy mode

const a = let => {};
let b = let => {};
var c = let => {};

const d = (let) => {};
let e = (let) => {};
var f = (let) => {};

const g = ({let}) => {};
const h = ([let]) => {};
const i = ({x: let}) => {};
const j = ({x: {y: [let]}}) => {};
