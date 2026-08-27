// This file copies from https://github.com/prettier/prettier/blob/main/tests/format/js/ignore/ignore.js,
// and replaces all `prettier-ignore` with `oxfmt-ignore` to verify that oxfmt correctly ignores those comments.
function a() {
  // oxfmt-ignore
  var fnString =
    '"' + this.USE + ' ' + this.STRICT + '";\n' +
    this.filterPrefix() +
    'var fn=' + this.generateFunction('fn', 's,l,a,i') +
    extra +
    this.watchFns() +
    'return fn;';

  // oxfmt-ignore
  const identity = Matrix.create(
    1, 0, 0,
    0, 1, 0,
    0, 0, 0
  );

  // Let's make sure that this comment doesn't interfere

  // oxfmt-ignore
  const commentsWithPrettierIgnore =   {
    "ewww":
            "gross-formatting",
  };

  function giveMeSome() {
    a(  a  ); // oxfmt-ignore
    // shouldn't I return something?  :shrug:
  }

  // oxfmt-ignore
  console.error(
    'In order to use ' + prompt + ', you need to configure a ' +
    'few environment variables to be able to commit to the ' +
    'repository. Follow those steps to get you setup:\n' +
    '\n' +
    'Go to https://github.com/settings/tokens/new\n' +
    ' - Fill "Token description" with "' + prompt + ' for ' +
      repoSlug + '"\n' +
    ' - Check "public_repo"\n' +
    ' - Press "Generate Token"\n' +
    '\n' +
    'In a different tab, go to https://travis-ci.org/' +
      repoSlug + '/settings\n' +
    ' - Make sure "Build only if .travis.yml is present" is ON\n' +
    ' - Fill "Name" with "GITHUB_USER" and "Value" with the name of the ' +
      'account you generated the token with. Press "Add"\n' +
    '\n' +
    'Once this is done, commit anything to the repository to restart ' +
      'Travis and it should work :)'
  );
}

const response = {
  // oxfmt-ignore
  '_text': 'Turn on the lights',
  intent: 'lights',
};

import   {foo as bar}   from   "pkg"; // oxfmt-ignore
export const item={  a:1,b:2}; // oxfmt-ignore

const config={  retries:10,timeout:5000}; // oxfmt-ignore
let data=[ 1,2,3 ]; // oxfmt-ignore

const items = [
  {a:aa(),b:bb(),c:cc(),d:dd(),e:ee(),f:ff(),g:gg()}, // oxfmt-ignore
];

foo(  {a:1,b:2},  // oxfmt-ignore
  {c:3,d:4}
);

function demo() {
  return   {a:1,b:2}; // oxfmt-ignore
}

function fail() {
  throw   new Error(  "boom"  ); // oxfmt-ignore
}

if (ok) logger(  payload  ); // oxfmt-ignore
while (keepGoing) tick(  value  ); // oxfmt-ignore

label  :  for ( ; ; ) { break   label; } // oxfmt-ignore
for (let i=0;i<3;i++) step(  i  ); // oxfmt-ignore
for (const k in obj) use(  k  ); // oxfmt-ignore
for (const v of arr) use(  v  ); // oxfmt-ignore
while (ok) run(  x  ); // oxfmt-ignore
do run(  x  ); while (ok); // oxfmt-ignore
switch (kind) {case 1: act(  ); break;} // oxfmt-ignore
try { a(  ); } catch (e) { b(  ); } // oxfmt-ignore
with (ctx) run(  x  ); // oxfmt-ignore

function f() {
  return   {a:1}; // oxfmt-ignore
}

function g() {
  throw   new Error(  "boom"  ); // oxfmt-ignore
}
