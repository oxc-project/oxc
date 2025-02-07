/* In esbuild scripts/terser-tests.js
 * Under `Run esbuild as a minifier`
 *
 * ```
     if (test.expect_stdout)  {
      const expected = test.expect_stdout.split('\n').filter((s) => s.trim().length > 0).map((s) => JSON.stringify(s)).join(', ');
      const prepend_code = test.prepend_code ? `const prepend = ${JSON.stringify(test.prepend_code)};` : '';
      const run_code = test.prepend_code ? `run(code, expected, prepend);` : `run(code, expected);`;
      console.log(`
        test('${test.name}', () => {
          ${prepend_code}
          const code = ${JSON.stringify(ast_as_string)};
          const expected = [${expected}];
          ${run_code}
        });
      `)
    }
 * ```
 **/

import { expect, test, vi } from 'vitest';
import { minify } from '../index';
import { run_code } from './sandbox';

function run(input: string, expected: string[], prepend_code?: string) {
  const consoleMock = vi.spyOn(console, 'log').mockImplementation(() => undefined);
  try {
    const minified = minify('test.mjs', input).code;
    expect(minified).not.toBeFalsy();
    // Use `consoleMock` instead of the returned output.
    const _ = run_code(minified, prepend_code);
    const calls = consoleMock.mock.calls.map((args) => args.map(convert).join(' '));
    expect(calls).toStrictEqual(expected);
  } finally {
    consoleMock.mockReset();
  }
}

function convert(arg: any) {
  if (arg == null) {
    return 'undefined';
  }
  if (typeof arg == 'string') {
    return arg;
  }
  if (arg && arg.length >= 0) {
    return JSON.stringify(arg);
  }
  return arg;
}

test('replace_index', () => {
  const code =
    'var arguments=[];console.log(arguments[0]);(function(){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(a,b){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(arguments){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(){var arguments;console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);';
  const expected = ['undefined', '42 42 undefined', '42 42 undefined', 'a a undefined', '42 42 undefined'];
  run(code, expected);
});

test('replace_index_strict', () => {
  const code =
    '"use strict";(function(){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(a,b){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);';
  const expected = ['42 42 undefined', '42 42 undefined'];
  run(code, expected);
});

test('replace_index_keep_fargs', () => {
  const code =
    'var arguments=[];console.log(arguments[0]);(function(){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(a,b){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(arguments){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(){var arguments;console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);';
  const expected = ['undefined', '42 42 undefined', '42 42 undefined', 'a a undefined', '42 42 undefined'];
  run(code, expected);
});

test('replace_index_keep_fargs_strict', () => {
  const code =
    '"use strict";(function(){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);(function(a,b){console.log(arguments[1],arguments["1"],arguments["foo"])})("bar",42);';
  const expected = ['42 42 undefined', '42 42 undefined'];
  run(code, expected);
});

test('modified', () => {
  const code =
    '(function(a,b){var c=arguments[0];var d=arguments[1];var a="foo";b++;arguments[0]="moo";arguments[1]*=2;console.log(a,b,c,d,arguments[0],arguments[1])})("bar",42);';
  const expected = ['moo 86 bar 42 moo 86'];
  run(code, expected);
});

test('modified_strict', () => {
  const code =
    '"use strict";(function(a,b){var c=arguments[0];var d=arguments[1];var a="foo";b++;arguments[0]="moo";arguments[1]*=2;console.log(a,b,c,d,arguments[0],arguments[1])})("bar",42);';
  const expected = ['foo 43 bar 42 moo 84'];
  run(code, expected);
});

test('arguments_in_arrow_func_1', () => {
  const code =
    '(function(a,b){console.log(arguments[0],a,arguments[1],arguments[3],b,arguments[2])})("bar",42,false);(function(a,b){(()=>{console.log(arguments[0],a,arguments[1],arguments[3],b,arguments[2])})(10,20,30,40)})("bar",42,false);';
  const expected = ['bar bar 42 undefined 42 false', 'bar bar 42 undefined 42 false'];
  run(code, expected);
});

test('arguments_in_arrow_func_2', () => {
  const code =
    '(function(a,b){console.log(arguments[0],a,arguments[1],arguments[3],b,arguments[2])})("bar",42,false);(function(a,b){(()=>{console.log(arguments[0],a,arguments[1],arguments[3],b,arguments[2])})(10,20,30,40)})("bar",42,false);';
  const expected = ['bar bar 42 undefined 42 false', 'bar bar 42 undefined 42 false'];
  run(code, expected);
});

test('arguments_and_destructuring_1', () => {
  const code = '(function({d:d}){console.log(a="foo",arguments[0].d)})({d:"Bar"});';
  const expected = ['foo Bar'];
  run(code, expected);
});

test('arguments_and_destructuring_2', () => {
  const code = '(function(a,{d:d}){console.log(a="foo",arguments[0])})("baz",{d:"Bar"});';
  const expected = ['foo baz'];
  run(code, expected);
});

test('arguments_and_destructuring_3', () => {
  const code = '(function({d:d},a){console.log(a="foo",arguments[0].d)})({d:"Bar"},"baz");';
  const expected = ['foo Bar'];
  run(code, expected);
});

test('duplicate_parameter_with_arguments', () => {
  const code = '(function(a,a){console.log(a="foo",arguments[0])})("baz","Bar");';
  const expected = ['foo baz'];
  run(code, expected);
});

test('for_loop', () => {
  const code =
    'function f0(){var a=[1,2,3];var b=0;for(var i=0;i<a.length;i++)b+=a[i];return b}function f1(){var a=[1,2,3];var b=0;for(var i=0,len=a.length;i<len;i++)b+=a[i];return b}function f2(){var a=[1,2,3];for(var i=0;i<a.length;i++)a[i]++;return a[2]}console.log(f0(),f1(),f2());';
  const expected = ['6 6 4'];
  run(code, expected);
});

test('index', () => {
  const code = 'var a=[1,2];console.log(a[0],a[1]);';
  const expected = ['1 2'];
  run(code, expected);
});

test('length', () => {
  const code = 'var a=[1,2];console.log(a.length);';
  const expected = ['2'];
  run(code, expected);
});

test('index_length', () => {
  const code = 'var a=[1,2];console.log(a[0],a.length);';
  const expected = ['1 2'];
  run(code, expected);
});

test('arrow_unused', () => {
  const code =
    'top=>dog;let fn=a=>{console.log(a*a)};let u=(x,y)=>x-y+g;(()=>{console.log("0")})();!function(x){(()=>{console.log("1")})();let unused=x=>{console.log(x)};let baz=e=>e+e;console.log(baz(x))}(1);fn(3);';
  const expected = ['0', '1', '2', '9'];
  run(code, expected);
});

test('arrow_unused_toplevel', () => {
  const code =
    'top=>dog;let fn=a=>{console.log(a*a)};let u=(x,y)=>x-y+g;(()=>{console.log("0")})();!function(x){(()=>{console.log("1")})();let unused=x=>{console.log(x)};let baz=e=>e+e;console.log(baz(x))}(1);fn(3);';
  const expected = ['0', '1', '2', '9'];
  run(code, expected);
});

test.skip('async_identifiers', () => {
  const code =
    'var async=function(x){console.log("async",x)};var await=function(x){console.log("await",x)};async(1);await(2);';
  const expected = ['async 1', 'await 2'];
  run(code, expected);
});

test('issue_2105_1', () => {
  const code =
    '!function(factory){factory()}((function(){return function(fn){fn()().prop()}((function(){function bar(){var quux=function(){console.log("PASS")},foo=function(){console.log;quux()};return{prop:foo}}return bar}))}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2105_2', () => {
  const code =
    '(factory=>{factory()})(()=>(fn=>{fn()().prop()})(()=>{let bar=()=>{var quux=()=>{console.log("PASS")},foo=()=>{console.log;quux()};return{prop:foo}};return bar}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2136_2', () => {
  const code = 'function f(x){console.log(x)}!function(a,...b){f(b[0])}(1,2,3);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2136_3', () => {
  const code = 'function f(x){console.log(x)}!function(a,...b){f(b[0])}(1,2,3);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2084', () => {
  const code =
    'var c=0;!function(){!function(c){c=1+c;var c=0;function f14(a_1){if(c=1+c,0!==23..toString())c=1+c,a_1&&(a_1[0]=0)}f14()}(-1)}();console.log(c);';
  const expected = ['0'];
  run(code, expected);
});

test('concise_methods_with_computed_property2', () => {
  const code = 'var foo={[[1]](v){return v}};console.log(foo[[1]]("PASS"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2271', () => {
  const code =
    'var Foo=function(){};Foo.prototype.set=function(value){this.value=value;return this};Foo.prototype.print=function(){console.log(this.value)};(new Foo).set("PASS").print();';
  const expected = ['PASS'];
  run(code, expected);
});

test('concise_method_with_super', () => {
  const code = 'var o={f:"FAIL",g(){return super.f}};Object.setPrototypeOf(o,{f:"PASS"});console.log(o.g());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3092a', () => {
  const code = 'console.log({*gen(x){return yield x.toUpperCase(),2}}.gen("pass").next().value);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3092b', () => {
  const code =
    'var obj={async bar(x){return await x,2},*gen(x){return yield x.toUpperCase(),2}};console.log(obj.gen("pass").next().value);';
  const expected = ['PASS'];
  run(code, expected);
});

test('async_inline', () => {
  const code =
    '(async function(){return await 3})();(async function(x){await console.log(x)})(4);function invoke(x,y){return x(y)}invoke((async function(){return await 1}));invoke((async function(x){await console.log(x)}),2);function top(){console.log("top")}top();async function async_top(){console.log("async_top")}async_top();';
  const expected = ['4', '2', 'top', 'async_top'];
  run(code, expected);
});

test.skip('async_identifiers', () => {
  const code =
    'var async=function(x){console.log("async",x)};var await=function(x){console.log("await",x)};async(1);await(2);';
  const expected = ['async 1', 'await 2'];
  run(code, expected);
});

test.skip('async_shorthand_property', () => {
  const code =
    'function print(o){console.log(o.async+" "+o.await)}var async="Async",await="Await";print({async:async});print({await:await});print({async:async,await:await});print({await:await,async:async});print({async:async});print({await:await});print({async:async,await:await});print({await:await,async:async});';
  const expected = [
    'Async undefined',
    'undefined Await',
    'Async Await',
    'Async Await',
    'Async undefined',
    'undefined Await',
    'Async Await',
    'Async Await',
  ];
  run(code, expected);
});

test('issue_3079', () => {
  const code =
    'async=>1;var async=async=>async;console.log(async(1));async=async=>async;console.log(async(2));console.log({m:async=>async?"3":"4"}.m(true));';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('issue_87', () => {
  const code = 'function async(async){console.log(async[0],async.prop)}async({0:1,prop:2});';
  const expected = ['1 2'];
  run(code, expected);
});

test('regression_block_scope_resolves', () => {
  const code =
    '(function(){if(1){let x;const y=1;class Zee{}}if(1){let ex;const why=2;class Zi{}}console.log(typeof x,typeof y,typeof Zee,typeof ex,typeof why,typeof Zi)})();';
  const expected = ['undefined undefined undefined undefined undefined undefined'];
  run(code, expected);
});

test('switch_block_scope_mangler', () => {
  const code =
    'var fn=function(code){switch(code){case 1:let apple=code+1;let dog=code+4;console.log(apple,dog);break;case 2:let banana=code+2;console.log(banana);break;default:let cat=code+3;console.log(cat)}};fn(1);fn(2);fn(3);';
  const expected = ['2 5', '4', '6'];
  run(code, expected);
});

test('issue_241', () => {
  const code =
    'var a={};(function(global){function fail(o){var result={};function inner(){return outer({one:o.one,two:o.two})}result.inner=function(){return inner()};return result}function outer(o){var ret;if(o){ret=o.one}else{ret=o.two}return ret}global.fail=fail})(a);var b=a.fail({one:"PASS"});console.log(b.inner());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_334', () => {
  const code =
    '(function(A){(function(){doPrint()})();function doPrint(){print(A)}})("Hello World!");function print(A){if(!A.x){console.log(A)}}';
  const expected = ['Hello World!'];
  run(code, expected);
});

test('issue_508', () => {
  const code =
    'const foo=()=>{let a;{let b=[];{console.log()}a=b;{let c=a;let b=123456;console.log(b);c.push(b)}}};foo();';
  const expected = ['', '123456'];
  run(code, expected);
});

test('issue_1664', () => {
  const code = 'var a=1;function f(){if(undefined)a=2;{function undefined(){}undefined()}}f();console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('basic_class_properties', () => {
  const code =
    'class A{static foo;bar;static fil="P";another="A";toString(){if("bar"in this&&"foo"in A){return A.fil+this.another}}}console.log((new A).toString());';
  const expected = ['PA'];
  run(code, expected);
});

test('computed_class_properties', () => {
  const code = 'const x="FOO";const y="BAR";class X{[x]="PASS";static[y]}if("BAR"in X){console.log((new X)[x])}';
  const expected = ['PASS'];
  run(code, expected);
});

test('static_class_properties_side_effects', () => {
  const code = 'class A{foo=console.log("PASS2");static bar=console.log("PASS1")}new A;';
  const expected = ['PASS1', 'PASS2'];
  run(code, expected);
});

test('class_expression_properties_side_effects', () => {
  const code = 'global.side=()=>{console.log("PASS")};(class{static foo=side();[side()](){}[side()]=4});';
  const expected = ['PASS', 'PASS', 'PASS'];
  run(code, expected);
});

test('static_property_side_effects', () => {
  const code = 'let x="FAIL";class cls{static[x="PASS"]}console.log(x);class cls2{static[console.log("PASS")]}';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('static_means_execution', () => {
  const code =
    'let x=0;class NoProps{}class WithProps{prop=x=x===1?"PASS":"FAIL"}class WithStaticProps{static prop=x=x===0?1:"FAIL"}new NoProps;new WithProps;new WithStaticProps;console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_vars_seq', () => {
  const code = 'var f1=function(x,y){var a,b,r=x+y,q=r*r,z=q-r;a=z,b=7;return a+b};console.log(f1(1,2));';
  const expected = ['13'];
  run(code, expected);
});

test('collapse_vars_throw', () => {
  const code = 'var f1=function(x,y){var a,b,r=x+y,q=r*r,z=q-r;a=z,b=7;throw a+b};try{f1(1,2)}catch(e){console.log(e)}';
  const expected = ['13'];
  run(code, expected);
});

test('collapse_vars_unary_2', () => {
  const code =
    'global.leak=n=>console.log(n);global.num=4;let counter=-1;for(const i in[0,1,2,3,4,5]){counter++,i==num&&leak(counter)}';
  const expected = ['4'];
  run(code, expected);
});

test('issue_1631_1', () => {
  const code = 'var pc=0;function f(x){pc=200;return 100}function x(){var t=f();pc+=t;return pc}console.log(x());';
  const expected = ['300'];
  run(code, expected);
});

test('issue_1631_2', () => {
  const code = 'var a=0,b=1;function f(){a=2;return 4}function g(){var t=f();b=a+t;return b}console.log(g());';
  const expected = ['6'];
  run(code, expected);
});

test('issue_1631_3', () => {
  const code = 'function g(){var a=0,b=1;function f(){a=2;return 4}var t=f();b=a+t;return b}console.log(g());';
  const expected = ['6'];
  run(code, expected);
});

test('reduce_vars_assign', () => {
  const code = '!function(){var a=1;a=[].length,console.log(a)}();';
  const expected = ['0'];
  run(code, expected);
});

test('var_defs', () => {
  const code = 'var f1=function(x,y){var a,b,r=x+y,q=r*r,z=q-r,a=z,b=7;console.log(a+b)};f1("1",0);';
  const expected = ['97'];
  run(code, expected);
});

test('switch_case_2', () => {
  const code = 'var a=1,b=2;switch(b++){case b:var c=a;var a;break}console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('switch_case_3', () => {
  const code = 'var a=1,b=2;switch(a){case a:var b;break;case b:break}console.log(b);';
  const expected = ['2'];
  run(code, expected);
});

test('modified', () => {
  const code =
    'function f1(b){var a=b;return b+a}function f2(b){var a=b;return b+++a}function f3(b){var a=b++;return b+a}function f4(b){var a=b++;return b+++a}function f5(b){var a=function(){return b}();return b+++a}console.log(f1(1),f2(1),f3(1),f4(1),f5(1));';
  const expected = ['2 2 3 3 2'];
  run(code, expected);
});

test('issue_1858', () => {
  const code = 'console.log(function(x){var a={},b=a.b=x;return a.b+b}(1));';
  const expected = ['2'];
  run(code, expected);
});

test('chained_3', () => {
  const code = 'console.log(function(a,b){var c=a,c=b;b++;return c}(1,2));';
  const expected = ['2'];
  run(code, expected);
});

test('unused_orig', () => {
  const code =
    'var a=1;console.log(function(b){var a;var c=b;for(var d in c){var a;return--b+c[0]}try{}catch(e){--b+a}a&&a.NaN}([2]),a);';
  const expected = ['3 1'];
  run(code, expected);
});

test('compound_assignment', () => {
  const code = 'var a;a=1;a+=a+2;console.log(a);';
  const expected = ['4'];
  run(code, expected);
});

test('issue_2187_1', () => {
  const code = 'var a=1;!function(foo){foo();var a=2;console.log(a)}((function(){console.log(a)}));';
  const expected = ['1', '2'];
  run(code, expected);
});

test('issue_2187_2', () => {
  const code = 'var b=1;console.log(function(a){return a&&++b}(b--));';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2187_3', () => {
  const code = 'var b=1;console.log(function(a){return a&&++b}(b--));';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2203_1', () => {
  const code =
    'a="FAIL";console.log({a:"PASS",b:function(){return function(c){return c.a}((String,Object,this))}}.b());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2203_2', () => {
  const code =
    'a="PASS";console.log({a:"FAIL",b:function(){return function(c){return c.a}((String,Object,function(){return this}()))}}.b());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2203_3', () => {
  const code =
    'a="FAIL";console.log({a:"PASS",b:function(){return function(c){return c.a}((String,Object,(()=>this)()))}}.b());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2203_4', () => {
  const code = 'a="FAIL";console.log({a:"PASS",b:function(){return(c=>c.a)((String,Object,(()=>this)()))}}.b());';
  const expected = ['PASS'];
  run(code, expected);
});

test('duplicate_argname', () => {
  const code = 'function f(){return"PASS"}console.log(function(a,a){f++;return a}("FAIL",f()));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2298', () => {
  const code =
    '!function(){function f(){var a=undefined;var undefined=a++;try{!function g(b){b[1]="foo"}();console.log("FAIL")}catch(e){console.log("PASS")}}f()}();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2313_1', () => {
  const code =
    'var a=0,b=0;var foo={get c(){a++;return 42},set c(c){b++},d:function(){this.c++;if(this.c)console.log(a,b)}};foo.d();';
  const expected = ['2 1'];
  run(code, expected);
});

test('issue_2313_2', () => {
  const code = 'var c=0;!function a(){a&&c++;var a=0;a&&c++}();console.log(c);';
  const expected = ['0'];
  run(code, expected);
});

test('issue_2319_1', () => {
  const code = 'console.log(function(a){return a}(!function(){return this}()));';
  const expected = ['false'];
  run(code, expected);
});

test('issue_2319_2', () => {
  const code = 'console.log(function(a){"use strict";return a}(!function(){return this}()));';
  const expected = ['false'];
  run(code, expected);
});

test('issue_2319_3', () => {
  const code = '"use strict";console.log(function(a){return a}(!function(){return this}()));';
  const expected = ['true'];
  run(code, expected);
});

test('issue_2365', () => {
  const code =
    'console.log(function(a){var b=a.f;a.f++;return b}({f:1}));console.log(function(){var a={f:1},b=a.f;a.f++;return b}());console.log({f:1,g:function(){var b=this.f;this.f++;return b}}.g());';
  const expected = ['1', '1', '1'];
  run(code, expected);
});

test('issue_2364_1', () => {
  const code =
    'function inc(obj){return obj.count++}function foo(){var first=arguments[0];var result=inc(first);return foo.amount=first.count,result}var data={count:0};var answer=foo(data);console.log(foo.amount,answer);';
  const expected = ['1 0'];
  run(code, expected);
});

test('issue_2364_3', () => {
  const code =
    'function inc(obj){return obj.count++}function foo(bar){var result=inc(bar);return foo.amount=bar.count,result}var data={count:0};var answer=foo(data);console.log(foo.amount,answer);';
  const expected = ['1 0'];
  run(code, expected);
});

test('issue_2364_4', () => {
  const code =
    'function inc(obj){return obj.count++}function foo(bar,baz){var result=inc(bar);return foo.amount=baz.count,result}var data={count:0};var answer=foo(data,data);console.log(foo.amount,answer);';
  const expected = ['1 0'];
  run(code, expected);
});

test('issue_2364_6', () => {
  const code = 'function f(a,b){var c=a.p;b.p="FAIL";return c}var o={p:"PASS"};console.log(f(o,o));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2364_7', () => {
  const code =
    'function f(a,b){var c=a.p;b.f();return c}var o={p:"PASS",f:function(){this.p="FAIL"}};console.log(f(o,o));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2364_8', () => {
  const code =
    'function f(a,b,c){var d=a[b.f=function(){return"PASS"}];return c.f(d)}var o={f:function(){return"FAIL"}};console.log(f({},o,o));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2364_9', () => {
  const code =
    'function f(a,b){var d=a();return b.f(d)}var o={f:function(){return"FAIL"}};console.log(f((function(){o.f=function(){return"PASS"}}),o));';
  const expected = ['PASS'];
  run(code, expected);
});

test('pure_getters_chain', () => {
  const code =
    'function o(t,r){var a=t[1],s=t[2],o=t[3],i=t[5];return a<=23&&s<=59&&o<=59&&(!r||i)}console.log(o([,23,59,59,,42],1));';
  const expected = ['42'];
  run(code, expected);
});

test('conditional_1', () => {
  const code =
    'function f(a,b){var c="";var d=b?">":"<";if(a)c+="=";return c+=d}console.log(f(0,0),f(0,1),f(1,0),f(1,1));';
  const expected = ['< > =< =>'];
  run(code, expected);
});

test('conditional_2', () => {
  const code = 'function f(a,b){var c=a+1,d=a+2;return b?c:d}console.log(f(3,0),f(4,1));';
  const expected = ['5 5'];
  run(code, expected);
});

test('issue_2425_1', () => {
  const code = 'var a=8;(function(b){b.toString()})(--a,a|=10);console.log(a);';
  const expected = ['15'];
  run(code, expected);
});

test('issue_2425_2', () => {
  const code = 'var a=8;(function(b,c){b.toString()})(--a,a|=10);console.log(a);';
  const expected = ['15'];
  run(code, expected);
});

test('issue_2425_3', () => {
  const code = 'var a=8;(function(b,b){b.toString()})(--a,a|=10);console.log(a);';
  const expected = ['15'];
  run(code, expected);
});

test('issue_2437_1', () => {
  const code =
    'function XMLHttpRequest(){this.onreadystatechange="PASS"}global.xhrDesc={};function foo(){return bar()}function bar(){if(xhrDesc){var req=new XMLHttpRequest;var result=req.onreadystatechange;Object.defineProperty(XMLHttpRequest.prototype,"onreadystatechange",xhrDesc||{});return result}}console.log(foo());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2437_2', () => {
  const code =
    'function XMLHttpRequest(){this.onreadystatechange="PASS"}global.SYMBOL_FAKE_ONREADYSTATECHANGE_1=Symbol();global.xhrDesc=null;function foo(){return bar()}function bar(){if(!xhrDesc){var req=new XMLHttpRequest;var detectFunc=function(){};req.onreadystatechange=detectFunc;var result=req[SYMBOL_FAKE_ONREADYSTATECHANGE_1]===detectFunc;req.onreadystatechange=null;return result}}console.log(foo());';
  const expected = ['false'];
  run(code, expected);
});

test('issue_2453', () => {
  const code = 'function log(n){console.log(n)}const a=42;log(a);';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2436_10', () => {
  const code =
    'var o={a:1,b:2};function f(n){o={b:3};return n}console.log(function(c){return[c.a,f(c.b),c.b]}(o).join(" "));';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('issue_2436_13', () => {
  const code =
    'var a="PASS";(function(){function f(b){(function g(b){var b=b&&(b.null="FAIL")})(a)}f()})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2506', () => {
  const code =
    'var c=0;function f0(bar){function f1(Infinity_2){function f13(NaN){if(false<=NaN&this>>1>=0){c++}}var b_2=f13(NaN,c++)}var bar=f1(-3,-1)}f0(false);console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2571_1', () => {
  const code = 'var b=1;try{var a=function f0(c){throw c}(2);var d=--b+a}catch(e){}console.log(b);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2571_2', () => {
  const code = 'try{var a=A,b=1;throw a}catch(e){console.log(b)}';
  const expected = ['undefined'];
  run(code, expected);
});

test('may_throw_2', () => {
  const code = 'function f(b){try{var a=x();++b;return b(a)}catch(e){}console.log(b)}f(0);';
  const expected = ['0'];
  run(code, expected);
});

test('replace_all_var', () => {
  const code = 'var a="PASS";(function(){var b=b||c&&c[a="FAIL"],c=a})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('cascade_forin', () => {
  const code = 'var a;function f(b){return[b,b,b]}for(var c in a=console,f(a))console.log(c);';
  const expected = ['0', '1', '2'];
  run(code, expected);
});

test('unsafe_builtin', () => {
  const code = 'function f(a){var b=Math.abs(a);return Math.pow(b,2)}console.log(f(-1),f(2));';
  const expected = ['1 4'];
  run(code, expected);
});

test('return_1', () => {
  const code = 'var log=console.log;function f(b,c){var a=c;if(b)return b;log(a)}f(false,1);f(true,2);';
  const expected = ['1'];
  run(code, expected);
});

test('return_2', () => {
  const code =
    'var log=console.log;function f(b,c){var a=c();if(b)return b;log(a)}f(false,(function(){return 1}));f(true,(function(){return 2}));';
  const expected = ['1'];
  run(code, expected);
});

test('return_3', () => {
  const code = 'var log=console.log;function f(b,c){var a=b<<=c;if(b)return b;log(a)}f(false,1);f(true,2);';
  const expected = ['0'];
  run(code, expected);
});

test('return_4', () => {
  const code = 'var a="FAIL";(function(b){a="PASS";return;b(a)})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test.skip('issue_2858', () => {
  const code = 'var b;(function(){function f(){a++}f();var c=f();var a=void 0;c||(b=a)})();console.log(b);';
  const expected = ['undefined'];
  run(code, expected);
});

test('cond_branch_1', () => {
  const code =
    'function f1(b,c){var log=console.log;var a=++c;if(b)b++;log(a,b)}function f2(b,c){var log=console.log;var a=++c;b&&b++;log(a,b)}function f3(b,c){var log=console.log;var a=++c;b?b++:b--;log(a,b)}f1(1,2);f2(3,4);f3(5,6);';
  const expected = ['3 2', '5 4', '7 6'];
  run(code, expected);
});

test('cond_branch_2', () => {
  const code =
    'function f1(b,c){var log=console.log;var a=++c;if(b)b+=a;log(a,b)}function f2(b,c){var log=console.log;var a=++c;b&&(b+=a);log(a,b)}function f3(b,c){var log=console.log;var a=++c;b?b+=a:b--;log(a,b)}f1(1,2);f2(3,4);f3(5,6);';
  const expected = ['3 4', '5 8', '7 12'];
  run(code, expected);
});

test('cond_branch_switch', () => {
  const code = 'var c=0;if(c=1+c,0)switch(c=1+c){}console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2873_1', () => {
  const code = 'var b=1,c=0;do{c++;if(!--b)break;c=1+c}while(0);console.log(b,c);';
  const expected = ['0 1'];
  run(code, expected);
});

test('issue_2873_2', () => {
  const code = 'var b=1,c=0;do{c++;if(!--b)continue;c=1+c}while(0);console.log(b,c);';
  const expected = ['0 1'];
  run(code, expected);
});

test('issue_2878', () => {
  const code = 'var c=0;(function(a,b){function f2(){if(a)c++}b=f2();a=1;b&&b.b;f2()})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2891_1', () => {
  const code = 'var a="PASS",b;try{b=c.p=0;a="FAIL";b()}catch(e){}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2908', () => {
  const code = 'var a=0,b=0;function f(c){if(1==c)return;a++;if(2==c)b=a}f(0);f(2);console.log(b);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_3096', () => {
  const code = 'console.log(function(){var ar=["a","b"];var first=ar.pop();return ar+""+first}());';
  const expected = ['ab'];
  run(code, expected);
});

test('issue_2914_1', () => {
  const code =
    'function read(input){var i=0;var e=0;var t=0;while(e<32){var n=input[i++];t|=(127&n)<<e;if(0===(128&n))return t;e+=7}}console.log(read([129]));';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2914_2', () => {
  const code =
    'function read(input){var i=0;var e=0;var t=0;while(e<32){var n=input[i++];t=(127&n)<<e;if(0===(128&n))return t;e+=7}}console.log(read([129]));';
  const expected = ['0'];
  run(code, expected);
});

test('issue_2931', () => {
  const code = 'console.log(function(){var a=function(){return}();return a}());';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2954_1', () => {
  const code = 'var a="PASS",b;try{do{b=function(){throw 0}();a="FAIL";b&&b.c}while(0)}catch(e){}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_conditional_1', () => {
  const code = 'var a="PASS",b="FAIL";b=a;"function"==typeof f&&f(a);console.log(a,b);';
  const expected = ['PASS PASS'];
  run(code, expected);
});

test('collapse_rhs_conditional_2', () => {
  const code = 'var a="FAIL",b;while((a="PASS",--b)&&"PASS"==b);console.log(a,b);';
  const expected = ['PASS NaN'];
  run(code, expected);
});

test('collapse_rhs_lhs_1', () => {
  const code = 'var c=0;new function(){this[c++]=1;c+=1};console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('collapse_rhs_lhs_2', () => {
  const code = 'var b=1;(function f(f){f=b;f[b]=0})();console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_loop', () => {
  const code =
    'var s;s="<tpl>PASS</tpl>";for(var m,r=/<tpl>(.*)<\\/tpl>/;m=s.match(r);)s=s.replace(m[0],m[1]);console.log(s);';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_side_effects', () => {
  const code = 'var a=1,c=0;new function f(){this[a--&&f()]=1;c+=1};console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('collapse_rhs_vardef', () => {
  const code = 'var a,b=1;a=--b+function c(){var b;c[--b]=1}();b|=a;console.log(a,b);';
  const expected = ['NaN 0'];
  run(code, expected);
});

test('collapse_rhs_array', () => {
  const code = 'var a,b;function f(){a=[];b=[];return[]}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['false false false'];
  run(code, expected);
});

test('collapse_rhs_boolean_1', () => {
  const code = 'var a,b;function f(){a=!0;b=!0;return!0}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('collapse_rhs_boolean_2', () => {
  const code =
    'var a;(function f1(){a=function(){};if(/foo/)console.log(typeof a)})();console.log(function f2(){a=[];return!1}());';
  const expected = ['function', 'false'];
  run(code, expected);
});

test('collapse_rhs_function', () => {
  const code =
    'var a,b;function f(){a=function(){};b=function(){};return function(){}}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['false false false'];
  run(code, expected);
});

test('collapse_rhs_number', () => {
  const code = 'var a,b;function f(){a=42;b=42;return 42}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('collapse_rhs_object', () => {
  const code = 'var a,b;function f(){a={};b={};return{}}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['false false false'];
  run(code, expected);
});

test('collapse_rhs_regexp', () => {
  const code = 'var a,b;function f(){a=/bar/;b=/bar/;return/bar/}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['false false false'];
  run(code, expected);
});

test('collapse_rhs_string', () => {
  const code = 'var a,b;function f(){a="foo";b="foo";return"foo"}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('collapse_rhs_var', () => {
  const code = 'var a,b;function f(){a=f;b=f;return f}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('collapse_rhs_this', () => {
  const code = 'var a,b;function f(){a=this;b=this;return this}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('collapse_rhs_undefined', () => {
  const code = 'var a,b;function f(){a=void 0;b=void 0;return void 0}var c=f();console.log(a===b,b===c,c===a);';
  const expected = ['true true true'];
  run(code, expected);
});

test('issue_2974', () => {
  const code =
    'var c=0;(function f(b){var a=2;do{b&&b[b];b&&(b.null=-4);c++}while(b.null&&--a>0)})(true);console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3032', () => {
  const code = 'console.log({f:function(){this.a=42;return[this.a,!1]}}.f()[0]);';
  const expected = ['42'];
  run(code, expected);
});

test('issue_805', () => {
  const code =
    'function f(){function Foo(){}Foo.prototype={};Foo.prototype.bar=42;return Foo}console.log((new(f())).bar);';
  const expected = ['42'];
  run(code, expected);
});

test('replace_all_var_scope', () => {
  const code = 'var a=100,b=10;(function(r,a){switch(~a){case b+=a:case a++:}})(--b,a);console.log(a,b);';
  const expected = ['100 109'];
  run(code, expected);
});

test('issue_348', () => {
  const code =
    'console.log(function x(EEE){return function(tee){if(tee){const EEE=tee;if(EEE)return EEE}}(EEE)}("PASS"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('ignore_class', () => {
  const code =
    'global.leak=x=>class dummy{get pass(){return x}};global.module={};(function(){const SuperClass=leak("PASS");class TheClass extends SuperClass{}module.exports=TheClass})();console.log((new module.exports).pass);';
  const expected = ['PASS'];
  run(code, expected);
});

test('comment_moved_between_return_and_value', () => {
  const code =
    'console.log(function(same_name){function licensed(same_name){return same_name.toUpperCase()}console.log("PASS");return licensed("PA")+"SS"}());';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('self_comparison_2', () => {
  const code = 'function f(){}var o={};console.log(f!=f,o===o);';
  const expected = ['false true'];
  run(code, expected);
});

test('issue_2857_6', () => {
  const code =
    'function f(a){if({}.b===undefined||{}.b===null)return a.b!==undefined&&a.b!==null}console.log(f({a:[null],get b(){return this.a.shift()}}));';
  const expected = ['true'];
  run(code, expected);
});

test('condition_symbol_matches_consequent', () => {
  const code =
    'function foo(x,y){return x?x:y}function bar(){return g?g:h}var g=4;var h=5;console.log(foo(3,null),foo(0,7),foo(true,false),bar());';
  const expected = ['3 7 true 4'];
  run(code, expected);
});

test('issue_2535_2', () => {
  const code =
    'function x(){}function y(){return"foo"}console.log(x()||true||y());console.log(y()||true||x());console.log((x()||true)&&y());console.log((y()||true)&&x());console.log(x()&&true||y());console.log(y()&&true||x());console.log(x()&&true&&y());console.log(y()&&true&&x());console.log(x()||false||y());console.log(y()||false||x());console.log((x()||false)&&y());console.log((y()||false)&&x());console.log(x()&&false||y());console.log(y()&&false||x());console.log(x()&&false&&y());console.log(y()&&false&&x());';
  const expected = [
    'true',
    'foo',
    'foo',
    'undefined',
    'foo',
    'true',
    'undefined',
    'undefined',
    'foo',
    'foo',
    'false',
    'undefined',
    'foo',
    'undefined',
    'undefined',
    'false',
  ];
  run(code, expected);
});

test('issue_2560', () => {
  const code =
    'function log(x){console.log(x)}function foo(){return log}function bar(){if(x!==(x=foo())){x(1)}else{x(2)}}var x=function(){console.log("init")};bar();bar();';
  const expected = ['1', '2'];
  run(code, expected);
});

test('issue_2994', () => {
  const code =
    'function f(condition1,condition2,condition3){if(condition1){if(condition2){return aValue}else{const variable1="something";if(condition3){const variable2="else";return anotherValue}else{return undefined}}}}let aValue=2,anotherValue=3;for(let i=0;i<8;++i){console.log(f(i&4,i&2,i&1))}';
  const expected = ['undefined', 'undefined', 'undefined', 'undefined', 'undefined', '3', '2', '2'];
  run(code, expected);
});

test.skip('try_catch_finally', () => {
  const code =
    'var a=1;!function(){try{if(false)throw x}catch(a){var a=2;console.log("FAIL")}finally{a=3;console.log("PASS")}}();try{console.log(a)}finally{}';
  const expected = ['PASS', '1'];
  run(code, expected);
});

test('global_fns', () => {
  const code =
    'Boolean(1,2);decodeURI(1,2);decodeURIComponent(1,2);Date(1,2);encodeURI(1,2);encodeURIComponent(1,2);Error(1,2);escape(1,2);EvalError(1,2);isFinite(1,2);isNaN(1,2);Number(1,2);Object(1,2);parseFloat(1,2);parseInt(1,2);RangeError(1,2);ReferenceError(1,2);String(1,2);SyntaxError(1,2);TypeError(1,2);unescape(1,2);URIError(1,2);try{Function(1,2)}catch(e){console.log(e.name)}try{RegExp(1,2)}catch(e){console.log(e.name)}try{Array(NaN)}catch(e){console.log(e.name)}';
  const expected = ['SyntaxError', 'SyntaxError', 'RangeError'];
  run(code, expected);
});

test('issue_2383_2', () => {
  const code = 'if(0){var{x:x=0,y:[w,,{z:z,p:q=7}]=[1,2,{z:3}]}={}}console.log(x,q,w,z);';
  const expected = ['undefined undefined undefined undefined'];
  run(code, expected);
});

test('issue_2383_3', () => {
  const code = 'var b=7,y=8;if(0){var a=1,[x,y,z]=[2,3,4],b=5}console.log(a,x,y,z,b);';
  const expected = ['undefined undefined 8 undefined 7'];
  run(code, expected);
});

test('return_assignment', () => {
  const code =
    'function f1(a,b,c){return a=x(),b=y(),b=a&&(c>>=5)}function f2(){return e=x()}function f3(e){return e=x()}function f4(){var e;return e=x()}function f5(a){try{return a=x()}catch(b){console.log(a)}}function f6(a){try{return a=x()}finally{console.log(a)}}function y(){console.log("y")}function test(inc){var counter=0;x=function(){counter+=inc;if(inc<0)throw counter;return counter};[f1,f2,f3,f4,f5,f6].forEach((function(f,i){e=null;try{i+=1;console.log("result "+f(10*i,100*i,1e3*i))}catch(x){console.log("caught "+x)}if(null!==e)console.log("e: "+e)}))}var x,e;test(1);test(-1);';
  const expected = [
    'y',
    'result 31',
    'result 2',
    'e: 2',
    'result 3',
    'result 4',
    'result 5',
    '6',
    'result 6',
    'caught -1',
    'caught -2',
    'caught -3',
    'caught -4',
    '50',
    'result undefined',
    '60',
    'caught -6',
  ];
  run(code, expected);
});

test.skip('throw_assignment', () => {
  const code =
    'function f1(){throw a=x()}function f2(a){throw a=x()}function f3(){var a;throw a=x()}function f4(){try{throw a=x()}catch(b){console.log(a)}}function f5(a){try{throw a=x()}catch(b){console.log(a)}}function f6(){var a;try{throw a=x()}catch(b){console.log(a)}}function f7(){try{throw a=x()}finally{console.log(a)}}function f8(a){try{throw a=x()}finally{console.log(a)}}function f9(){var a;try{throw a=x()}finally{console.log(a)}}function test(inc){var counter=0;x=function(){counter+=inc;if(inc<0)throw counter;return counter};[f1,f2,f3,f4,f5,f6,f7,f8,f9].forEach((function(f,i){a=null;try{f(10*(1+i))}catch(x){console.log("caught "+x)}if(null!==a)console.log("a: "+a)}))}var x,a;test(1);test(-1);';
  const expected = [
    'caught 1',
    'a: 1',
    'caught 2',
    'caught 3',
    '4',
    'a: 4',
    '5',
    '6',
    '7',
    'caught 7',
    'a: 7',
    '8',
    'caught 8',
    '9',
    'caught 9',
    'caught -1',
    'caught -2',
    'caught -3',
    'null',
    '50',
    'undefined',
    'null',
    'caught -7',
    '80',
    'caught -8',
    'undefined',
    'caught -9',
  ];
  run(code, expected);
});

test('issue_2597', () => {
  const code =
    'function f(b){try{try{throw"foo"}catch(e){return b=true}}finally{b&&(a="PASS")}}var a="FAIL";f();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2666', () => {
  const code = 'function f(a){return a={p:function(){return a}}}console.log(typeof f().p());';
  const expected = ['object'];
  run(code, expected);
});

test('issue_2692', () => {
  const code = 'function f(a){return a=g;function g(){return a}}console.log(typeof f()());';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2701', () => {
  const code = 'function f(a){return a=function(){return function(){return a}}()}console.log(typeof f()());';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2749', () => {
  const code = 'var a=2,c="PASS";while(a--)(function(){return b?c="FAIL":b=1;try{}catch(b){var b}})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2860_1', () => {
  const code = 'console.log(function(a){return a^=1}());';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2860_2', () => {
  const code = 'console.log(function(a){return a^=1}());';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2929', () => {
  const code = 'console.log(function(a){try{return null.p=a=1}catch(e){return a?"PASS":"FAIL"}}());';
  const expected = ['PASS'];
  run(code, expected);
});

test('defaults_undefined', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('defaults_false', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('defaults_false_evaluate_true', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('defaults_true', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('defaults_true_conditionals_false', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('defaults_true_evaluate_false', () => {
  const code = 'if(true){console.log(1+2)}';
  const expected = ['3'];
  run(code, expected);
});

test('destructuring_decl_of_numeric_key', () => {
  const code = 'let{3:x}={[1+2]:42};console.log(x);';
  const expected = ['42'];
  run(code, expected);
});

test('destructuring_decl_of_computed_key', () => {
  const code = 'let four=4;let{[7-four]:x}={[1+2]:42};console.log(x);';
  const expected = ['42'];
  run(code, expected);
});

test('destructuring_assign_of_numeric_key', () => {
  const code = 'let x;({3:x}={[1+2]:42});console.log(x);';
  const expected = ['42'];
  run(code, expected);
});

test('destructuring_assign_of_computed_key', () => {
  const code = 'let x;let four=4;({[5+2-four]:x}={[1+2]:42});console.log(x);';
  const expected = ['42'];
  run(code, expected);
});

test('mangle_destructuring_decl', () => {
  const code =
    'function test(opts){let a=opts.a||{e:7,n:8};let{t:t,e:e,n:n,s:s=5+4,o:o,r:r}=a;console.log(t,e,n,s,o,r)}test({a:{t:1,e:2,n:3,s:4,o:5,r:6}});test({});';
  const expected = ['1 2 3 4 5 6', 'undefined 7 8 9 undefined undefined'];
  run(code, expected);
});

test('mangle_destructuring_decl_collapse_vars', () => {
  const code =
    'function test(opts){let a=opts.a||{e:7,n:8};let{t:t,e:e,n:n,s:s=5+4,o:o,r:r}=a;console.log(t,e,n,s,o,r)}test({a:{t:1,e:2,n:3,s:4,o:5,r:6}});test({});';
  const expected = ['1 2 3 4 5 6', 'undefined 7 8 9 undefined undefined'];
  run(code, expected);
});

test('mangle_destructuring_assign_toplevel_true', () => {
  const code =
    'function test(opts){let s,o,r;let a=opts.a||{e:7,n:8};({t,e,n,s=5+4,o,r}=a);console.log(t,e,n,s,o,r)}let t,e,n;test({a:{t:1,e:2,n:3,s:4,o:5,r:6}});test({});';
  const expected = ['1 2 3 4 5 6', 'undefined 7 8 9 undefined undefined'];
  run(code, expected);
});

test('mangle_destructuring_assign_toplevel_false', () => {
  const code =
    'function test(opts){let s,o,r;let a=opts.a||{e:7,n:8};({t,e,n,s=9,o,r}=a);console.log(t,e,n,s,o,r)}let t,e,n;test({a:{t:1,e:2,n:3,s:4,o:5,r:6}});test({});';
  const expected = ['1 2 3 4 5 6', 'undefined 7 8 9 undefined undefined'];
  run(code, expected);
});

test('mangle_destructuring_decl_array', () => {
  const code = 'var[,t,e,n,s,o=2,r=[1+2]]=[9,8,7,6];console.log(t,e,n,s,o,r);';
  const expected = ['8 7 6 undefined 2 [3]'];
  run(code, expected);
});

test('anon_func_with_destructuring_args', () => {
  const code = '(function({foo:foo=1+0,bar:bar=2},[car=3,far=4]){console.log(foo,bar,car,far)})({bar:5-0},[,6]);';
  const expected = ['1 5 3 6'];
  run(code, expected);
});

test('arrow_func_with_destructuring_args', () => {
  const code = '(({foo:foo=1+0,bar:bar=2},[car=3,far=4])=>{console.log(foo,bar,car,far)})({bar:5-0},[,6]);';
  const expected = ['1 5 3 6'];
  run(code, expected);
});

test('issue_2140', () => {
  const code = '!function(){var t={};console.log(([t.a]=[42])[0])}();';
  const expected = ['42'];
  run(code, expected);
});

test('issue_3205_1', () => {
  const code = 'function f(a){function g(){var{b:b,c:c}=a;console.log(b,c)}g()}f({b:2,c:3});';
  const expected = ['2 3'];
  run(code, expected);
});

test('issue_3205_2', () => {
  const code = '(function(){function f(){var o={a:"PASS"},{a:x}=o;console.log(x)}f()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3205_3', () => {
  const code = '(function(){function f(o,{a:x}=o){console.log(x)}f({a:"PASS"})})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3205_4', () => {
  const code = '(function(){function f(o){var{a:x}=o;console.log(x)}f({a:"PASS"})})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3205_5', () => {
  const code = '(function(){function f(g){var o=g,{a:x}=o;console.log(x)}f({a:"PASS"})})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_destructuring_decl_1', () => {
  const code = 'let{x:L,y:y}={x:2};var{U:u,V:V}={V:3};const{C:C,D:D}={C:1,D:4};console.log(L,V);';
  const expected = ['2 3'];
  run(code, expected);
});

test('unused_destructuring_decl_2', () => {
  const code =
    'const{a:a,b:c,d:d=new Object(1)}={b:7};let{e:e,f:g,h:h=new Object(2)}={e:8};var{w:w,x:y,z:z=new Object(3)}={w:4,x:5,y:6};console.log(c,e,z+0);';
  const expected = ['7 8 3'];
  run(code, expected);
});

test('unused_destructuring_decl_3', () => {
  const code =
    'const{a:a,b:c,d:d=new Object(1)}={b:7};let{e:e,f:g,h:h=new Object(2)}={e:8};var{w:w,x:y,z:z=new Object(3)}={w:4,x:5,y:6};console.log(c,e,z+0);';
  const expected = ['7 8 3'];
  run(code, expected);
});

test('unused_destructuring_decl_4', () => {
  const code =
    'const{a:a,b:c,d:d=new Object(1)}={b:7};let{e:e,f:g,h:h=new Object(2)}={e:8};var{w:w,x:y,z:z=new Object(3)}={w:4,x:5,y:6};console.log(c,e,z+0);';
  const expected = ['7 8 3'];
  run(code, expected);
});

test('unused_destructuring_decl_5', () => {
  const code =
    'const{a:a,b:c,d:d=new Object(1)}={b:7};let{e:e,f:g,h:h=new Object(2)}={e:8};var{w:w,x:y,z:z=new Object(3)}={w:4,x:5,y:6};console.log(c,e,z+0);';
  const expected = ['7 8 3'];
  run(code, expected);
});

test('unused_destructuring_function_param', () => {
  const code = 'function foo({w:w=console.log("side effect"),x:x,y:z}){console.log(x)}foo({x:1,y:2,z:3});';
  const expected = ['side effect', '1'];
  run(code, expected);
});

test('unused_destructuring_arrow_param', () => {
  const code = 'let bar=({w:w=console.log("side effect"),x:x,y:z})=>{console.log(x)};bar({x:4,y:5,z:6});';
  const expected = ['side effect', '4'];
  run(code, expected);
});

test('unused_destructuring_object_method_param', () => {
  const code = '({baz({w:w=console.log("side effect"),x:x,y:z}){console.log(x)}}).baz({x:7,y:8,z:9});';
  const expected = ['side effect', '7'];
  run(code, expected);
});

test('unused_destructuring_class_method_param', () => {
  const code = '(new class{baz({w:w=console.log("side effect"),x:x,y:z}){console.log(x)}}).baz({x:7,y:8,z:9});';
  const expected = ['side effect', '7'];
  run(code, expected);
});

test('unused_destructuring_getter_side_effect_1', () => {
  const code =
    'function extract(obj){const{a:a,b:b}=obj;console.log(b)}extract({a:1,b:2});extract({get a(){var s="side effect";console.log(s);return s},b:4});';
  const expected = ['2', 'side effect', '4'];
  run(code, expected);
});

test('unused_destructuring_assign_1', () => {
  const code = 'function extract(obj){var a;let b;({a:a,b:b}=obj);console.log(b)}extract({a:1,b:2});extract({b:4});';
  const expected = ['2', '4'];
  run(code, expected);
});

test('unused_destructuring_assign_2', () => {
  const code =
    'function extract(obj){var a;let b;({a:a,b:b}=obj);console.log(b)}extract({a:1,b:2});extract({get a(){var s="side effect";console.log(s);return s},b:4});';
  const expected = ['2', 'side effect', '4'];
  run(code, expected);
});

test('unused_destructuring_declaration_complex_1', () => {
  const code = 'const[,w,,x,{y:y,z:z}]=[1,2,3,4,{z:5}];console.log(x,z);';
  const expected = ['4 5'];
  run(code, expected);
});

test('unused_destructuring_declaration_complex_2', () => {
  const code = 'const[,w,,x,{y:y,z:z}]=[1,2,3,4,{z:5}];console.log(x,z);';
  const expected = ['4 5'];
  run(code, expected);
});

test('unused_destructuring_multipass', () => {
  const code = 'let{w:w,x:y,z:z}={x:1,y:2,z:3};console.log(y);if(0){console.log(z)}';
  const expected = ['1'];
  run(code, expected);
});

test('issue_t111_1', () => {
  const code = 'var p=x=>(console.log(x),x),unused=p(1),{}=p(2);';
  const expected = ['1', '2'];
  run(code, expected);
});

test('issue_t111_2a', () => {
  const code = 'var p=x=>(console.log(x),x),a=p(1),{}=p(2),c=p(3),d=p(4);';
  const expected = ['1', '2', '3', '4'];
  run(code, expected);
});

test('issue_t111_2b', () => {
  const code = 'let p=x=>(console.log(x),x),a=p(1),{}=p(2),c=p(3),d=p(4);';
  const expected = ['1', '2', '3', '4'];
  run(code, expected);
});

test('issue_t111_2c', () => {
  const code = 'const p=x=>(console.log(x),x),a=p(1),{}=p(2),c=p(3),d=p(4);';
  const expected = ['1', '2', '3', '4'];
  run(code, expected);
});

test('issue_t111_3', () => {
  const code = 'let p=x=>(console.log(x),x),a=p(1),{}=p(2),c=p(3),{}=p(4);';
  const expected = ['1', '2', '3', '4'];
  run(code, expected);
});

test('issue_t111_4', () => {
  const code = 'let p=x=>(console.log(x),x),a=1,{length:length}=[0],c=3,{x:x}={x:2};p(`${length} ${x}`);';
  const expected = ['1 2'];
  run(code, expected);
});

test('empty_object_destructuring_1', () => {
  const code =
    'var{}=Object;let{L:L}=Object,L2="foo";const bar="bar",{prop:C1,C2:C2=console.log("side effect"),C3:C3}=Object;';
  const expected = ['side effect'];
  run(code, expected);
});

test('empty_object_destructuring_2', () => {
  const code =
    'var{}=Object;let{L:L}=Object,L2="foo";const bar="bar",{prop:C1,C2:C2=console.log("side effect"),C3:C3}=Object;';
  const expected = ['side effect'];
  run(code, expected);
});

test('empty_object_destructuring_3', () => {
  const code =
    'var{}=Object;let{L:L}=Object,L2="foo";const bar="bar",{prop:C1,C2:C2=console.log("side effect"),C3:C3}=Object;';
  const expected = ['side effect'];
  run(code, expected);
});

test('empty_object_destructuring_4', () => {
  const code =
    'var{}=Object;let{L:L}=Object,L2="foo";const bar="bar",{prop:C1,C2:C2=console.log("side effect"),C3:C3}=Object;';
  const expected = ['side effect'];
  run(code, expected);
});

test('empty_object_destructuring_misc', () => {
  const code =
    'let out=[],foo=(out.push(0),1),{}={k:9},bar=out.push(2),{unused:unused}=(out.push(3),{unused:7}),{a:b,prop:prop,w:w,x:y,z:z}={prop:8},baz=(out.push(4),5);console.log(`${foo} ${prop} ${baz} ${JSON.stringify(out)}`);';
  const expected = ['1 8 5 [0,2,3,4]'];
  run(code, expected);
});

test('simple_statement_is_not_a_directive', () => {
  const code =
    '"use strict".split(" ").forEach((function(s){console.log(s)}));console.log(!this);(function(){"directive";"";"use strict";"hello world".split(" ").forEach((function(s){console.log(s)}));console.log(!this)})();';
  const expected = ['use', 'strict', 'false', 'hello', 'world', 'true'];
  run(code, expected);
});

test('issue_1715_1', () => {
  const code = 'var a=1;function f(){a++;try{x()}catch(a){var a}}f();console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1715_2', () => {
  const code = 'var a=1;function f(){a++;try{x()}catch(a){var a=2}}f();console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1715_3', () => {
  const code = 'var a=1;function f(){a++;try{console}catch(a){var a=2+x()}}f();console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1715_4', () => {
  const code = 'var a=1;!function a(){a++;try{x()}catch(a){var a}}();console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('drop_var', () => {
  const code = 'var a;console.log(a,b);var a=1,b=2;console.log(a,b);var a=3;console.log(a,b);';
  const expected = ['undefined undefined', '1 2', '3 2'];
  run(code, expected);
});

test('issue_1830_1', () => {
  const code = '!function(){L:for(var b=console.log(1);!1;)continue L}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1830_2', () => {
  const code = '!function(){L:for(var a=1,b=console.log(a);--a;)continue L}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1968', () => {
  const code = 'function f(c){var a;if(c){let b;return(a=2)+(b=3)}}console.log(f(1));';
  const expected = ['5'];
  run(code, expected);
});

test('issue_2105_1', () => {
  const code =
    '!function(factory){factory()}((function(){return function(fn){fn()().prop()}((function(){function bar(){var quux=function(){console.log("PASS")},foo=function(){console.log;quux()};return{prop:foo}}return bar}))}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2105_2', () => {
  const code =
    '!function(factory){factory()}((function(){return function(fn){fn()().prop()}((function(){function bar(){var quux=function(){console.log("PASS")},foo=function(){console.log;quux()};return{prop:foo}}return bar}))}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2136_1', () => {
  const code = '!function(a,...b){console.log(b)}();';
  const expected = ['[]'];
  run(code, expected);
});

test('issue_2136_2', () => {
  const code = 'function f(x){console.log(x)}!function(a,...b){f(b[0])}(1,2,3);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2136_3', () => {
  const code = 'function f(x){console.log(x)}!function(a,...b){f(b[0])}(1,2,3);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2226_2', () => {
  const code = 'console.log(function(a,b){a+=b;return a}(1,2));';
  const expected = ['3'];
  run(code, expected);
});

test('issue_2226_3', () => {
  const code = 'console.log(function(a,b){a+=b;return a}(1,2));';
  const expected = ['3'];
  run(code, expected);
});

test('defun_lambda_same_name', () => {
  const code = 'function f(n){return n?n*f(n-1):1}console.log(function f(n){return n?n*f(n-1):1}(5));';
  const expected = ['120'];
  run(code, expected);
});

test('issue_2660_1', () => {
  const code = 'var a=2;function f(b){return b&&f()||a--}f(1);console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2660_2', () => {
  const code = 'var a=1;function f(b){b&&f();--a,a.toString()}f();console.log(a);';
  const expected = ['0'];
  run(code, expected);
});

test('issue_2665', () => {
  const code =
    'var a=1;function g(){a--&&g()}typeof h=="function"&&h();function h(){typeof g=="function"&&g()}console.log(a);';
  const expected = ['-1'];
  run(code, expected);
});

test('cascade_drop_assign', () => {
  const code = 'var a,b=a="PASS";console.log(b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('chained_3', () => {
  const code = 'console.log(function(a,b){var c=a,c=b;b++;return c}(1,2));';
  const expected = ['2'];
  run(code, expected);
});

test('function_argument_modified_by_function_statement', () => {
  const code = 'var printTest=function(ret){function ret(){console.log("PASS")}return ret}("FAIL");printTest();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2768', () => {
  const code = 'var a="FAIL",c=1;var c=function(b){var d=b=a;var e=--b+(d&&(a="PASS"))}();console.log(a,typeof c);';
  const expected = ['PASS undefined'];
  run(code, expected);
});

test('issue_2846', () => {
  const code = 'function f(a,b){var a=0;b&&b(a);return a++}var c=f();console.log(c);';
  const expected = ['0'];
  run(code, expected);
});

test('issue_805_1', () => {
  const code =
    '(function(a){var unused=function(){};unused.prototype[a()]=42;(unused.prototype.bar=function(){console.log("bar")})();return unused})((function(){console.log("foo");return"foo"}));';
  const expected = ['foo', 'bar'];
  run(code, expected);
});

test('issue_805_2', () => {
  const code =
    '(function(a){function unused(){}unused.prototype[a()]=42;(unused.prototype.bar=function(){console.log("bar")})();return unused})((function(){console.log("foo");return"foo"}));';
  const expected = ['foo', 'bar'];
  run(code, expected);
});

test('issue_2995', () => {
  const code = 'function f(a){var b;a.b=b=function(){};b.c="PASS"}var o={};f(o);console.log(o.b.c);';
  const expected = ['PASS'];
  run(code, expected);
});

test.skip('issue_3146_1', () => {
  const code = '(function(f){f("g()")})((function(a){eval(a);function g(b){if(!b)b="PASS";console.log(b)}}));';
  const expected = ['PASS'];
  run(code, expected);
});

test.skip('issue_3146_2', () => {
  const code = '(function(f){f("g()")})((function(a){eval(a);function g(b){if(!b)b="PASS";console.log(b)}}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3146_3', () => {
  const code = 'var g="PASS";(function(f){var g="FAIL";f("console.log(g)",g[g])})((function(a){eval(a)}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3146_4', () => {
  const code = 'var g="PASS";(function(f){var g="FAIL";f("console.log(g)",g[g])})((function(a){eval(a)}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3192', () => {
  const code =
    '(function(a){console.log(a="foo",arguments[0])})("bar");(function(a){"use strict";console.log(a="foo",arguments[0])})("bar");';
  const expected = ['foo foo', 'foo bar'];
  run(code, expected);
});

test('issue_t161_top_retain_1', () => {
  const code = 'function f(){return 2}function g(){return 3}console.log(f(),g());';
  const expected = ['2 3'];
  run(code, expected);
});

test('issue_t161_top_retain_2', () => {
  const code = 'function f(){return 2}function g(){return 3}console.log(f(),f(),g(),g());';
  const expected = ['2 2 3 3'];
  run(code, expected);
});

test('issue_t161_top_retain_3', () => {
  const code = 'function f(){return 2}function g(){return 3}console.log(f(),g());';
  const expected = ['2 3'];
  run(code, expected);
});

test('issue_t161_top_retain_4', () => {
  const code = 'function f(){return 2}function g(){return 3}console.log(f(),f(),g(),g());';
  const expected = ['2 2 3 3'];
  run(code, expected);
});

test('issue_t161_top_retain_5', () => {
  const code = '(function(){function f(){return 2}function g(){return 3}console.log(f(),g())})();';
  const expected = ['2 3'];
  run(code, expected);
});

test('issue_t161_top_retain_6', () => {
  const code = '(function(){function f(){return 2}function g(){return 3}console.log(f(),f(),g(),g())})();';
  const expected = ['2 2 3 3'];
  run(code, expected);
});

test('issue_t161_top_retain_7', () => {
  const code = 'var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z);';
  const expected = ['2 3 4 6 8 12'];
  run(code, expected);
});

test('issue_t161_top_retain_8', () => {
  const code =
    'function f(){return x}function g(){return y}function h(){return z}var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_9', () => {
  const code =
    'function f(){return x}function g(){return y}function h(){return z}var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_10', () => {
  const code =
    'function f(){return x}function g(){return y}function h(){return z}var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_11', () => {
  const code =
    'function f(){return x}function g(){return y}function h(){return z}var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_12', () => {
  const code =
    'function f(){return x}function g(){return y}function h(){return z}var x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_13', () => {
  const code =
    'const f=()=>x;const g=()=>y;const h=()=>z;const x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h());';
  const expected = ['2 3 4 6 8 12 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_14', () => {
  const code =
    'class Alpha{num(){return x}}class Beta{num(){return y}}class Carrot{num(){return z}}function f(){return x}const g=()=>y;const h=()=>z;let x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h(),(new Alpha).num(),(new Beta).num(),(new Carrot).num());';
  const expected = ['2 3 4 6 8 12 2 3 4 2 3 4'];
  run(code, expected);
});

test('issue_t161_top_retain_15', () => {
  const code =
    'class Alpha{num(){return x}}class Beta{num(){return y}}class Carrot{num(){return z}}function f(){return x}const g=()=>y;const h=()=>z;let x=2,y=3,z=4;console.log(x,y,z,x*y,x*z,y*z,f(),g(),h(),(new Alpha).num(),(new Beta).num(),(new Carrot).num());';
  const expected = ['2 3 4 6 8 12 2 3 4 2 3 4'];
  run(code, expected);
});

test('issue_t183', () => {
  const code = 'function foo(val){function bar(x){if(x)return x;bar(x-1)}return bar(val)}console.log(foo("PASS"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_seq_elements', () => {
  const code = 'var a=0,b=0;console.log("just-make-sure-it-is-compilable")&&(a++,b++);';
  const expected = ['just-make-sure-it-is-compilable'];
  run(code, expected);
});

test('unused_class_with_static_props_side_effects', () => {
  const code = 'let x="FAIL";class X{static _=x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_with_static_props_side_effects_2', () => {
  const code = 'let x="FAIL";function impure(){x="PASS"}class Unused{static _=impure()}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_which_extends_might_throw', () => {
  const code = 'let x="FAIL";try{class X extends(might_throw_lol()){constructor(){}}}catch(e){x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_which_might_throw', () => {
  const code = 'let x="FAIL";try{class X{static _=ima_throw_lol()}}catch(e){x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_which_might_throw_2', () => {
  const code = 'let x="FAIL";try{class X{[ima_throw_lol()]=null}}catch(e){x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_which_might_throw_3', () => {
  const code = 'let x="FAIL";try{class X{[ima_throw_lol()](){return null}}}catch(e){x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unused_class_which_might_throw_4', () => {
  const code = 'let x="FAIL";try{class X{get[ima_throw_lol()](){return null}}}catch(e){x="PASS"}console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('variable_refs_outside_unused_class', () => {
  const code =
    'var symbols=id({prop:"method"});var input=id({prop:class{}});var staticProp=id({prop:"foo"});class unused extends input.prop{static prop=staticProp.prop;[symbols.prop](){console.log("PASS")}}console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('pow_sequence_with_parens', () => {
  const code = 'var one=1;var two=2;var four=4;console.log((four**one)**two,(four**one)**(one/two));';
  const expected = ['16 2'];
  run(code, expected);
});

test('pow_sequence_with_parens_evaluated', () => {
  const code = 'var one=1;var two=2;var four=4;console.log((four**one)**two,(four**one)**(one/two));';
  const expected = ['16 2'];
  run(code, expected);
});

test('pow_sequence_with_constants_and_parens', () => {
  const code = 'console.log((4**1)**2,(4**1)**(1/2));';
  const expected = ['16 2'];
  run(code, expected);
});

test('pow_sequence_with_parens_exact', () => {
  const code =
    'console.log((4**1)**2,(4**1)**(1/2));var one=1;var two=2;var four=4;console.log((four**one)**two,(four**one)**(one/two));';
  const expected = ['16 2', '16 2'];
  run(code, expected);
});

test('issue_1649', () => {
  const code = 'console.log(-1+-1);';
  const expected = ['-2'];
  run(code, expected);
});

test('issue_1760_1', () => {
  const code = '!function(a){try{throw 0}catch(NaN){a=+"foo"}console.log(a)}();';
  const expected = ['NaN'];
  run(code, expected);
});

test('issue_1760_2', () => {
  const code = '!function(a){try{throw 0}catch(Infinity){a=123456789/0}console.log(a)}();';
  const expected = ['Infinity'];
  run(code, expected);
});

test('issue_1964_1', () => {
  const code =
    'function f(){var long_variable_name=/\\s/;console.log(long_variable_name.source);return"a b c".split(long_variable_name)[1]}console.log(f());';
  const expected = ['\\s', 'b'];
  run(code, expected);
});

test('issue_1964_2', () => {
  const code =
    'function f(){var long_variable_name=/\\s/;console.log(long_variable_name.source);return"a b c".split(long_variable_name)[1]}console.log(f());';
  const expected = ['\\s', 'b'];
  run(code, expected);
});

test('array_slice_index', () => {
  const code = 'console.log([1,2,3].slice(1)[1]);';
  const expected = ['3'];
  run(code, expected);
});

test('string_charCodeAt', () => {
  const code = 'console.log("foo".charCodeAt("bar".length));';
  const expected = ['NaN'];
  run(code, expected);
});

test('issue_2231_3', () => {
  const code = 'console.log(Object.keys({foo:"bar"})[0]);';
  const expected = ['foo'];
  run(code, expected);
});

test('self_comparison_1', () => {
  const code = 'var o={n:NaN};console.log(typeof o.n,o.n==o.n,o.n===o.n,o.n!=o.n,o.n!==o.n);';
  const expected = ['number false false true true'];
  run(code, expected);
});

test('self_comparison_2', () => {
  const code = 'var o={n:NaN};console.log(typeof o.n,o.n==o.n,o.n===o.n,o.n!=o.n,o.n!==o.n);';
  const expected = ['number false false true true'];
  run(code, expected);
});

test('issue_2822', () => {
  const code = 'console.log([function(){},"PASS","FAIL"][1]);';
  const expected = ['PASS'];
  run(code, expected);
});

test('string_case', () => {
  const code =
    'console.log("İ".toLowerCase().charCodeAt(0));console.log("I".toLowerCase().charCodeAt(0));console.log("Ş".toLowerCase().charCodeAt(0));console.log("Ğ".toLowerCase().charCodeAt(0));console.log("Ü".toLowerCase().charCodeAt(0));console.log("Ö".toLowerCase().charCodeAt(0));console.log("Ç".toLowerCase().charCodeAt(0));console.log("i".toUpperCase().charCodeAt(0));console.log("ı".toUpperCase().charCodeAt(0));console.log("ş".toUpperCase().charCodeAt(0));console.log("ğ".toUpperCase().charCodeAt(0));console.log("ü".toUpperCase().charCodeAt(0));console.log("ö".toUpperCase().charCodeAt(0));console.log("ç".toUpperCase().charCodeAt(0));';
  const expected = ['105', '105', '351', '287', '252', '246', '231', '73', '73', '350', '286', '220', '214', '199'];
  run(code, expected);
});

test('issue_2916_1', () => {
  const code = 'var c="PASS";(function(a,b){(function(d){d[0]=1})(b);a==b&&(c="FAIL")})("",[]);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2916_2', () => {
  const code = 'var c="FAIL";(function(b){(function(d){d[0]=1})(b);+b&&(c="PASS")})([]);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2926_1', () => {
  const code = '(function f(a){console.log(f.name.length,f.length)})();';
  const expected = ['1 1'];
  run(code, expected);
});

test('issue_2926_2', () => {
  const code = 'console.log(typeof function(){}.valueOf());';
  const expected = ['function'];
  run(code, expected);
});

test('optional_expect_when_expect_stdout_present', () => {
  const code = 'console.log(5%3);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2968', () => {
  const code =
    'var c="FAIL";(function(){(function(a,b){a<<=0;a&&(a[(c="PASS",0>>>(b+=1))]=0)})(42,-42)})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('avoid_spread_in_ternary', () => {
  const code =
    'function print(...x){console.log(...x)}var a=[1,2],b=[3,4],m=Math;if(m)print(a);else print(b);if(m)print(...a);else print(b);if(m.no_such_property)print(a);else print(...b);';
  const expected = ['[1,2]', '1 2', '3 4'];
  run(code, expected);
});

test('function_returning_constant_literal', () => {
  const code = 'function greeter(){return{message:"Hello there"}}var greeting=greeter();console.log(greeting.message);';
  const expected = ['Hello there'];
  run(code, expected);
});

test.skip('hoist_funs', () => {
  const code =
    'console.log(typeof f,typeof g,1);if(console.log(typeof f,typeof g,2))console.log(typeof f,typeof g,3);else{console.log(typeof f,typeof g,4);function f(){}console.log(typeof f,typeof g,5)}function g(){}console.log(typeof f,typeof g,6);';
  const expected = [
    'undefined function 1',
    'undefined function 2',
    'function function 4',
    'function function 5',
    'function function 6',
  ];
  run(code, expected);
});

test('hoist_funs_strict', () => {
  const code =
    '"use strict";console.log(typeof f,typeof g,1);if(console.log(typeof f,typeof g,2))console.log(typeof f,typeof g,3);else{console.log(typeof f,typeof g,4);function f(){}console.log(typeof f,typeof g,5)}function g(){}console.log(typeof f,typeof g,6);';
  const expected = [
    'undefined function 1',
    'undefined function 2',
    'function function 4',
    'function function 5',
    'undefined function 6',
  ];
  run(code, expected);
});

test('issue_203', () => {
  const code =
    'var m={};var fn=Function("require","module","exports","module.exports = 42;");fn(null,m,m.exports);console.log(m.exports);';
  const expected = ['42'];
  run(code, expected);
});

test('no_webkit', () => {
  const code = 'console.log(function(){1+1}.a=1);';
  const expected = ['1'];
  run(code, expected);
});

test('webkit', () => {
  const code = 'console.log(function(){1+1}.a=1);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2084', () => {
  const code =
    'var c=0;!function(){!function(c){c=1+c;var c=0;function f14(a_1){if(c=1+c,0!==23..toString())c=1+c,a_1&&(a_1[0]=0)}f14()}(-1)}();console.log(c);';
  const expected = ['0'];
  run(code, expected);
});

test('issue_2097', () => {
  const code = 'function f(){try{throw 0}catch(e){console.log(arguments[0])}}f(1);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2101', () => {
  const code = 'a={};console.log(function(){return function(){return this.a}()}()===function(){return a}());';
  const expected = ['true'];
  run(code, expected);
});

test('inner_ref', () => {
  const code =
    'console.log(function(a){return function(){return a+1}()}(1),function(a){return function(a){return a===undefined}()}(2));';
  const expected = ['2 true'];
  run(code, expected);
});

test('issue_2107', () => {
  const code =
    'var c=0;!function(){c++}(c+++new function(){this.a=0;var a=(c=c+1)+(c=1+c);return c+++a});console.log(c);';
  const expected = ['5'];
  run(code, expected);
});

test('issue_2114_1', () => {
  const code =
    'var c=0;!function(a){a=0}([{0:c=c+1,length:c=1+c},typeof void function a(){var b=function f1(a){}(b&&(b.b+=(c=c+1,0)))}()]);console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2114_2', () => {
  const code =
    'var c=0;!function(a){a=0}([{0:c=c+1,length:c=1+c},typeof void function a(){var b=function f1(a){}(b&&(b.b+=(c=c+1,0)))}()]);console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2428', () => {
  const code =
    'function bar(k){console.log(k)}function foo(x){return bar(x)}function baz(a){foo(a)}baz(42);baz("PASS");';
  const expected = ['42', 'PASS'];
  run(code, expected);
});

test('issue_2531_1', () => {
  const code =
    'function outer(){function inner(value){function closure(){return value}return function(){return closure()}}return inner("Hello")}console.log("Greeting:",outer()());';
  const expected = ['Greeting: Hello'];
  run(code, expected);
});

test('issue_2531_2', () => {
  const code =
    'function outer(){function inner(value){function closure(){return value}return function(){return closure()}}return inner("Hello")}console.log("Greeting:",outer()());';
  const expected = ['Greeting: Hello'];
  run(code, expected);
});

test('issue_2531_3', () => {
  const code =
    'function outer(){function inner(value){function closure(){return value}return function(){return closure()}}return inner("Hello")}console.log("Greeting:",outer()());';
  const expected = ['Greeting: Hello'];
  run(code, expected);
});

test('issue_2476', () => {
  const code =
    'function foo(x,y,z){return x<y?x*y+z:x*z-y}for(var sum=0,i=0;i<10;i++)sum+=foo(i,i+1,3*i);console.log(sum);';
  const expected = ['465'];
  run(code, expected);
});

test('issue_2601_1', () => {
  const code =
    'var a="FAIL";(function(){function f(b){function g(b){b&&b()}g();(function(){b&&(a="PASS")})()}f("foo")})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2601_2', () => {
  const code =
    'var a="FAIL";(function(){function f(b){function g(b){b&&b()}g();(function(){b&&(a="PASS")})()}f("foo")})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2604_1', () => {
  const code =
    'var a="FAIL";(function(){try{throw 1}catch(b){(function f(b){b&&b()})();b&&(a="PASS")}})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2604_2', () => {
  const code =
    'var a="FAIL";(function(){try{throw 1}catch(b){(function f(b){b&&b()})();b&&(a="PASS")}})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('unsafe_apply_expansion_1', () => {
  const code = 'console.log.apply(console,[1,...[2,3],4]);';
  const expected = ['1 2 3 4'];
  run(code, expected);
});

test('unsafe_apply_expansion_2', () => {
  const code = 'var values=[2,3];console.log.apply(console,[1,...values,4]);';
  const expected = ['1 2 3 4'];
  run(code, expected);
});

test('unsafe_call_3', () => {
  const code = 'console.log(function(){return arguments[0]+eval("arguments")[1]}.call(0,1,2));';
  const expected = ['3'];
  run(code, expected);
});

test('unsafe_call_expansion_1', () => {
  const code = '(function(...a){console.log(...a)}).call(console,1,...[2,3],4);';
  const expected = ['1 2 3 4'];
  run(code, expected);
});

test('unsafe_call_expansion_2', () => {
  const code = 'var values=[2,3];(function(...a){console.log(...a)}).call(console,1,...values,4);';
  const expected = ['1 2 3 4'];
  run(code, expected);
});

test('issue_2616', () => {
  const code =
    'var c="FAIL";(function(){function f(){function g(NaN){(true<<NaN)-0/0||(c="PASS")}g([])}f()})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2620_1', () => {
  const code =
    'var c="FAIL";(function(){function f(a){var b=function g(a){a&&a()}();if(a){var d=c="PASS"}}f(1)})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2620_2', () => {
  const code =
    'var c="FAIL";(function(){function f(a){var b=function g(a){a&&a()}();if(a){var d=c="PASS"}}f(1)})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2620_3', () => {
  const code =
    'var c="FAIL";(function(){function f(a,NaN){function g(){switch(a){case a:break;case c="PASS",NaN:break}}g()}f(0/0)})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2620_4', () => {
  const code =
    'var c="FAIL";(function(){function f(a,NaN){function g(){switch(a){case a:break;case c="PASS",NaN:break}}g()}f(0/0)})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2630_1', () => {
  const code = 'var c=0;(function(){while(f());function f(){var a=function(){var b=c++,d=c=1+c}()}})();console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2630_2', () => {
  const code = 'var c=0;!function(){while(f()){}function f(){var not_used=function(){c=1+c}(c=c+1)}}();console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2630_3', () => {
  const code =
    'var x=2,a=1;(function(){function f1(a){f2();--x>=0&&f1({})}f1(a++);function f2(){a++}})();console.log(a);';
  const expected = ['5'];
  run(code, expected);
});

test('issue_2630_4', () => {
  const code =
    'var x=3,a=1,b=2;(function(){(function f1(){while(--x>=0&&f2());})();function f2(){a+++(b+=a)}})();console.log(a);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_2630_5', () => {
  const code =
    'var c=1;!function(){do{c*=10}while(f());function f(){return function(){return(c=2+c)<100}(c=c+3)}}();console.log(c);';
  const expected = ['155'];
  run(code, expected);
});

test('issue_2647_1', () => {
  const code =
    '(function(n,o="FAIL"){console.log(n)})("PASS");(function(n,o="PASS"){console.log(o)})("FAIL");(function(o="PASS"){console.log(o)})();(function(n,{o:o="FAIL"}){console.log(n)})("PASS",{});';
  const expected = ['PASS', 'PASS', 'PASS', 'PASS'];
  run(code, expected);
});

test('issue_2647_2', () => {
  const code = '(function(){function foo(x){return x.toUpperCase()}console.log((()=>foo("pass"))())})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2647_3', () => {
  const code = '(function(){function foo(x){return x.toUpperCase()}console.log((()=>foo("pass"))())})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('recursive_inline_2', () => {
  const code = 'function f(n){return n?n*f(n-1):1}console.log(f(5));';
  const expected = ['120'];
  run(code, expected);
});

test('issue_2657', () => {
  const code =
    '"use strict";console.log(function f(){return h;function g(b){return b||b()}function h(a){g(a);return a}}()(42));';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2663_1', () => {
  const code =
    '(function(){var i,o={};function createFn(j){return function(){console.log(j)}}for(i in{a:1,b:2,c:3})o[i]=createFn(i);for(i in o)o[i]()})();';
  const expected = ['a', 'b', 'c'];
  run(code, expected);
});

test('issue_2663_2', () => {
  const code = '(function(){var i;function fn(j){return function(){console.log(j)}()}for(i in{a:1,b:2,c:3})fn(i)})();';
  const expected = ['a', 'b', 'c'];
  run(code, expected);
});

test('issue_2663_3', () => {
  const code =
    '(function(){var outputs=[{type:0,target:null,eventName:"ngSubmit",propName:null},{type:0,target:null,eventName:"submit",propName:null},{type:0,target:null,eventName:"reset",propName:null}];function listenToElementOutputs(outputs){var handlers=[];for(var i=0;i<outputs.length;i++){var output=outputs[i];var handleEventClosure=renderEventHandlerClosure(output.eventName);handlers.push(handleEventClosure)}var target,name;return handlers}function renderEventHandlerClosure(eventName){return function(){return console.log(eventName)}}listenToElementOutputs(outputs).forEach((function(handler){return handler()}))})();';
  const expected = ['ngSubmit', 'submit', 'reset'];
  run(code, expected);
});

test('duplicate_argnames', () => {
  const code = 'var a="PASS";function f(b,b,b){b&&(a="FAIL")}f(0,console);console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('loop_init_arg', () => {
  const code = 'var a="PASS";for(var k in"12")(function(b){(b>>=1)&&(a="FAIL"),b=2})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_false', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('inline_0', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('inline_1', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('inline_2', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('inline_3', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('inline_true', () => {
  const code =
    '(function(){console.log(1)})();(function(a){console.log(a)})(2);(function(b){var c=b;console.log(c)})(3);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('issue_2842', () => {
  const code =
    '(function(){function inlinedFunction(data){return data[data[0]]}function testMinify(){if(true){const data=inlinedFunction([1,2,3]);console.log(data)}}return testMinify()})();';
  const expected = ['2'];
  run(code, expected);
});

test('use_before_init_in_loop', () => {
  const code =
    'var a="PASS";for(var b=2;--b>=0;)(function(){var c=function(){return 1}(c&&(a="FAIL"))})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('duplicate_arg_var', () => {
  const code = 'console.log(function(b){return b+"ING";var b}("PASS"));';
  const expected = ['PASSING'];
  run(code, expected);
});

test('issue_2737_1', () => {
  const code = '(function(a){while(a());})((function f(){console.log(typeof f)}));';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2737_2', () => {
  const code = '(function(bar){for(;bar();)break})((function qux(){return console.log("PASS"),qux}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2783', () => {
  const code =
    '(function(){return g;function f(a){var b=a.b;if(b)return b;return a}function g(o,i){while(i--){console.log(f(o))}}})()({b:"PASS"},1);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2898', () => {
  const code = 'var c=0;(function(){while(f());function f(){var b=(c=1+c,void(c=1+c));b&&b[0]}})();console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_3016_1', () => {
  const code = 'var b=1;do{(function(a){return a[b];var a})(3)}while(0);console.log(b);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3016_2', () => {
  const code = 'var b=1;do{(function(a){return a[b];try{a=2}catch(a){var a}})(3)}while(0);console.log(b);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3016_2_ie8', () => {
  const code = 'var b=1;do{(function(a){return a[b];try{a=2}catch(a){var a}})(3)}while(0);console.log(b);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3016_3', () => {
  const code = 'var b=1;do{console.log(function(){return a?"FAIL":a="PASS";try{a=2}catch(a){var a}}())}while(b--);';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('issue_3016_3_ie8', () => {
  const code = 'var b=1;do{console.log(function(){return a?"FAIL":a="PASS";try{a=2}catch(a){var a}}())}while(b--);';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('issue_3018', () => {
  const code = 'var b=1,c="PASS";do{(function(){(function(a){a=0!=(a&&(c="FAIL"))})()})()}while(b--);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3054', () => {
  const code = '"use strict";function f(){return{a:true}}console.log(function(b){b=false;return f()}().a,f.call().a);';
  const expected = ['true true'];
  run(code, expected);
});

test('issue_3076', () => {
  const code =
    'var c="PASS";(function(b){var n=2;while(--b+function(){e&&(c="FAIL");e=5;return 1;try{var a=5}catch(e){var e}}().toString()&&--n>0);})(2);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3125', () => {
  const code = 'console.log(function(){return"PASS"}.call());';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_t131a', () => {
  const code =
    '(function(){function thing(){return{a:1}}function one(){return thing()}function two(){var x=thing();x.a=2;x.b=3;return x}console.log(JSON.stringify(one()),JSON.stringify(two()))})();';
  const expected = ['{"a":1} {"a":2,"b":3}'];
  run(code, expected);
});

test('issue_t131b', () => {
  const code =
    '(function(){function thing(){return{a:1}}function one(){return thing()}function two(){var x=thing();x.a=2;x.b=3;return x}console.log(JSON.stringify(one()),JSON.stringify(two()))})();';
  const expected = ['{"a":1} {"a":2,"b":3}'];
  run(code, expected);
});

test('avoid_generating_duplicate_functions_compared_together', () => {
  const code = 'const x=()=>null;const y=()=>x;console.log(y()===y());';
  const expected = ['true'];
  run(code, expected);
});

test('avoid_generating_duplicate_functions_compared_together_2', () => {
  const code = 'const defaultArg=input=>input;const fn=(arg=defaultArg)=>arg;console.log(fn()===fn());';
  const expected = ['true'];
  run(code, expected);
});

test('avoid_generating_duplicate_functions_compared_together_3', () => {
  const code = 'const x=()=>null;console.log(id(x)===id(x));';
  const expected = ['true'];
  run(code, expected);
});

test('avoid_generating_duplicate_functions_compared_together_4', () => {
  const code =
    'const x=()=>null;const y=()=>x;const fns=[y(),y()];console.log(fns[0]===fns[1]);const fns_obj={a:y(),b:y()};console.log(fns_obj.a===fns_obj.b);';
  const expected = ['true', 'true'];
  run(code, expected);
});

test('typeof_arrow_functions', () => {
  const code = 'var foo=typeof(x=>null);console.log(foo);';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2028', () => {
  const code =
    'var a={};(function(x){x.X=function(){return X};class X{static hello(){console.log("hello")}}})(a);a.X().hello();';
  const expected = ['hello'];
  run(code, expected);
});

test('array_spread_of_sequence', () => {
  const code = 'var a=[1];console.log([...(a,a)]);console.log([...a,a]);console.log([...a||a]);console.log([...a||a]);';
  const expected = ['[1]', '[1,[1]]', '[1]', '[1]'];
  run(code, expected);
});

test('issue_2345', () => {
  const code = 'console.log([...[3,2,1]].join("-"));var a=[3,2,1];console.log([...a].join("-"));';
  const expected = ['3-2-1', '3-2-1'];
  run(code, expected);
});

test('issue_2349', () => {
  const code =
    'function foo(boo,key){const value=boo.get();return value.map(({[key]:bar})=>bar)}console.log(foo({get:()=>[{blah:42}]},"blah"));';
  const expected = ['[42]'];
  run(code, expected);
});

test('issue_2349b', () => {
  const code =
    'function foo(boo,key){const value=boo.get();return value.map((function({[key]:bar}){return bar}))}console.log(foo({get:function(){return[{blah:42}]}},"blah"));';
  const expected = ['[42]'];
  run(code, expected);
});

test('array_literal_with_spread_1', () => {
  const code = 'var f=x=>[...x][0];console.log(f(["PASS"]));';
  const expected = ['PASS'];
  run(code, expected);
});

test('array_literal_with_spread_2a', () => {
  const code =
    'console.log([10,...[],20,...[30,40],50]["length"]);console.log([10,...[],20,...[30,40],50][0]);console.log([10,...[],20,...[30,40],50][1]);console.log([10,...[],20,...[30,40],50][2]);console.log([10,...[],20,...[30,40],50][3]);console.log([10,...[],20,...[30,40],50][4]);console.log([10,...[],20,...[30,40],50][5]);';
  const expected = ['5', '10', '20', '30', '40', '50', 'undefined'];
  run(code, expected);
});

test('array_literal_with_spread_2b', () => {
  const code =
    'var x=[30,40];console.log([10,...[],20,...x,50]["length"]);console.log([10,...[],20,...x,50][0]);console.log([10,...[],20,...x,50][1]);console.log([10,...[],20,...x,50][2]);console.log([10,...[],20,...x,50][3]);console.log([10,...[],20,...x,50][4]);console.log([10,...[],20,...x,50][5]);';
  const expected = ['5', '10', '20', '30', '40', '50', 'undefined'];
  run(code, expected);
});

test('array_literal_with_spread_3a', () => {
  const code =
    'console.log([10,20][0]);console.log([10,20][1]);console.log([10,20][2]);console.log([...[],10,20][0]);console.log([...[],10,20][1]);console.log([...[],10,20][2]);console.log([10,...[],20][0]);console.log([10,...[],20][1]);console.log([10,...[],20][2]);console.log([10,20,...[]][0]);console.log([10,20,...[]][1]);console.log([10,20,...[]][2]);';
  const expected = ['10', '20', 'undefined', '10', '20', 'undefined', '10', '20', 'undefined', '10', '20', 'undefined'];
  run(code, expected);
});

test('array_literal_with_spread_3b', () => {
  const code =
    'var nothing=[];console.log([10,20][0]);console.log([10,20][1]);console.log([10,20][2]);console.log([...nothing,10,20][0]);console.log([...nothing,10,20][1]);console.log([...nothing,10,20][2]);console.log([10,...nothing,20][0]);console.log([10,...nothing,20][1]);console.log([10,...nothing,20][2]);console.log([10,20,...nothing][0]);console.log([10,20,...nothing][1]);console.log([10,20,...nothing][2]);';
  const expected = ['10', '20', 'undefined', '10', '20', 'undefined', '10', '20', 'undefined', '10', '20', 'undefined'];
  run(code, expected);
});

test('array_literal_with_spread_4a', () => {
  const code =
    'function t(x){console.log("("+x+")");return 10*x}console.log([t(1),t(2)][0]);console.log([t(1),t(2)][1]);console.log([t(1),t(2)][2]);console.log([...[],t(1),t(2)][0]);console.log([...[],t(1),t(2)][1]);console.log([...[],t(1),t(2)][2]);console.log([t(1),...[],t(2)][0]);console.log([t(1),...[],t(2)][1]);console.log([t(1),...[],t(2)][2]);console.log([t(1),t(2),...[]][0]);console.log([t(1),t(2),...[]][1]);console.log([t(1),t(2),...[]][2]);';
  const expected = [
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
  ];
  run(code, expected);
});

test('array_literal_with_spread_4b', () => {
  const code =
    'var nothing=[];function t(x){console.log("("+x+")");return 10*x}console.log([t(1),t(2)][0]);console.log([t(1),t(2)][1]);console.log([t(1),t(2)][2]);console.log([...nothing,t(1),t(2)][0]);console.log([...nothing,t(1),t(2)][1]);console.log([...nothing,t(1),t(2)][2]);console.log([t(1),...nothing,t(2)][0]);console.log([t(1),...nothing,t(2)][1]);console.log([t(1),...nothing,t(2)][2]);console.log([t(1),t(2),...nothing][0]);console.log([t(1),t(2),...nothing][1]);console.log([t(1),t(2),...nothing][2]);';
  const expected = [
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
    '(1)',
    '(2)',
    '10',
    '(1)',
    '(2)',
    '20',
    '(1)',
    '(2)',
    'undefined',
  ];
  run(code, expected);
});

test('object_literal_method_using_arguments', () => {
  const code = 'console.log({m(){return arguments[0]}}.m("PASS"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('class_method_using_arguments', () => {
  const code = 'console.log((new class{m(){return arguments[0]}}).m("PASS"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2762', () => {
  const code =
    'var bar=1,T=true;(function(){if(T){const a=function(){var foo=bar;console.log(foo,a.prop,b.prop)};a.prop=2;const b={prop:3};a()}})();';
  const expected = ['1 2 3'];
  run(code, expected);
});

test('issue_2794_1', () => {
  const code =
    'function foo(){for(const a of func(value)){console.log(a)}function func(va){return doSomething(va)}}function doSomething(x){return[x,2*x,3*x]}const value=10;foo();';
  const expected = ['10', '20', '30'];
  run(code, expected);
});

test('issue_2794_2', () => {
  const code =
    'function foo(){for(const a of func(value)){console.log(a)}function func(va){return doSomething(va)}}function doSomething(x){return[x,2*x,3*x]}const value=10;foo();';
  const expected = ['10', '20', '30'];
  run(code, expected);
});

test('issue_2794_3', () => {
  const code =
    'function foo(){for(const a of func(value)){console.log(a)}function func(va){return doSomething(va)}}function doSomething(x){return[x,2*x,3*x]}const value=10;foo();';
  const expected = ['10', '20', '30'];
  run(code, expected);
});

test('issue_2794_4', () => {
  const code = 'for(var x of([1,2],[3,4])){console.log(x)}';
  const expected = ['3', '4'];
  run(code, expected);
});

test('issue_2794_5', () => {
  const code = 'for(var x of([1,2],[3,4])){console.log(x)}';
  const expected = ['3', '4'];
  run(code, expected);
});

test('issue_2794_6', () => {
  const code = 'for(let e of([1,2],[3,4,5])){console.log(e)}';
  const expected = ['3', '4', '5'];
  run(code, expected);
});

test('inline_arrow_using_arguments', () => {
  const code = '(function(){(x=>{console.log.apply(console,arguments),console.log(x)})(4)})(3,2,1);';
  const expected = ['3 2 1', '4'];
  run(code, expected);
});

test('issue_2874_1', () => {
  const code =
    '(function(){function foo(){let letters=["A","B","C"];let result=[2,1,0].map(key=>bar(letters[key]+key));return result}function bar(value){return()=>console.log(value)}foo().map(fn=>fn())})();';
  const expected = ['C2', 'B1', 'A0'];
  run(code, expected);
});

test('issue_2874_2', () => {
  const code =
    '(function(){let keys=[];function foo(){var result=[2,1,0].map(value=>{keys.push(value);return bar()});return result}function bar(){var letters=["A","B","C"],key=keys.shift();return()=>console.log(letters[key]+key)}foo().map(fn=>fn())})();';
  const expected = ['C2', 'B1', 'A0'];
  run(code, expected);
});

test('issue_2874_3', () => {
  const code = 'function f(){return x+y}let x,y;let a=z=>{x="A";y=z;console.log(f())};a(1);a(2);';
  const expected = ['A1', 'A2'];
  run(code, expected);
});

test('issue_3061', () => {
  const code = 'console.log(new class extends(function(base){return class extends base{}}(Error)){}instanceof Error);';
  const expected = ['true'];
  run(code, expected);
});

test('issue_3028', () => {
  const code =
    'function a(array){var shifted=array.splice(0,2);return[...array,...shifted]}console.log(a([1,2,3]).join(" "));';
  const expected = ['3 1 2'];
  run(code, expected);
});

test('issue_t80', () => {
  const code =
    'function foo(data=[]){var u,v="unused";if(arguments.length==1){data=[data]}return data}console.log(JSON.stringify([foo(),foo(null),foo(5,6)]));';
  const expected = ['[[],[null],5]'];
  run(code, expected);
});

test('issue_2377_1', () => {
  const code =
    'var obj={foo:1,bar:2,square:function(x){return x*x},cube:function(x){return x*x*x}};console.log(obj.foo,obj.cube(3));';
  const expected = ['1 27'];
  run(code, expected);
});

test('issue_2377_2', () => {
  const code =
    'var obj={foo:1,bar:2,square:function(x){return x*x},cube:function(x){return x*x*x}};console.log(obj.foo,obj.cube(3));';
  const expected = ['1 27'];
  run(code, expected);
});

test('issue_2377_3', () => {
  const code =
    'var obj={foo:1,bar:2,square:function(x){return x*x},cube:function(x){return x*x*x}};console.log(obj.foo,obj.cube(3));';
  const expected = ['1 27'];
  run(code, expected);
});

test('direct_access_1', () => {
  const code = 'var a=0;var obj={a:1,b:2};for(var k in obj)a++;console.log(a,obj.a);';
  const expected = ['2 1'];
  run(code, expected);
});

test('direct_access_2', () => {
  const code = 'var o={a:1};var f=function(k){if(o[k])return"PASS"};console.log(f("a"));';
  const expected = ['PASS'];
  run(code, expected);
});

test('direct_access_3', () => {
  const code = 'var o={a:1};o.b;console.log(o.a);';
  const expected = ['1'];
  run(code, expected);
});

test('name_collision_1', () => {
  const code =
    'var obj_foo=1;var obj_bar=2;function f(){var obj={foo:3,bar:4,"b-r":5,"b+r":6,"b!r":7};console.log(obj_foo,obj.foo,obj.bar,obj["b-r"],obj["b+r"],obj["b!r"])}f();';
  const expected = ['1 3 4 5 6 7'];
  run(code, expected);
});

test('name_collision_2', () => {
  const code =
    'var o={p:1,"+":function(x){return x},"-":function(x){return x+1}},o__$0=2,o__$1=3;console.log(o.p===o.p,o["+"](4),o["-"](5),o__$0,o__$1);';
  const expected = ['true 4 6 2 3'];
  run(code, expected);
});

test('name_collision_3', () => {
  const code =
    'var o={p:1,"+":function(x){return x},"-":function(x){return x+1}},o__$0=2,o__$1=3;console.log(o.p===o.p,o["+"](4),o["-"](5));';
  const expected = ['true 4 6'];
  run(code, expected);
});

test('contains_this_1', () => {
  const code = 'var o={u:function(){return this===this},p:1};console.log(o.p,o.p);';
  const expected = ['1 1'];
  run(code, expected);
});

test('contains_this_3', () => {
  const code = 'var o={u:function(){return this===this},p:1};console.log(o.p,o.p,o.u());';
  const expected = ['1 1 true'];
  run(code, expected);
});

test.skip('hoist_class', () => {
  const code =
    'function run(c,v){return new c(v).value}var o={p:class Foo{constructor(value){this.value=value*10}},x:1,y:2};console.log(o.p.name,o.p===o.p,run(o.p,o.x),run(o.p,o.y));';
  const expected = ['Foo true 10 20'];
  run(code, expected);
});

test.skip('hoist_class_with_new', () => {
  const code =
    'var o={p:class Foo{constructor(value){this.value=value*10}},x:1,y:2};console.log(o.p.name,o.p===o.p,new o.p(o.x).value,new o.p(o.y).value);';
  const expected = ['Foo true 10 20'];
  run(code, expected);
});

test.skip('hoist_function_with_call', () => {
  const code =
    'var o={p:function Foo(value){return 10*value},x:1,y:2};console.log(o.p.name,o.p===o.p,o.p(o.x),o.p(o.y));';
  const expected = ['Foo true 10 20'];
  run(code, expected);
});

test('new_this', () => {
  const code = 'var o={a:1,b:2,f:function(a){this.b=a}};console.log(new o.f(o.a).b,o.b);';
  const expected = ['1 2'];
  run(code, expected);
});

test('issue_2473_3', () => {
  const code = 'var o={a:1,b:2};console.log(o.a,o.b);';
  const expected = ['1 2'];
  run(code, expected);
});

test('issue_2473_4', () => {
  const code = '(function(){var o={a:1,b:2};console.log(o.a,o.b)})();';
  const expected = ['1 2'];
  run(code, expected);
});

test('issue_2519', () => {
  const code =
    'function testFunc(){var dimensions={minX:5,maxX:6};var scale=1;var d={x:(dimensions.maxX+dimensions.minX)/2};return d.x*scale}console.log(testFunc());';
  const expected = ['5.5'];
  run(code, expected);
});

test('toplevel_const', () => {
  const code = 'const a={b:1,c:2};console.log(a.b+a.c);';
  const expected = ['3'];
  run(code, expected);
});

test('toplevel_let', () => {
  const code = 'let a={b:1,c:2};console.log(a.b+a.c);';
  const expected = ['3'];
  run(code, expected);
});

test('toplevel_var', () => {
  const code = 'var a={b:1,c:2};console.log(a.b+a.c);';
  const expected = ['3'];
  run(code, expected);
});

test('undefined_key', () => {
  const code = 'var a,o={};o[a]=1;o.b=2;console.log(o[a]+o.b);';
  const expected = ['3'];
  run(code, expected);
});

test('issue_3021', () => {
  const code = 'var a=1,b=2;(function(){b=a;if(a+++b--)return 1;return;var b={}})();console.log(a,b);';
  const expected = ['2 2'];
  run(code, expected);
});

test('issue_3046', () => {
  const code = 'console.log(function(a){do{var b={c:a++}}while(b.c&&a);return a}(0));';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3071_1', () => {
  const code = '(function(){var obj={};obj.one=1;obj.two=2;console.log(obj.one)})();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3071_2', () => {
  const code = '(function(){obj={};obj.one=1;obj.two=2;console.log(obj.one);var obj})();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3071_2_toplevel', () => {
  const code = '(function(){obj={};obj.one=1;obj.two=2;console.log(obj.one);var obj})();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3071_3', () => {
  const code = 'var c=0;(function(a,b){(function f(o){var n=2;while(--b+(o={p:c++})&&--n>0);})()})();console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('does_not_hoist_objects_with_computed_props', () => {
  const code = 'const x={[console.log("PASS")]:123};';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_identity', () => {
  const code = 'const id=x=>x;console.log(id(1),id(2));';
  const expected = ['1 2'];
  run(code, expected);
});

test('inline_identity_function', () => {
  const code = 'function id(x){return x}console.log(id(1),id(2));';
  const expected = ['1 2'];
  run(code, expected);
});

test('inline_identity_undefined', () => {
  const code = 'const id=x=>x;console.log(id(),id(undefined));';
  const expected = ['undefined undefined'];
  run(code, expected);
});

test('inline_identity_extra_params', () => {
  const code = 'const id=x=>x;console.log(id(1,console.log(2)),id(3,4));';
  const expected = ['2', '1 3'];
  run(code, expected);
});

test('inline_identity_higher_order', () => {
  const code = 'const id=x=>x;const inc=x=>x+1;console.log(id(inc(1)),id(inc)(2));';
  const expected = ['2 3'];
  run(code, expected);
});

test('inline_identity_inline_function', () => {
  const code = 'const id=x=>x;console.log(id(x=>x+1)(1),id((x=>x+1)(2)));';
  const expected = ['2 3'];
  run(code, expected);
});

test('inline_identity_duplicate_arg_var', () => {
  const code = 'const id=x=>{return x;var x};console.log(id(1),id(2));';
  const expected = ['1 2'];
  run(code, expected);
});

test('inline_identity_inner_ref', () => {
  const code = 'const id=a=>function(){return a}();const undef=a=>(a=>a)();console.log(id(1),id(2),undef(3),undef(4));';
  const expected = ['1 2 undefined undefined'];
  run(code, expected);
});

test('inline_identity_async', () => {
  const code = 'const id=x=>x;id(async()=>await 1)();id(async x=>await console.log(2))();';
  const expected = ['2'];
  run(code, expected);
});

test('inline_identity_regression', () => {
  const code = 'global.id=x=>x;const foo=({bar:bar})=>id(bar);console.log(foo({bar:"PASS"}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_identity_lose_this', () => {
  const code =
    '"use strict";const id=x=>x;const func_bag={func:function(){return this===undefined?"PASS":"FAIL"}};func_bag.func2=function(){return this===undefined?"PASS":"FAIL"};console.log(id(func_bag.func)());console.log(id(func_bag.func2)());';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('issue_2120_1', () => {
  const code = '"aaaaaaaa";var a=1,b="FAIL";try{throw 1}catch(c){try{throw 0}catch(a){if(c)b="PASS"}}console.log(b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2120_2', () => {
  const code = '"aaaaaaaa";var a=1,b="FAIL";try{throw 1}catch(c){try{throw 0}catch(a){if(c)b="PASS"}}console.log(b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2254_1', () => {
  const code = '"eeeeee";try{console.log(f("PASS"))}catch(e){}function f(s){try{throw"FAIL"}catch(e){return s}}';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2254_2', () => {
  const code = '"eeeeee";try{console.log(f("PASS"))}catch(e){}function f(s){try{throw"FAIL"}catch(e){return s}}';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1317', () => {
  const code = '!function(a){if(a)return;let b=1;function g(){return b}console.log(g())}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1317_strict', () => {
  const code = '"use strict";!function(a){if(a)return;let b=1;function g(){return b}console.log(g())}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2747', () => {
  const code =
    '"use strict";function f(baz){if(baz===0){return null}let r;if(baz>2){r=4}else{r=5}return r}console.log(f(0),f(1),f(3));';
  const expected = ['undefined 5 4'];
  run(code, expected);
});

test('inline_within_extends_1', () => {
  const code =
    '(function(){function foo(foo_base){return class extends foo_base{}}function bar(bar_base){return class extends bar_base{}}console.log((new class extends(foo(bar(Array))){}).concat(["PASS"])[0])})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_within_extends_2', () => {
  const code =
    'class Baz extends(foo(bar(Array))){constructor(){super(...arguments)}}function foo(foo_base){return class extends foo_base{constructor(){super(...arguments)}second(){return this[1]}}}function bar(bar_base){return class extends bar_base{constructor(...args){super(...args)}}}console.log(new Baz(1,"PASS",3).second());';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_into_scope_conflict', () => {
  const code =
    'var mod=pass;const c=function c(){mod()};const b=function b(){for(;;){c();break}};(function(){var mod=id(mod);b()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_into_scope_conflict_enclosed', () => {
  const code =
    'global.same_name="PASS";function $(same_name){if(same_name)indirection_1(same_name)}function indirection_2(){console.log(same_name)}function indirection_1(){indirection_2()}$("FAIL");';
  const expected = ['PASS'];
  run(code, expected);
});

test('inline_into_scope_conflict_enclosed_2', () => {
  const code =
    'global.same_name=()=>console.log("PASS");function $(same_name){console.log(same_name===undefined?"PASS":"FAIL");indirection_1()}function indirection_1(){return indirection_2()}function indirection_2(){for(const x of[1]){same_name();return}}$();';
  const expected = ['PASS', 'PASS'];
  run(code, expected);
});

test('inline_annotation_2', () => {
  const code = 'const shouldInline=n=>+n;const a=shouldInline("42.0");const b=shouldInline("abc");console.log(a,b);';
  const expected = ['42 NaN'];
  run(code, expected);
});

test('inline_func_with_name_existing_in_block_scope', () => {
  const code =
    'let something="PASS";function getSomething(){return something}function setSomething(){something={value:42}}function main(){if(typeof somethingElse=="undefined"){const something=getSomething();console.log(something)}}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('dont_inline_funcs_into_default_param', () => {
  const code =
    '"use strict";const getData=val=>({val:val});const print=function(data=getData(id("PASS"))){console.log(data.val)};print();';
  const expected = ['PASS'];
  run(code, expected);
});

test('non_hoisted_function_after_return_strict', () => {
  const code =
    '"use strict";function foo(x){if(x){return bar();not_called1()}else{return baz();not_called2()}function bar(){return 7}return not_reached;function UnusedFunction(){}function baz(){return 8}}console.log(foo(0),foo(1));';
  const expected = ['8 7'];
  run(code, expected);
});

test('non_hoisted_function_after_return_2a_strict', () => {
  const code =
    '"use strict";function foo(x){if(x){return bar(1);var a=not_called(1)}else{return bar(2);var b=not_called(2)}var c=bar(3);function bar(x){return 7-x}function nope(){}return b||c}console.log(foo(0),foo(1));';
  const expected = ['5 6'];
  run(code, expected);
});

test('non_hoisted_function_after_return_2b_strict', () => {
  const code =
    '"use strict";function foo(x){if(x){return bar(1)}else{return bar(2);var b}var c=bar(3);function bar(x){return 7-x}return b||c}console.log(foo(0),foo(1));';
  const expected = ['5 6'];
  run(code, expected);
});

test('screw_ie8', () => {
  const code = 'try{throw"foo"}catch(x){console.log(x)}';
  const expected = ['foo'];
  run(code, expected);
});

test('support_ie8', () => {
  const code = 'try{throw"foo"}catch(x){console.log(x)}';
  const expected = ['foo'];
  run(code, expected);
});

test('side_effects_catch', () => {
  const code = 'function f(){function g(){try{throw 0}catch(e){console.log("PASS")}}g()}f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('side_effects_else', () => {
  const code = 'function f(x){function g(){if(x);else console.log("PASS")}g()}f(0);';
  const expected = ['PASS'];
  run(code, expected);
});

test('side_effects_finally', () => {
  const code = 'function f(){function g(){try{x()}catch(e){}finally{console.log("PASS")}}g()}f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('side_effects_label', () => {
  const code = 'function f(x){function g(){L:{console.log("PASS");break L}}g()}f(0);';
  const expected = ['PASS'];
  run(code, expected);
});

test('side_effects_switch', () => {
  const code = 'function f(){function g(){switch(0){default:case console.log("PASS"):}}g()}f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_ie8', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_ie8', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_ie8_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_ie8_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_ie8', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_toplevel', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_ie8_toplevel', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_2', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_ie8', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_toplevel', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_ie8_toplevel', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_3', () => {
  const code = 'var o="PASS";try{throw 0}catch(o){(function(){function f(){o="FAIL"}f(),f()})()}console.log(o);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_3_toplevel', () => {
  const code = 'var o="PASS";try{throw 0}catch(o){(function(){function f(){o="FAIL"}f(),f()})()}console.log(o);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_ie8_3', () => {
  const code = 'var o="PASS";try{throw 0}catch(o){(function(){function f(){o="FAIL"}f(),f()})()}console.log(o);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_3_ie8_toplevel', () => {
  const code = 'var o="PASS";try{throw 0}catch(o){(function(){function f(){o="FAIL"}f(),f()})()}console.log(o);';
  const expected = ['PASS'];
  run(code, expected);
});

test('function_iife_catch', () => {
  const code = 'function f(n){!function(){try{throw 0}catch(n){var a=1;console.log(n,a)}}()}f();';
  const expected = ['0 1'];
  run(code, expected);
});

test('function_iife_catch_ie8', () => {
  const code = 'function f(n){!function(){try{throw 0}catch(n){var a=1;console.log(n,a)}}()}f();';
  const expected = ['0 1'];
  run(code, expected);
});

test('function_catch_catch', () => {
  const code =
    'var o=0;function f(){try{throw 1}catch(c){try{throw 2}catch(o){var o=3;console.log(o)}}console.log(o)}f();';
  const expected = ['3', 'undefined'];
  run(code, expected);
});

test('function_catch_catch_ie8', () => {
  const code =
    'var o=0;function f(){try{throw 1}catch(c){try{throw 2}catch(o){var o=3;console.log(o)}}console.log(o)}f();';
  const expected = ['3', 'undefined'];
  run(code, expected);
});

test('case_1', () => {
  const code = 'var a=0,b=1;switch(true){case a||true:default:b=2;case true:}console.log(a,b);';
  const expected = ['0 2'];
  run(code, expected);
});

test('case_2', () => {
  const code = 'var a=0,b=1;switch(0){default:b=2;case a:a=3;case 0:}console.log(a,b);';
  const expected = ['3 1'];
  run(code, expected);
});

test('mangle_props', () => {
  const code =
    'var obj={undefined:1,NaN:2,Infinity:3,"-Infinity":4,null:5};console.log(obj[void 0],obj[undefined],obj["undefined"],obj[0/0],obj[NaN],obj["NaN"],obj[1/0],obj[Infinity],obj["Infinity"],obj[-1/0],obj[-Infinity],obj["-Infinity"],obj[null],obj["null"]);';
  const expected = ['1 1 1 2 2 2 3 3 3 4 4 4 5 5'];
  run(code, expected);
});

test('numeric_literal', () => {
  const code =
    'var obj={0:0,"-0":1,42:2,42:3,37:4,o:5,1e42:6,j:7,1e42:8};console.log(obj[-0],obj[-""],obj["-0"]);console.log(obj[42],obj["42"]);console.log(obj[37],obj["o"],obj[37],obj["37"]);console.log(obj[1e42],obj["j"],obj["1e+42"]);';
  const expected = ['0 0 1', '3 3', '4 5 4 4', '8 7 8'];
  run(code, expected);
});

test('modified', () => {
  const code = 'function f5(b){var a=function(){return b}();return b+++a}console.log(f5(1));';
  const expected = ['2'];
  run(code, expected);
});

test('issue_1758', () => {
  const code = 'console.log(function(c){var undefined=42;return function(){c--;c--,c.toString();return}()}());';
  const expected = ['undefined'];
  run(code, expected);
});

test('drop_fargs', () => {
  const code = 'var a=1;!function(a_1){a++}(a+++(a&&a.var));console.log(a);';
  const expected = ['3'];
  run(code, expected);
});

test('keep_fargs', () => {
  const code = 'var a=1;!function(a_1){a++}(a+++(a&&a.var));console.log(a);';
  const expected = ['3'];
  run(code, expected);
});

test('inline_script_off', () => {
  const code = 'console.log("<\\/sCrIpT>");';
  const expected = ['</sCrIpT>'];
  run(code, expected);
});

test('inline_script_on', () => {
  const code = 'console.log("<\\/sCrIpT>");';
  const expected = ['</sCrIpT>'];
  run(code, expected);
});

test('test_unexpected_crash', () => {
  const prepend = 'x();';
  const code =
    'function x(){var getsInlined=function(){var leakedVariable1=3;var leakedVariable2=1+2*leakedVariable1;console.log(leakedVariable1);console.log(leakedVariable2)};var getsDropped=getsInlined()}';
  const expected = ['3', '7'];
  run(code, expected, prepend);
});

test('test_unexpected_crash_2', () => {
  const prepend = 'x();';
  const code =
    'function x(){var getsInlined=function(){var leakedVariable1=3;var leakedVariable2=1+leakedVariable1[0];console.log(leakedVariable1);console.log(leakedVariable2)};var getsDropped=getsInlined()}';
  const expected = ['3', 'NaN'];
  run(code, expected, prepend);
});

test('issue_1724', () => {
  const code = 'var a=0;++a%Infinity|Infinity?a++:0;console.log(a);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_1725', () => {
  const code = '([].length===0)%Infinity?console.log("PASS"):console.log("FAIL");';
  const expected = ['PASS'];
  run(code, expected);
});

test('dont_reuse_prop', () => {
  const code = '"aaaaaaaaaabbbbb";var obj={};obj.a=123;obj.asd=256;console.log(obj.a);';
  const expected = ['123'];
  run(code, expected);
});

test('unmangleable_props_should_always_be_reserved', () => {
  const code = '"aaaaaaaaaabbbbb";var obj={};obj.asd=256;obj.a=123;console.log(obj.a);';
  const expected = ['123'];
  run(code, expected);
});

test('this_binding_sequences', () => {
  const code =
    'console.log(typeof function(){return eval("this")}());console.log(typeof function(){"use strict";return eval("this")}());console.log(typeof function(){return(0,eval)("this")}());console.log(typeof function(){"use strict";return(0,eval)("this")}());';
  const expected = ['object', 'undefined', 'object', 'object'];
  run(code, expected);
});

test('issue_t120_1', () => {
  const code =
    'function foo(node){var traverse=function(obj){var i=obj.data;return i&&i.a!=i.b};while(traverse(node)){node=node.data}return node}var x={a:1,b:2,data:{a:"hello"}};console.log(foo(x).a,foo({a:"world"}).a);';
  const expected = ['hello world'];
  run(code, expected);
});

test('issue_t120_2', () => {
  const code =
    'function foo(node){var traverse=function(obj){var i=obj.data;return i&&i.a!=i.b};while(traverse(node)){node=node.data}return node}var x={a:1,b:2,data:{a:"hello"}};console.log(foo(x).a,foo({a:"world"}).a);';
  const expected = ['hello world'];
  run(code, expected);
});

test('issue_t120_3', () => {
  const code = 'for(var t=o=>{var i=+o;return console.log(i+i)&&0};t(1););';
  const expected = ['2'];
  run(code, expected);
});

test('issue_t120_4', () => {
  const code = 'for(var x=1,t=o=>{var i=+o;return console.log(i+i)&&0};x--;t(2));';
  const expected = ['4'];
  run(code, expected);
});

test('issue_t120_5', () => {
  const code = 'for(var x=1,t=o=>{var i=+o;return console.log(i+i)&&0};x--;)t(3);';
  const expected = ['6'];
  run(code, expected);
});

test('pr_152_regression', () => {
  const code =
    '(function(root,factory){root.CryptoJS=factory()})(this,(function(){var CryptoJS=CryptoJS||function(Math){var C={};C.demo=function(n){return Math.ceil(n)};return C}(Math);return CryptoJS}));var result=this.CryptoJS.demo(1.3);console.log(result);';
  const expected = ['2'];
  run(code, expected);
});

test('no_flatten_with_arg_colliding_with_arg_value_inner_scope', () => {
  const code =
    'var g=["a"];function problem(arg){return g.indexOf(arg)}function unused(arg){return problem(arg)}function a(arg){return problem(arg)}function b(problem){return g[problem]}function c(arg){return b(a(arg))}console.log(c("a"));';
  const expected = ['a'];
  run(code, expected);
});

test('no_flatten_with_var_colliding_with_arg_value_inner_scope', () => {
  const code =
    'var g=["a"];function problem(arg){return g.indexOf(arg)}function unused(arg){return problem(arg)}function a(arg){return problem(arg)}function b(test){var problem=test*2;console.log(problem);return g[problem]}function c(arg){return b(a(arg))}console.log(c("a"));';
  const expected = ['0', 'a'];
  run(code, expected);
});

test('issue_t50', () => {
  const code =
    '(function(){var v1=[4,[{a:-1,b:5}]];var v2=[4,[{a:-2,b:5}]];var v3=[4,[{a:-3,b:5}]];var v4=[4,[{a:-4,b:5}]];var v5=[4,[{a:-5,b:5}]];var v6=[4,[{a:-6,b:5}]];var v7=[4,[{a:-7,b:5}]];var v8=[4,[{a:-8,b:5}]];var v9=[4,[{a:-9,b:5}]];var v10=[4,[{a:-10,b:5}]];var v11=[4,[{a:-11,b:5}]];var v12=[4,[{a:-12,b:5}]];var v13=[4,[{a:-13,b:5}]];var v14=[4,[{a:-14,b:5}]];var v15=[4,[{a:-15,b:5}]];var v16=[4,[{a:-16,b:5}]];var v17=[4,[{a:-17,b:5}]];var v18=[4,[{a:-18,b:5}]];var v19=[4,[{a:-19,b:5}]];var v20=[4,[{a:-20,b:5}]];var v21=[4,[{a:-21,b:5}]];var v22=[4,[{a:-22,b:5}]];var v23=[4,[{a:-23,b:5}]];var v24=[4,[{a:-24,b:5}]];var v25=[4,[{a:-25,b:5}]];var v26=[4,[{a:-26,b:5}]];var v27=[4,[{a:-27,b:5}]];var v28=[4,[{a:-28,b:5}]];var v29=[4,[{a:-29,b:5}]];var v30=[4,[{a:-30,b:5}]];var v31=[4,[{a:-31,b:5}]];var v32=[4,[{a:-32,b:5}]];var v33=[4,[{a:-33,b:5}]];var v34=[4,[{a:-34,b:5}]];var v35=[4,[{a:-35,b:5}]];var v36=[4,[{a:-36,b:5}]];var v37=[4,[{a:-37,b:5}]];var v38=[4,[{a:-38,b:5}]];var v39=[4,[{a:-39,b:5}]];var v40=[4,[{a:-40,b:5}]];var v41=[4,[{a:-41,b:5}]];var v42=[4,[{a:-42,b:5}]];var v43=[4,[{a:-43,b:5}]];var v44=[4,[{a:-44,b:5}]];var v45=[4,[{a:-45,b:5}]];var v46=[4,[{a:-46,b:5}]];var v47=[4,[{a:-47,b:5}]];var v48=[4,[{a:-48,b:5}]];var v49=[4,[{a:-49,b:5}]];var v50=[4,[{a:-50,b:5}]];var v51=[4,[{a:-51,b:5}]];var v52=[4,[{a:-52,b:5}]];var v53=[4,[{a:-53,b:5}]];var v54=[4,[{a:-54,b:5}]];var v55=[4,[{a:-55,b:5}]];var v56=[4,[{a:-56,b:5}]];var v57=[4,[{a:-57,b:5}]];var v58=[4,[{a:-58,b:5}]];var v59=[4,[{a:-59,b:5}]];var v60=[4,[{a:-60,b:5}]];var v61=[4,[{a:-61,b:5}]];var v62=[4,[{a:-62,b:5}]];var v63=[4,[{a:-63,b:5}]];var v64=[4,[{a:-64,b:5}]];var v65=[4,[{a:-65,b:5}]];var v66=[4,[{a:-66,b:5}]];var v67=[4,[{a:-67,b:5}]];var v68=[4,[{a:-68,b:5}]];var v69=[4,[{a:-69,b:5}]];var v70=[4,[{a:-70,b:5}]];var v71=[4,[{a:-71,b:5}]];var v72=[4,[{a:-72,b:5}]];var v73=[4,[{a:-73,b:5}]];var v74=[4,[{a:-74,b:5}]];var v75=[4,[{a:-75,b:5}]];var v76=[4,[{a:-76,b:5}]];var v77=[4,[{a:-77,b:5}]];var v78=[4,[{a:-78,b:5}]];var v79=[4,[{a:-79,b:5}]];var v80=[4,[{a:-80,b:5}]];var v81=[4,[{a:-81,b:5}]];var v82=[4,[{a:-82,b:5}]];var v83=[4,[{a:-83,b:5}]];var v84=[4,[{a:-84,b:5}]];var v85=[4,[{a:-85,b:5}]];var v86=[4,[{a:-86,b:5}]];var v87=[4,[{a:-87,b:5}]];var v88=[4,[{a:-88,b:5}]];var v89=[4,[{a:-89,b:5}]];var v90=[4,[{a:-90,b:5}]];var v91=[4,[{a:-91,b:5}]];var v92=[4,[{a:-92,b:5}]];var v93=[4,[{a:-93,b:5}]];var v94=[4,[{a:-94,b:5}]];var v95=[4,[{a:-95,b:5}]];var v96=[4,[{a:-96,b:5}]];var v97=[4,[{a:-97,b:5}]];var v98=[4,[{a:-98,b:5}]];var v99=[4,[{a:-99,b:5}]];var v100=[4,[{a:-100,b:5}]];var v101=[4,[{a:-101,b:5}]];var v102=[4,[{a:-102,b:5}]];var v103=[4,[{a:-103,b:5}]];var v104=[4,[{a:-104,b:5}]];var v105=[4,[{a:-105,b:5}]];var v106=[4,[{a:-106,b:5}]];var v107=[4,[{a:-107,b:5}]];var v108=[4,[{a:-108,b:5}]];var v109=[4,[{a:-109,b:5}]];var v110=[4,[{a:-110,b:5}]];var v111=[4,[{a:-111,b:5}]];var v112=[4,[{a:-112,b:5}]];var v113=[4,[{a:-113,b:5}]];var v114=[4,[{a:-114,b:5}]];var v115=[4,[{a:-115,b:5}]];var v116=[4,[{a:-116,b:5}]];var v117=[4,[{a:-117,b:5}]];var v118=[4,[{a:-118,b:5}]];var v119=[4,[{a:-119,b:5}]];var v120=[4,[{a:-120,b:5}]];var v121=[4,[{a:-121,b:5}]];var v122=[4,[{a:-122,b:5}]];var v123=[4,[{a:-123,b:5}]];var v124=[4,[{a:-124,b:5}]];var v125=[4,[{a:-125,b:5}]];var v126=[4,[{a:-126,b:5}]];var v127=[4,[{a:-127,b:5}]];var v128=[4,[{a:-128,b:5}]];var v129=[4,[{a:-129,b:5}]];var v130=[4,[{a:-130,b:5}]];var v131=[4,[{a:-131,b:5}]];var v132=[4,[{a:-132,b:5}]];var v133=[4,[{a:-133,b:5}]];var v134=[4,[{a:-134,b:5}]];var v135=[4,[{a:-135,b:5}]];var v136=[4,[{a:-136,b:5}]];var v137=[4,[{a:-137,b:5}]];var v138=[4,[{a:-138,b:5}]];var v139=[4,[{a:-139,b:5}]];var v140=[4,[{a:-140,b:5}]];var v141=[4,[{a:-141,b:5}]];var v142=[4,[{a:-142,b:5}]];var v143=[4,[{a:-143,b:5}]];var v144=[4,[{a:-144,b:5}]];var v145=[4,[{a:-145,b:5}]];var v146=[4,[{a:-146,b:5}]];var v147=[4,[{a:-147,b:5}]];var v148=[4,[{a:-148,b:5}]];var v149=[4,[{a:-149,b:5}]];var v150=[4,[{a:-150,b:5}]];var v151=[4,[{a:-151,b:5}]];var v152=[4,[{a:-152,b:5}]];var v153=[4,[{a:-153,b:5}]];var v154=[4,[{a:-154,b:5}]];var v155=[4,[{a:-155,b:5}]];var v156=[4,[{a:-156,b:5}]];var v157=[4,[{a:-157,b:5}]];var v158=[4,[{a:-158,b:5}]];var v159=[4,[{a:-159,b:5}]];var v160=[4,[{a:-160,b:5}]];var v161=[4,[{a:-161,b:5}]];var v162=[4,[{a:-162,b:5}]];var v163=[4,[{a:-163,b:5}]];var v164=[4,[{a:-164,b:5}]];var v165=[4,[{a:-165,b:5}]];var v166=[4,[{a:-166,b:5}]];var v167=[4,[{a:-167,b:5}]];var v168=[4,[{a:-168,b:5}]];var v169=[4,[{a:-169,b:5}]];var v170=[4,[{a:-170,b:5}]];var v171=[4,[{a:-171,b:5}]];var v172=[4,[{a:-172,b:5}]];var v173=[4,[{a:-173,b:5}]];var v174=[4,[{a:-174,b:5}]];var v175=[4,[{a:-175,b:5}]];var v176=[4,[{a:-176,b:5}]];var v177=[4,[{a:-177,b:5}]];var v178=[4,[{a:-178,b:5}]];var v179=[4,[{a:-179,b:5}]];var v180=[4,[{a:-180,b:5}]];var v181=[4,[{a:-181,b:5}]];var v182=[4,[{a:-182,b:5}]];var v183=[4,[{a:-183,b:5}]];var v184=[4,[{a:-184,b:5}]];var v185=[4,[{a:-185,b:5}]];var v186=[4,[{a:-186,b:5}]];var v187=[4,[{a:-187,b:5}]];var v188=[4,[{a:-188,b:5}]];var v189=[4,[{a:-189,b:5}]];var v190=[4,[{a:-190,b:5}]];var v191=[4,[{a:-191,b:5}]];var v192=[4,[{a:-192,b:5}]];var v193=[4,[{a:-193,b:5}]];var v194=[4,[{a:-194,b:5}]];var v195=[4,[{a:-195,b:5}]];var v196=[4,[{a:-196,b:5}]];var v197=[4,[{a:-197,b:5}]];var v198=[4,[{a:-198,b:5}]];var v199=[4,[{a:-199,b:5}]];var v200=[4,[{a:-200,b:5}]];var v201=[4,[{a:-201,b:5}]];var v202=[4,[{a:-202,b:5}]];var v203=[4,[{a:-203,b:5}]];var v204=[4,[{a:-204,b:5}]];var v205=[4,[{a:-205,b:5}]];var v206=[4,[{a:-206,b:5}]];var v207=[4,[{a:-207,b:5}]];var v208=[4,[{a:-208,b:5}]];var v209=[4,[{a:-209,b:5}]];var v210=[4,[{a:-210,b:5}]];var v211=[4,[{a:-211,b:5}]];var v212=[4,[{a:-212,b:5}]];var v213=[4,[{a:-213,b:5}]];var v214=[4,[{a:-214,b:5}]];var v215=[4,[{a:-215,b:5}]];var v216=[4,[{a:-216,b:5}]];var v217=[4,[{a:-217,b:5}]];var v218=[4,[{a:-218,b:5}]];var v219=[4,[{a:-219,b:5}]];var v220=[4,[{a:-220,b:5}]];var v221=[4,[{a:-221,b:5}]];var v222=[4,[{a:-222,b:5}]];var v223=[4,[{a:-223,b:5}]];var v224=[4,[{a:-224,b:5}]];var v225=[4,[{a:-225,b:5}]];var v226=[4,[{a:-226,b:5}]];var v227=[4,[{a:-227,b:5}]];var v228=[4,[{a:-228,b:5}]];var v229=[4,[{a:-229,b:5}]];var v230=[4,[{a:-230,b:5}]];var v231=[4,[{a:-231,b:5}]];var v232=[4,[{a:-232,b:5}]];var v233=[4,[{a:-233,b:5}]];var v234=[4,[{a:-234,b:5}]];var v235=[4,[{a:-235,b:5}]];var v236=[4,[{a:-236,b:5}]];var v237=[4,[{a:-237,b:5}]];var v238=[4,[{a:-238,b:5}]];var v239=[4,[{a:-239,b:5}]];var v240=[4,[{a:-240,b:5}]];var v241=[4,[{a:-241,b:5}]];var v242=[4,[{a:-242,b:5}]];var v243=[4,[{a:-243,b:5}]];var v244=[4,[{a:-244,b:5}]];var v245=[4,[{a:-245,b:5}]];var v246=[4,[{a:-246,b:5}]];var v247=[4,[{a:-247,b:5}]];var v248=[4,[{a:-248,b:5}]];var v249=[4,[{a:-249,b:5}]];var v250=[4,[{a:-250,b:5}]];var v251=[4,[{a:-251,b:5}]];var v252=[4,[{a:-252,b:5}]];var v253=[4,[{a:-253,b:5}]];var v254=[4,[{a:-254,b:5}]];var v255=[4,[{a:-255,b:5}]];var v256=[4,[{a:-256,b:5}]];var v257=[4,[{a:-257,b:5}]];var v258=[4,[{a:-258,b:5}]];var v259=[4,[{a:-259,b:5}]];var v260=[4,[{a:-260,b:5}]];var v261=[4,[{a:-261,b:5}]];var v262=[4,[{a:-262,b:5}]];var v263=[4,[{a:-263,b:5}]];var v264=[4,[{a:-264,b:5}]];var v265=[4,[{a:-265,b:5}]];var v266=[4,[{a:-266,b:5}]];var v267=[4,[{a:-267,b:5}]];var v268=[4,[{a:-268,b:5}]];var v269=[4,[{a:-269,b:5}]];var v270=[4,[{a:-270,b:5}]];var v271=[4,[{a:-271,b:5}]];var v272=[4,[{a:-272,b:5}]];var v273=[4,[{a:-273,b:5}]];var v274=[4,[{a:-274,b:5}]];var v275=[4,[{a:-275,b:5}]];var v276=[4,[{a:-276,b:5}]];var v277=[4,[{a:-277,b:5}]];var v278=[4,[{a:-278,b:5}]];var v279=[4,[{a:-279,b:5}]];var v280=[4,[{a:-280,b:5}]];var v281=[4,[{a:-281,b:5}]];var v282=[4,[{a:-282,b:5}]];var v283=[4,[{a:-283,b:5}]];var v284=[4,[{a:-284,b:5}]];var v285=[4,[{a:-285,b:5}]];var v286=[4,[{a:-286,b:5}]];var v287=[4,[{a:-287,b:5}]];var v288=[4,[{a:-288,b:5}]];var v289=[4,[{a:-289,b:5}]];var v290=[4,[{a:-290,b:5}]];var v291=[4,[{a:-291,b:5}]];var v292=[4,[{a:-292,b:5}]];var v293=[4,[{a:-293,b:5}]];var v294=[4,[{a:-294,b:5}]];var v295=[4,[{a:-295,b:5}]];var v296=[4,[{a:-296,b:5}]];var v297=[4,[{a:-297,b:5}]];var v298=[4,[{a:-298,b:5}]];var v299=[4,[{a:-299,b:5}]];var v300=[4,[{a:-300,b:5}]];var v301=[4,[{a:-301,b:5}]];var v302=[4,[{a:-302,b:5}]];var v303=[4,[{a:-303,b:5}]];var v304=[4,[{a:-304,b:5}]];var v305=[4,[{a:-305,b:5}]];var v306=[4,[{a:-306,b:5}]];var v307=[4,[{a:-307,b:5}]];var v308=[4,[{a:-308,b:5}]];var v309=[4,[{a:-309,b:5}]];var v310=[4,[{a:-310,b:5}]];var v311=[4,[{a:-311,b:5}]];var v312=[4,[{a:-312,b:5}]];var v313=[4,[{a:-313,b:5}]];var v314=[4,[{a:-314,b:5}]];var v315=[4,[{a:-315,b:5}]];var v316=[4,[{a:-316,b:5}]];var v317=[4,[{a:-317,b:5}]];var v318=[4,[{a:-318,b:5}]];var v319=[4,[{a:-319,b:5}]];var v320=[4,[{a:-320,b:5}]];var v321=[4,[{a:-321,b:5}]];var v322=[4,[{a:-322,b:5}]];var v323=[4,[{a:-323,b:5}]];var v324=[4,[{a:-324,b:5}]];var v325=[4,[{a:-325,b:5}]];var v326=[4,[{a:-326,b:5}]];var v327=[4,[{a:-327,b:5}]];var v328=[4,[{a:-328,b:5}]];var v329=[4,[{a:-329,b:5}]];var v330=[4,[{a:-330,b:5}]];var v331=[4,[{a:-331,b:5}]];var v332=[4,[{a:-332,b:5}]];var v333=[4,[{a:-333,b:5}]];var v334=[4,[{a:-334,b:5}]];var v335=[4,[{a:-335,b:5}]];var v336=[4,[{a:-336,b:5}]];var v337=[4,[{a:-337,b:5}]];var v338=[4,[{a:-338,b:5}]];var v339=[4,[{a:-339,b:5}]];var v340=[4,[{a:-340,b:5}]];var v341=[4,[{a:-341,b:5}]];var v342=[4,[{a:-342,b:5}]];var v343=[4,[{a:-343,b:5}]];var v344=[4,[{a:-344,b:5}]];var v345=[4,[{a:-345,b:5}]];var v346=[4,[{a:-346,b:5}]];var v347=[4,[{a:-347,b:5}]];var v348=[4,[{a:-348,b:5}]];var v349=[4,[{a:-349,b:5}]];var v350=[4,[{a:-350,b:5}]];var v351=[4,[{a:-351,b:5}]];var v352=[4,[{a:-352,b:5}]];var v353=[4,[{a:-353,b:5}]];var v354=[4,[{a:-354,b:5}]];var v355=[4,[{a:-355,b:5}]];var v356=[4,[{a:-356,b:5}]];var v357=[4,[{a:-357,b:5}]];var v358=[4,[{a:-358,b:5}]];var v359=[4,[{a:-359,b:5}]];var v360=[4,[{a:-360,b:5}]];var v361=[4,[{a:-361,b:5}]];var v362=[4,[{a:-362,b:5}]];var v363=[4,[{a:-363,b:5}]];var v364=[4,[{a:-364,b:5}]];var v365=[4,[{a:-365,b:5}]];var v366=[4,[{a:-366,b:5}]];var v367=[4,[{a:-367,b:5}]];var v368=[4,[{a:-368,b:5}]];var v369=[4,[{a:-369,b:5}]];var v370=[4,[{a:-370,b:5}]];var v371=[4,[{a:-371,b:5}]];var v372=[4,[{a:-372,b:5}]];var v373=[4,[{a:-373,b:5}]];var v374=[4,[{a:-374,b:5}]];var v375=[4,[{a:-375,b:5}]];var v376=[4,[{a:-376,b:5}]];var v377=[4,[{a:-377,b:5}]];var v378=[4,[{a:-378,b:5}]];var v379=[4,[{a:-379,b:5}]];var v380=[4,[{a:-380,b:5}]];var v381=[4,[{a:-381,b:5}]];var v382=[4,[{a:-382,b:5}]];var v383=[4,[{a:-383,b:5}]];var v384=[4,[{a:-384,b:5}]];var v385=[4,[{a:-385,b:5}]];var v386=[4,[{a:-386,b:5}]];var v387=[4,[{a:-387,b:5}]];var v388=[4,[{a:-388,b:5}]];var v389=[4,[{a:-389,b:5}]];var v390=[4,[{a:-390,b:5}]];var v391=[4,[{a:-391,b:5}]];var v392=[4,[{a:-392,b:5}]];var v393=[4,[{a:-393,b:5}]];var v394=[4,[{a:-394,b:5}]];var v395=[4,[{a:-395,b:5}]];var v396=[4,[{a:-396,b:5}]];var v397=[4,[{a:-397,b:5}]];var v398=[4,[{a:-398,b:5}]];var v399=[4,[{a:-399,b:5}]];var v400=[4,[{a:-400,b:5}]];var v401=[4,[{a:-401,b:5}]];var v402=[4,[{a:-402,b:5}]];var v403=[4,[{a:-403,b:5}]];var v404=[4,[{a:-404,b:5}]];var v405=[4,[{a:-405,b:5}]];var v406=[4,[{a:-406,b:5}]];var v407=[4,[{a:-407,b:5}]];var v408=[4,[{a:-408,b:5}]];var v409=[4,[{a:-409,b:5}]];var v410=[4,[{a:-410,b:5}]];var v411=[4,[{a:-411,b:5}]];var v412=[4,[{a:-412,b:5}]];var v413=[4,[{a:-413,b:5}]];var v414=[4,[{a:-414,b:5}]];var v415=[4,[{a:-415,b:5}]];var v416=[4,[{a:-416,b:5}]];var v417=[4,[{a:-417,b:5}]];var v418=[4,[{a:-418,b:5}]];var v419=[4,[{a:-419,b:5}]];var v420=[4,[{a:-420,b:5}]];var v421=[4,[{a:-421,b:5}]];var v422=[4,[{a:-422,b:5}]];var v423=[4,[{a:-423,b:5}]];var v424=[4,[{a:-424,b:5}]];var v425=[4,[{a:-425,b:5}]];var v426=[4,[{a:-426,b:5}]];var v427=[4,[{a:-427,b:5}]];var v428=[4,[{a:-428,b:5}]];var v429=[4,[{a:-429,b:5}]];var v430=[4,[{a:-430,b:5}]];var v431=[4,[{a:-431,b:5}]];var v432=[4,[{a:-432,b:5}]];var v433=[4,[{a:-433,b:5}]];var v434=[4,[{a:-434,b:5}]];var v435=[4,[{a:-435,b:5}]];var v436=[4,[{a:-436,b:5}]];var v437=[4,[{a:-437,b:5}]];var v438=[4,[{a:-438,b:5}]];var v439=[4,[{a:-439,b:5}]];var v440=[4,[{a:-440,b:5}]];var v441=[4,[{a:-441,b:5}]];var v442=[4,[{a:-442,b:5}]];var v443=[4,[{a:-443,b:5}]];var v444=[4,[{a:-444,b:5}]];var v445=[4,[{a:-445,b:5}]];var v446=[4,[{a:-446,b:5}]];var v447=[4,[{a:-447,b:5}]];var v448=[4,[{a:-448,b:5}]];var v449=[4,[{a:-449,b:5}]];var v450=[4,[{a:-450,b:5}]];var v451=[4,[{a:-451,b:5}]];var v452=[4,[{a:-452,b:5}]];var v453=[4,[{a:-453,b:5}]];var v454=[4,[{a:-454,b:5}]];var v455=[4,[{a:-455,b:5}]];var v456=[4,[{a:-456,b:5}]];var v457=[4,[{a:-457,b:5}]];var v458=[4,[{a:-458,b:5}]];var v459=[4,[{a:-459,b:5}]];var v460=[4,[{a:-460,b:5}]];var v461=[4,[{a:-461,b:5}]];var v462=[4,[{a:-462,b:5}]];var v463=[4,[{a:-463,b:5}]];var v464=[4,[{a:-464,b:5}]];var v465=[4,[{a:-465,b:5}]];var v466=[4,[{a:-466,b:5}]];var v467=[4,[{a:-467,b:5}]];var v468=[4,[{a:-468,b:5}]];var v469=[4,[{a:-469,b:5}]];var v470=[4,[{a:-470,b:5}]];var v471=[4,[{a:-471,b:5}]];var v472=[4,[{a:-472,b:5}]];var v473=[4,[{a:-473,b:5}]];var v474=[4,[{a:-474,b:5}]];var v475=[4,[{a:-475,b:5}]];var v476=[4,[{a:-476,b:5}]];var v477=[4,[{a:-477,b:5}]];var v478=[4,[{a:-478,b:5}]];var v479=[4,[{a:-479,b:5}]];var v480=[4,[{a:-480,b:5}]];var v481=[4,[{a:-481,b:5}]];var v482=[4,[{a:-482,b:5}]];var v483=[4,[{a:-483,b:5}]];var v484=[4,[{a:-484,b:5}]];var v485=[4,[{a:-485,b:5}]];var v486=[4,[{a:-486,b:5}]];var v487=[4,[{a:-487,b:5}]];var v488=[4,[{a:-488,b:5}]];var v489=[4,[{a:-489,b:5}]];var v490=[4,[{a:-490,b:5}]];var v491=[4,[{a:-491,b:5}]];var v492=[4,[{a:-492,b:5}]];var v493=[4,[{a:-493,b:5}]];var v494=[4,[{a:-494,b:5}]];var v495=[4,[{a:-495,b:5}]];var v496=[4,[{a:-496,b:5}]];var v497=[4,[{a:-497,b:5}]];var v498=[4,[{a:-498,b:5}]];var v499=[4,[{a:-499,b:5}]];var v500=[4,[{a:-500,b:5}]];var unused={p1:v1,p2:v2,p3:v3,p4:v4,p5:v5,p6:v6,p7:v7,p8:v8,p9:v9,p10:v10,p11:v11,p12:v12,p13:v13,p14:v14,p15:v15,p16:v16,p17:v17,p18:v18,p19:v19,p20:v20,p21:v21,p22:v22,p23:v23,p24:v24,p25:v25,p26:v26,p27:v27,p28:v28,p29:v29,p30:v30,p31:v31,p32:v32,p33:v33,p34:v34,p35:v35,p36:v36,p37:v37,p38:v38,p39:v39,p40:v40,p41:v41,p42:v42,p43:v43,p44:v44,p45:v45,p46:v46,p47:v47,p48:v48,p49:v49,p50:v50,p51:v51,p52:v52,p53:v53,p54:v54,p55:v55,p56:v56,p57:v57,p58:v58,p59:v59,p60:v60,p61:v61,p62:v62,p63:v63,p64:v64,p65:v65,p66:v66,p67:v67,p68:v68,p69:v69,p70:v70,p71:v71,p72:v72,p73:v73,p74:v74,p75:v75,p76:v76,p77:v77,p78:v78,p79:v79,p80:v80,p81:v81,p82:v82,p83:v83,p84:v84,p85:v85,p86:v86,p87:v87,p88:v88,p89:v89,p90:v90,p91:v91,p92:v92,p93:v93,p94:v94,p95:v95,p96:v96,p97:v97,p98:v98,p99:v99,p100:v100,p101:v101,p102:v102,p103:v103,p104:v104,p105:v105,p106:v106,p107:v107,p108:v108,p109:v109,p110:v110,p111:v111,p112:v112,p113:v113,p114:v114,p115:v115,p116:v116,p117:v117,p118:v118,p119:v119,p120:v120,p121:v121,p122:v122,p123:v123,p124:v124,p125:v125,p126:v126,p127:v127,p128:v128,p129:v129,p130:v130,p131:v131,p132:v132,p133:v133,p134:v134,p135:v135,p136:v136,p137:v137,p138:v138,p139:v139,p140:v140,p141:v141,p142:v142,p143:v143,p144:v144,p145:v145,p146:v146,p147:v147,p148:v148,p149:v149,p150:v150,p151:v151,p152:v152,p153:v153,p154:v154,p155:v155,p156:v156,p157:v157,p158:v158,p159:v159,p160:v160,p161:v161,p162:v162,p163:v163,p164:v164,p165:v165,p166:v166,p167:v167,p168:v168,p169:v169,p170:v170,p171:v171,p172:v172,p173:v173,p174:v174,p175:v175,p176:v176,p177:v177,p178:v178,p179:v179,p180:v180,p181:v181,p182:v182,p183:v183,p184:v184,p185:v185,p186:v186,p187:v187,p188:v188,p189:v189,p190:v190,p191:v191,p192:v192,p193:v193,p194:v194,p195:v195,p196:v196,p197:v197,p198:v198,p199:v199,p200:v200,p201:v201,p202:v202,p203:v203,p204:v204,p205:v205,p206:v206,p207:v207,p208:v208,p209:v209,p210:v210,p211:v211,p212:v212,p213:v213,p214:v214,p215:v215,p216:v216,p217:v217,p218:v218,p219:v219,p220:v220,p221:v221,p222:v222,p223:v223,p224:v224,p225:v225,p226:v226,p227:v227,p228:v228,p229:v229,p230:v230,p231:v231,p232:v232,p233:v233,p234:v234,p235:v235,p236:v236,p237:v237,p238:v238,p239:v239,p240:v240,p241:v241,p242:v242,p243:v243,p244:v244,p245:v245,p246:v246,p247:v247,p248:v248,p249:v249,p250:v250,p251:v251,p252:v252,p253:v253,p254:v254,p255:v255,p256:v256,p257:v257,p258:v258,p259:v259,p260:v260,p261:v261,p262:v262,p263:v263,p264:v264,p265:v265,p266:v266,p267:v267,p268:v268,p269:v269,p270:v270,p271:v271,p272:v272,p273:v273,p274:v274,p275:v275,p276:v276,p277:v277,p278:v278,p279:v279,p280:v280,p281:v281,p282:v282,p283:v283,p284:v284,p285:v285,p286:v286,p287:v287,p288:v288,p289:v289,p290:v290,p291:v291,p292:v292,p293:v293,p294:v294,p295:v295,p296:v296,p297:v297,p298:v298,p299:v299,p300:v300,p301:v301,p302:v302,p303:v303,p304:v304,p305:v305,p306:v306,p307:v307,p308:v308,p309:v309,p310:v310,p311:v311,p312:v312,p313:v313,p314:v314,p315:v315,p316:v316,p317:v317,p318:v318,p319:v319,p320:v320,p321:v321,p322:v322,p323:v323,p324:v324,p325:v325,p326:v326,p327:v327,p328:v328,p329:v329,p330:v330,p331:v331,p332:v332,p333:v333,p334:v334,p335:v335,p336:v336,p337:v337,p338:v338,p339:v339,p340:v340,p341:v341,p342:v342,p343:v343,p344:v344,p345:v345,p346:v346,p347:v347,p348:v348,p349:v349,p350:v350,p351:v351,p352:v352,p353:v353,p354:v354,p355:v355,p356:v356,p357:v357,p358:v358,p359:v359,p360:v360,p361:v361,p362:v362,p363:v363,p364:v364,p365:v365,p366:v366,p367:v367,p368:v368,p369:v369,p370:v370,p371:v371,p372:v372,p373:v373,p374:v374,p375:v375,p376:v376,p377:v377,p378:v378,p379:v379,p380:v380,p381:v381,p382:v382,p383:v383,p384:v384,p385:v385,p386:v386,p387:v387,p388:v388,p389:v389,p390:v390,p391:v391,p392:v392,p393:v393,p394:v394,p395:v395,p396:v396,p397:v397,p398:v398,p399:v399,p400:v400,p401:v401,p402:v402,p403:v403,p404:v404,p405:v405,p406:v406,p407:v407,p408:v408,p409:v409,p410:v410,p411:v411,p412:v412,p413:v413,p414:v414,p415:v415,p416:v416,p417:v417,p418:v418,p419:v419,p420:v420,p421:v421,p422:v422,p423:v423,p424:v424,p425:v425,p426:v426,p427:v427,p428:v428,p429:v429,p430:v430,p431:v431,p432:v432,p433:v433,p434:v434,p435:v435,p436:v436,p437:v437,p438:v438,p439:v439,p440:v440,p441:v441,p442:v442,p443:v443,p444:v444,p445:v445,p446:v446,p447:v447,p448:v448,p449:v449,p450:v450,p451:v451,p452:v452,p453:v453,p454:v454,p455:v455,p456:v456,p457:v457,p458:v458,p459:v459,p460:v460,p461:v461,p462:v462,p463:v463,p464:v464,p465:v465,p466:v466,p467:v467,p468:v468,p469:v469,p470:v470,p471:v471,p472:v472,p473:v473,p474:v474,p475:v475,p476:v476,p477:v477,p478:v478,p479:v479,p480:v480,p481:v481,p482:v482,p483:v483,p484:v484,p485:v485,p486:v486,p487:v487,p488:v488,p489:v489,p490:v490,p491:v491,p492:v492,p493:v493,p494:v494,p495:v495,p496:v496,p497:v497,p498:v498,p499:v499,p500:v500};console.log(v1[1][0].a,v10[1][0].a)})();';
  const expected = ['-1 -10'];
  run(code, expected);
});

test('issue_t50_let', () => {
  const code =
    '(function(){let v1=[4,[{a:-1,b:5}]];let v2=[4,[{a:-2,b:5}]];let v3=[4,[{a:-3,b:5}]];let v4=[4,[{a:-4,b:5}]];let v5=[4,[{a:-5,b:5}]];let v6=[4,[{a:-6,b:5}]];let v7=[4,[{a:-7,b:5}]];let v8=[4,[{a:-8,b:5}]];let v9=[4,[{a:-9,b:5}]];let v10=[4,[{a:-10,b:5}]];let v11=[4,[{a:-11,b:5}]];let v12=[4,[{a:-12,b:5}]];let v13=[4,[{a:-13,b:5}]];let v14=[4,[{a:-14,b:5}]];let v15=[4,[{a:-15,b:5}]];let v16=[4,[{a:-16,b:5}]];let v17=[4,[{a:-17,b:5}]];let v18=[4,[{a:-18,b:5}]];let v19=[4,[{a:-19,b:5}]];let v20=[4,[{a:-20,b:5}]];let unused={p1:v1,p2:v2,p3:v3,p4:v4,p5:v5,p6:v6,p7:v7,p8:v8,p9:v9,p10:v10,p11:v11,p12:v12,p13:v13,p14:v14,p15:v15,p16:v16,p17:v17,p18:v18,p19:v19,p20:v20};console.log(v1[1][0].a,v10[1][0].a)})();';
  const expected = ['-1 -10'];
  run(code, expected);
});

test('issue_t50_const', () => {
  const code =
    '(function(){const v1=[4,[{a:-1,b:5}]];const v2=[4,[{a:-2,b:5}]];const v3=[4,[{a:-3,b:5}]];const v4=[4,[{a:-4,b:5}]];const v5=[4,[{a:-5,b:5}]];const v6=[4,[{a:-6,b:5}]];const v7=[4,[{a:-7,b:5}]];const v8=[4,[{a:-8,b:5}]];const v9=[4,[{a:-9,b:5}]];const v10=[4,[{a:-10,b:5}]];const v11=[4,[{a:-11,b:5}]];const v12=[4,[{a:-12,b:5}]];const v13=[4,[{a:-13,b:5}]];const v14=[4,[{a:-14,b:5}]];const v15=[4,[{a:-15,b:5}]];const v16=[4,[{a:-16,b:5}]];const v17=[4,[{a:-17,b:5}]];const v18=[4,[{a:-18,b:5}]];const v19=[4,[{a:-19,b:5}]];const v20=[4,[{a:-20,b:5}]];const unused={p1:v1,p2:v2,p3:v3,p4:v4,p5:v5,p6:v6,p7:v7,p8:v8,p9:v9,p10:v10,p11:v11,p12:v12,p13:v13,p14:v14,p15:v15,p16:v16,p17:v17,p18:v18,p19:v19,p20:v20};console.log(v1[1][0].a,v10[1][0].a)})();';
  const expected = ['-1 -10'];
  run(code, expected);
});

test('keep_fnames_and_avoid_collisions', () => {
  const code =
    'global.t="ttttttttttttttttttttt";(function testBug(){var param1="PASS";return()=>{console.log(param1);var t=function(){};return t}})()();';
  const expected = ['PASS'];
  run(code, expected);
});

test('keep_quoted_strict', () => {
  const code =
    'var a={propa:1,get propb(){return 2},propc:3,get propd(){return 4}};var b={propa:5,get propb(){return 6},propc:7,get propd(){return 8}};var c={};Object.defineProperty(c,"propa",{value:9});Object.defineProperty(c,"propc",{value:10});console.log(a.propa,a.propb,a.propc,a["propc"],a.propd,a["propd"]);console.log(b["propa"],b["propb"],b.propc,b["propc"],b.propd,b["propd"]);console.log(c.propa,c["propc"]);';
  const expected = ['1 2 3 3 4 4', '5 6 7 7 8 8', '9 10'];
  run(code, expected);
});

test('dead_code_condition', () => {
  const code = 'for(var a=0,b=5;(a+=1,3)-3&&b>0;b--){var c=function(){b--}(a++)}console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2740_3', () => {
  const code = 'L1:for(var x=0;x<3;x++){L2:for(var y=0;y<2;y++){break L1}}console.log(x,y);';
  const expected = ['0 0'];
  run(code, expected);
});

test('issue_2740_4', () => {
  const code = 'L1:for(var x=0;x<3;x++){L2:for(var y=0;y<2;y++){break L2}}console.log(x,y);';
  const expected = ['3 0'];
  run(code, expected);
});

test('issue_2740_5', () => {
  const code = 'L1:for(var x=0;x<3;x++){break L1;L2:for(var y=0;y<2;y++){break L2}}console.log(x,y);';
  const expected = ['0 undefined'];
  run(code, expected);
});

test('issue_2740_6', () => {
  const code = 'const a=9,b=0;for(const a=1;a<3;++b)break;console.log(a,b);';
  const expected = ['9 0'];
  run(code, expected);
});

test('issue_2740_7', () => {
  const code = 'let a=9,b=0;for(const a=1;a<3;++b)break;console.log(a,b);';
  const expected = ['9 0'];
  run(code, expected);
});

test('issue_2740_8', () => {
  const code = 'var a=9,b=0;for(const a=1;a<3;++b)break;console.log(a,b);';
  const expected = ['9 0'];
  run(code, expected);
});

test('issue_2904', () => {
  const code = 'var a=1;do{console.log(a)}while(--a);';
  const expected = ['1'];
  run(code, expected);
});

test('dot_parenthesis_1', () => {
  const code = 'console.log(new(Math.random().constructor)instanceof Number);';
  const expected = ['true'];
  run(code, expected);
});

test('dot_parenthesis_2', () => {
  const code = 'console.log(typeof new function(){Math.random()}.constructor);';
  const expected = ['function'];
  run(code, expected);
});

test('eval_let_6', () => {
  const code = 'eval("let a;");console.log();';
  const expected = [''];
  run(code, expected);
});

test('simplify_nullish_coalescing', () => {
  const code =
    'const y=id("one");const is_null=null;const not_null="two";console.log(is_null??y);console.log(not_null??y);';
  const expected = ['one', 'two'];
  run(code, expected);
});

test('convert_computed_props_to_regular_ones', () => {
  const code =
    'var o={["hi"]:0,["A"+1]:1,[/B/]:2,[100+23]:3,[1+.5]:4,[Math.PI]:5,[undefined]:6,[true]:7,[false]:8,[null]:9,[Infinity]:10,[NaN]:11};for(var k in o){console.log(k,o[k])}';
  const expected = [
    '123 3',
    'hi 0',
    'A1 1',
    '/B/ 2',
    '1.5 4',
    '3.141592653589793 5',
    'undefined 6',
    'true 7',
    'false 8',
    'null 9',
    'Infinity 10',
    'NaN 11',
  ];
  run(code, expected);
});

test('computed_property_names_side_effects', () => {
  const code = 'const foo={[console.log("PASS")]:42};';
  const expected = ['PASS'];
  run(code, expected);
});

test('prop_func_to_concise_method', () => {
  const code = '({emit:function NamedFunctionExpression(){console.log("PASS")},run:function(){this.emit()}}).run();';
  const expected = ['PASS'];
  run(code, expected);
});

test('prop_arrow_to_concise_method', () => {
  const code = '({run:()=>{console.log("PASS")}}).run();';
  const expected = ['PASS'];
  run(code, expected);
});

test('concise_method_to_prop_arrow', () => {
  const code =
    'console.log({a:()=>1}.a());console.log({a:()=>2}.a());console.log({a(){return 3}}.a());console.log({a(){return this.b},b:4}.a());';
  const expected = ['1', '2', '3', '4'];
  run(code, expected);
});

test('prop_func_to_async_concise_method', () => {
  const code = '({run:async function(){console.log("PASS")}}).run();';
  const expected = ['PASS'];
  run(code, expected);
});

test('prop_arrow_with_this', () => {
  const code =
    'function run(arg){console.log(arg===this?"global":arg===foo?"foo":arg)}var foo={func_no_this:function(){run()},func_with_this:function(){run(this)},arrow_no_this:()=>{run()},arrow_with_this:()=>{run(this)}};for(var key in foo)foo[key]();';
  const expected = ['undefined', 'foo', 'undefined', 'global'];
  run(code, expected);
});

test('prop_arrow_with_nested_this', () => {
  const code =
    'function run(arg){console.log(arg===this?"global":arg===foo?"foo":arg)}var foo={func_func_this:function(){(function(){run(this)})()},func_arrow_this:function(){(()=>{run(this)})()},arrow_func_this:()=>{(function(){run(this)})()},arrow_arrow_this:()=>{(()=>{run(this)})()}};for(var key in foo)foo[key]();';
  const expected = ['global', 'foo', 'global', 'global'];
  run(code, expected);
});

test('issue_2554_1', () => {
  const code =
    'var obj={["x"+""]:1,["method"+""](){this.s="PASS"},get["g"+""](){return this.x},set["s"+""](value){this.x=value}};obj.method();console.log(obj.g);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2554_2', () => {
  const code =
    'var instance=new class{constructor(){this.x=2}["method"+""](){this.s="PASS"}get["g"+""](){return this.x}set["s"+""](value){this.x=value}};instance.method();console.log(instance.g);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2554_3', () => {
  const code =
    'var foo={[1+0]:1,[2+0](){this[4]="PASS"},get[3+0](){return this[1]},set[4+0](value){this[1]=value}};foo[2]();console.log(foo[3]);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2554_4', () => {
  const code =
    'var bar=new class{constructor(){this[1]=2}[2+0](){this[4]="PASS"}get[3+0](){return this[1]}set[4+0](value){this[1]=value}};bar[2]();console.log(bar[3]);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2554_5', () => {
  const code = 'new class{["constructor"](){console.log("FAIL")}constructor(){console.log("PASS")}};';
  const expected = ['PASS'];
  run(code, expected);
});

test('accept_destructuring_async_word_with_default', () => {
  const code = 'console.log((({async:async="PASS"})=>async)({}));';
  const expected = ['PASS'];
  run(code, expected);
});

test('native_prototype_lhs', () => {
  const code = 'console.log(function(){Function.prototype.bar="PASS";return function(){}}().bar);';
  const expected = ['PASS'];
  run(code, expected);
});

test('accessor_boolean', () => {
  const code = 'var a=1;var b={get true(){return a},set false(c){a=c}};console.log(b.true,b.false=2,b.true);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('accessor_get_set', () => {
  const code = 'var a=1;var b={get set(){return a},set get(c){a=c}};console.log(b.set,b.get=2,b.set);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('accessor_null_undefined', () => {
  const code = 'var a=1;var b={get null(){return a},set undefined(c){a=c}};console.log(b.null,b.undefined=2,b.null);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('accessor_number', () => {
  const code = 'var a=1;var b={get 42(){return a},set 42(c){a=c}};console.log(b[42],b[42]=2,b[42]);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('accessor_string', () => {
  const code = 'var a=1;var b={get"a-b"(){return a},set"a-b"(c){a=c}};console.log(b["a-b"],b["a-b"]=2,b["a-b"]);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('accessor_this', () => {
  const code = 'var a=1;var b={get this(){return a},set this(c){a=c}};console.log(b.this,b.this=2,b.this);';
  const expected = ['1 2 2'];
  run(code, expected);
});

test('issue_2208_1', () => {
  const code = 'console.log({p:function(){return 42}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_2', () => {
  const code = 'console.log({a:42,p:function(){return this.a}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_3', () => {
  const code = 'a=42;console.log({p:function(){return function(){return this.a}()}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_4', () => {
  const code = 'function foo(){}console.log({a:foo(),p:function(){return 42}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_5', () => {
  const code = 'console.log({p:"FAIL",p:function(){return 42}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_6', () => {
  const code = 'console.log({p:()=>42}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_7', () => {
  const code = 'console.log({p(){return 42}}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2208_9', () => {
  const code = 'a=42;console.log({p:()=>function(){return this.a}()}.p());';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2321', () => {
  const code =
    'var f={foo:function(){console.log("foo")},bar(){console.log("bar")}};var foo=new f.foo;var bar=f.bar();';
  const expected = ['foo', 'bar'];
  run(code, expected);
});

test('unsafe_methods_regex', () => {
  const code =
    'var f={123:function(){console.log("123")},foo:function(){console.log("foo")},bar(){console.log("bar")},Baz:function(){console.log("baz")},BOO:function(){console.log("boo")},null:function(){console.log("null")},undefined:function(){console.log("undefined")}};f[123]();new f.foo;f.bar();f.Baz();f.BOO();new f.null;new f.undefined;';
  const expected = ['123', 'foo', 'bar', 'baz', 'boo', 'null', 'undefined'];
  run(code, expected);
});

test('lhs_prop_1', () => {
  const code = 'console.log(++{a:1}.a);';
  const expected = ['2'];
  run(code, expected);
});

test('literal_duplicate_key_side_effects', () => {
  const code = 'console.log({a:"FAIL",a:console.log?"PASS":"FAIL"}.a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('prop_side_effects_1', () => {
  const code = 'var C=1;console.log(C);var obj={bar:function(){return C+C}};console.log(obj.bar());';
  const expected = ['1', '2'];
  run(code, expected);
});

test('prop_side_effects_2', () => {
  const code = 'var C=1;console.log(C);var obj={"":function(){return C+C}};console.log(obj[""]());';
  const expected = ['1', '2'];
  run(code, expected);
});

test('accessor_1', () => {
  const code = 'console.log({a:"FAIL",get a(){return"PASS"}}.a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('array_hole', () => {
  const code = 'console.log([1,2,,3][1],[1,2,,3][2],[1,2,,3][3]);';
  const expected = ['2 undefined 3'];
  run(code, expected);
});

test('computed_property', () => {
  const code = 'console.log({a:"bar",[console.log("foo")]:42}.a);';
  const expected = ['foo', 'bar'];
  run(code, expected);
});

test('issue_2513', () => {
  const code =
    '!function(Infinity,NaN,undefined){console.log("a"[1/0],"b"["Infinity"]);console.log("c"[0/0],"d"["NaN"]);console.log("e"[void 0],"f"["undefined"])}(0,0,0);';
  const expected = ['undefined undefined', 'undefined undefined', 'undefined undefined'];
  run(code, expected);
});

test('join_object_assignments_2', () => {
  const code = 'var o={foo:1};o.bar=2;o.baz=3;console.log(o.foo,o.bar+o.bar,o.foo*o.bar*o.baz);';
  const expected = ['1 4 6'];
  run(code, expected);
});

test('join_object_assignments_3', () => {
  const code = 'console.log(function(){var o={a:"PASS"},a=o.a;o.a="FAIL";return a}());';
  const expected = ['PASS'];
  run(code, expected);
});

test('join_object_assignments_4', () => {
  const code = 'var o;console.log(o);o={};o.a="foo";console.log(o.b);o.b="bar";console.log(o.a);';
  const expected = ['undefined', 'undefined', 'foo'];
  run(code, expected);
});

test('join_object_assignments_return_1', () => {
  const code = 'console.log(function(){var o={p:3};return o.q="foo"}());';
  const expected = ['foo'];
  run(code, expected);
});

test('join_object_assignments_return_2', () => {
  const code = 'console.log(function(){var o={p:3};return o.q=/foo/,o.r="bar"}());';
  const expected = ['bar'];
  run(code, expected);
});

test('join_object_assignments_return_3', () => {
  const code = 'console.log(function(){var o={p:3};return o.q="foo",o.p+="",console.log(o.q),o.p}());';
  const expected = ['foo', '3'];
  run(code, expected);
});

test('join_object_assignments_for', () => {
  const code = 'console.log(function(){var o={p:3};for(o.q="foo";console.log(o.q););return o.p}());';
  const expected = ['foo', '3'];
  run(code, expected);
});

test('join_object_assignments_if', () => {
  const code = 'console.log(function(){var o={};if(o.a="PASS")return o.a}());';
  const expected = ['PASS'];
  run(code, expected);
});

test('join_object_assignments_forin', () => {
  const code = 'console.log(function(){var o={};for(var a in o.a="PASS",o)return o[a]}());';
  const expected = ['PASS'];
  run(code, expected);
});

test('join_object_assignments_negative', () => {
  const code = 'var o={};o[0]=0;o[-0]=1;o[-1]=2;console.log(o[0],o[-0],o[-1]);';
  const expected = ['1 1 2'];
  run(code, expected);
});

test('join_object_assignments_NaN_1', () => {
  const code = 'var o={};o[NaN]=1;o[0/0]=2;console.log(o[NaN],o[NaN]);';
  const expected = ['2 2'];
  run(code, expected);
});

test('join_object_assignments_NaN_2', () => {
  const code = 'var o={};o[NaN]=1;o[0/0]=2;console.log(o[NaN],o[NaN]);';
  const expected = ['2 2'];
  run(code, expected);
});

test('join_object_assignments_null_0', () => {
  const code = 'var o={};o[null]=1;console.log(o[null]);';
  const expected = ['1'];
  run(code, expected);
});

test('join_object_assignments_null_1', () => {
  const code = 'var o={};o[null]=1;console.log(o[null]);';
  const expected = ['1'];
  run(code, expected);
});

test('join_object_assignments_void_0', () => {
  const code = 'var o={};o[void 0]=1;console.log(o[void 0]);';
  const expected = ['1'];
  run(code, expected);
});

test('join_object_assignments_undefined_1', () => {
  const code = 'var o={};o[undefined]=1;console.log(o[undefined]);';
  const expected = ['1'];
  run(code, expected);
});

test('join_object_assignments_undefined_2', () => {
  const code = 'var o={};o[undefined]=1;console.log(o[undefined]);';
  const expected = ['1'];
  run(code, expected);
});

test('join_object_assignments_Infinity', () => {
  const code =
    'var o={};o[Infinity]=1;o[1/0]=2;o[-Infinity]=3;o[-1/0]=4;console.log(o[Infinity],o[1/0],o[-Infinity],o[-1/0]);';
  const expected = ['2 2 4 4'];
  run(code, expected);
});

test('join_object_assignments_regex', () => {
  const code = 'var o={};o[/rx/]=1;console.log(o[/rx/]);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2816', () => {
  const code = '"use strict";var o={a:1};o.b=2;o.a=3;o.c=4;console.log(o.a,o.b,o.c);';
  const expected = ['3 2 4'];
  run(code, expected);
});

test('issue_2816_ecma6', () => {
  const code = '"use strict";var o={a:1};o.b=2;o.a=3;o.c=4;console.log(o.a,o.b,o.c);';
  const expected = ['3 2 4'];
  run(code, expected);
});

test('issue_2893_1', () => {
  const code = 'var o={get a(){return"PASS"}};o.a="FAIL";console.log(o.a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2893_2', () => {
  const code = 'var o={set a(v){this.b=v},b:"FAIL"};o.a="PASS";console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2893_3', () => {
  const code = 'var o={get a(){return"PASS"}};o.a="FAIL";console.log(o.a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2893_4', () => {
  const code = 'var o={set a(v){this.b=v},b:"FAIL"};o.a="PASS";console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2893_6', () => {
  const code = '"use strict";var o={set a(v){this.b=v},b:"FAIL"};o.a="PASS";console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2893_8', () => {
  const code = '"use strict";var o={set a(v){this.b=v},b:"FAIL"};o.a="PASS";console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_869_1', () => {
  const code = 'var o={p:"FAIL"};Object.defineProperty(o,"p",{get:function(){return"PASS"}});console.log(o.p);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_869_2', () => {
  const code = 'var o={p:"FAIL"};Object.defineProperties(o,{p:{get:function(){return"PASS"}}});console.log(o.p);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3188_1', () => {
  const code = '(function(){function f(){console.log(this.p)}(function(){var o={p:"PASS",f:f};o.f()})()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3188_2', () => {
  const code = '(function(){var f=function(){console.log(this.p)};function g(){var o={p:"PASS",f:f};o.f()}g()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3188_3', () => {
  const code = '(function(){function f(){console.log(this[0])}(function(){var o=["PASS",f];o[1]()})()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_t64', () => {
  const code =
    'var obj={};obj.Base=class{constructor(){this.id="PASS"}};obj.Derived=class extends obj.Base{constructor(){super();console.log(this.id)}};new obj.Derived;';
  const expected = ['PASS'];
  run(code, expected);
});

test('dont_mangle_computed_property_1', () => {
  const code =
    '"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";const prop=Symbol("foo");const obj={[prop]:"bar",baz:1,qux:2,[3+4]:"seven",0:"zero",1:"one",null:"Null",undefined:"Undefined",Infinity:"infinity",NaN:"nan",void:"Void"};console.log(obj[prop],obj["baz"],obj.qux,obj[7],obj[0],obj[1+0],obj[null],obj[undefined],obj[1/0],obj[NaN],obj.void);console.log(obj.null,obj.undefined,obj.Infinity,obj.NaN);';
  const expected = ['bar 1 2 seven zero one Null Undefined infinity nan Void', 'Null Undefined infinity nan'];
  run(code, expected);
});

test('dont_mangle_computed_property_2', () => {
  const code =
    'const prop=Symbol("foo");const obj={[prop]:"bar",baz:1,qux:2,[3+4]:"seven",0:"zero",1:"one",null:"Null",undefined:"Undefined",Infinity:"infinity",NaN:"nan",void:"Void"};console.log(obj[prop],obj["baz"],obj.qux,obj[7],obj[0],obj[1+0],obj[null],obj[undefined],obj[1/0],obj[NaN],obj.void);console.log(obj.null,obj.undefined,obj.Infinity,obj.NaN);';
  const expected = ['bar 1 2 seven zero one Null Undefined infinity nan Void', 'Null Undefined infinity nan'];
  run(code, expected);
});

test('issue_3065_2b', () => {
  const code =
    'function modifyWrapper(a,f,wrapper){wrapper.a=a;wrapper.f=f;return wrapper}function pureFunc(fun){return modifyWrapper(1,fun,(function(a){return fun(a)}))}var unused=pureFunc((function(x){return x}));function print(message){console.log(message)}print(2);print(3);';
  const expected = ['2', '3'];
  run(code, expected);
});

test('impure_getter_1', () => {
  const code = '({get a(){console.log(1)},b:1}).a;({get a(){console.log(1)},b:1}).b;';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2110_1', () => {
  const code = 'function f(){function f(){}function g(){return this}f.g=g;return f.g()}console.log(typeof f());';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2110_2', () => {
  const code = 'function f(){function f(){}function g(){return this}f.g=g;return f.g()}console.log(typeof f());';
  const expected = ['function'];
  run(code, expected);
});

test('set_immutable_1', () => {
  const code = 'var a=1;a.foo+="";if(a.foo)console.log("FAIL");else console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('set_immutable_2', () => {
  const code = 'var a=1;a.foo+="";if(a.foo)console.log("FAIL");else console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('set_mutable_1', () => {
  const code = '!function a(){a.foo+="";if(a.foo)console.log("PASS");else console.log("FAIL")}();';
  const expected = ['PASS'];
  run(code, expected);
});

test('set_mutable_2', () => {
  const code = '!function a(){a.foo+="";if(a.foo)console.log("PASS");else console.log("FAIL")}();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2313_1', () => {
  const code =
    'function x(){console.log(1);return{y:function(){console.log(2);return{z:0}}}}x().y().z++;if(x().y().z){console.log(3)}';
  const expected = ['1', '2', '1', '2'];
  run(code, expected);
});

test('issue_2313_2', () => {
  const code =
    'function x(){console.log(1);return{y:function(){console.log(2);return{z:0}}}}x().y().z++;if(x().y().z){console.log(3)}';
  const expected = ['1', '2', '1', '2'];
  run(code, expected);
});

test('issue_2313_3', () => {
  const code =
    'function x(){console.log(1);return{y:function(){console.log(2);return{z:0}}}}x().y().z++;if(x().y().z){console.log(3)}';
  const expected = ['1', '2', '1', '2'];
  run(code, expected);
});

test('issue_2313_4', () => {
  const code =
    'function x(){console.log(1);return{y:function(){console.log(2);return{z:0}}}}x().y().z++;if(x().y().z){console.log(3)}';
  const expected = ['1', '2', '1', '2'];
  run(code, expected);
});

test('issue_2313_7', () => {
  const code =
    'var a=0,b=0;class foo{get c(){a++;return 42}set c(c){b++}}class bar extends foo{d(){super.c++;if(super.c)console.log(a,b)}}(new bar).d();';
  const expected = ['2 1'];
  run(code, expected);
});

test('issue_2678', () => {
  const code = 'var a=1,c="FAIL";(function f(){(a--&&f()).p;return{get p(){c="PASS"}}})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2838', () => {
  const code =
    'function f(a,b){(a||b).c="PASS";(function(){return f(a,b)}).prototype.foo="bar"}var o={};f(null,o);console.log(o.c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2938_1', () => {
  const code = 'function f(a){a.b="PASS"}var o={};f(o);console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2938_2', () => {
  const code =
    'var Parser=function Parser(){};var p=Parser.prototype;p.initialContext=function initialContext(){console.log("PASS")};p.braceIsBlock=function(){};(new Parser).initialContext();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2938_3', () => {
  const code = 'function f(a){var unused=a.a;a.b="PASS";a.c}var o={};o.d;f(o);console.log(o.b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2938_4', () => {
  const code =
    'var Parser=function Parser(){};var p=Parser.prototype;var unused=p.x;p.initialContext=function initialContext(){p.y;console.log("PASS")};p.braceIsBlock=function(){};(new Parser).initialContext();';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_true', () => {
  const code =
    'console.log((42..length="PASS","PASS"));console.log(("foo".length="PASS","PASS"));console.log((false.length="PASS","PASS"));console.log((function(){}.length="PASS","PASS"));console.log(({get length(){return"FAIL"}}.length="PASS","PASS"));';
  const expected = ['PASS', 'PASS', 'PASS', 'PASS', 'PASS'];
  run(code, expected);
});

test('collapse_rhs_false', () => {
  const code =
    'console.log((42..length="PASS","PASS"));console.log(("foo".length="PASS","PASS"));console.log((false.length="PASS","PASS"));console.log((function(){}.length="PASS","PASS"));console.log(({get length(){return"FAIL"}}.length="PASS","PASS"));';
  const expected = ['PASS', 'PASS', 'PASS', 'PASS', 'PASS'];
  run(code, expected);
});

test('collapse_rhs_strict', () => {
  const code =
    'console.log((42..length="PASS","PASS"));console.log(("foo".length="PASS","PASS"));console.log((false.length="PASS","PASS"));console.log((function(){}.length="PASS","PASS"));console.log(({get length(){return"FAIL"}}.length="PASS","PASS"));';
  const expected = ['PASS', 'PASS', 'PASS', 'PASS', 'PASS'];
  run(code, expected);
});

test('collapse_rhs_setter', () => {
  const code = 'try{console.log(({set length(v){throw"PASS"}}.length="FAIL","FAIL"))}catch(e){console.log(e)}';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_call', () => {
  const code = 'var o={};function f(){console.log("PASS")}o.f=f;f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('collapse_rhs_lhs', () => {
  const code = 'function f(a,b){a.b=b,b+=2;console.log(a.b,b)}f({},1);';
  const expected = ['1 3'];
  run(code, expected);
});

test('window_access_is_impure', () => {
  const code = 'try{window}catch(e){console.log("PASS")}';
  const expected = ['PASS'];
  run(code, expected);
});

test('modified', () => {
  const code =
    'function f0(){var a=1,b=2;b++;console.log(a+1);console.log(b+1)}function f1(){var a=1,b=2;--b;console.log(a+1);console.log(b+1)}function f2(){var a=1,b=2,c=3;b=c;console.log(a+b);console.log(b+c);console.log(a+c);console.log(a+b+c)}function f3(){var a=1,b=2,c=3;b*=c;console.log(a+b);console.log(b+c);console.log(a+c);console.log(a+b+c)}function f4(){var a=1,b=2,c=3;if(a){b=c}else{c=b}console.log(a+b);console.log(b+c);console.log(a+c);console.log(a+b+c)}function f5(a){B=a;console.log(typeof A?"yes":"no");console.log(typeof B?"yes":"no")}f0(),f1(),f2(),f3(),f4(),f5();';
  const expected = ['2', '4', '2', '2', '4', '6', '4', '7', '7', '9', '4', '10', '4', '6', '4', '7', 'yes', 'yes'];
  run(code, expected);
});

test('unsafe_evaluate_array_3', () => {
  const code = 'var arr=[1,2,function(){return++arr[0]}];console.log(arr[0],arr[1],arr[2](),arr[0]);';
  const expected = ['1 2 2 2'];
  run(code, expected);
});

test('unsafe_evaluate_array_5', () => {
  const code = 'var arr=[1,2,function(){return++this[0]}];console.log(arr[0],arr[1],arr[2](),arr[0]);';
  const expected = ['1 2 2 2'];
  run(code, expected);
});

test('inner_var_for_2', () => {
  const code = '!function(){var a=1;for(var b=1;--b;)var a=2;console.log(a)}();';
  const expected = ['1'];
  run(code, expected);
});

test('unused_modified', () => {
  const code = 'console.log(function(){var b=1,c="FAIL";if(0||b--)c="PASS";b=1;return c}());';
  const expected = ['PASS'];
  run(code, expected);
});

test('iife_func_side_effects', () => {
  const code =
    'function x(){console.log("x")}function y(){console.log("y")}function z(){console.log("z")}(function(a,b,c){function y(){console.log("FAIL")}return y+b()})(x(),(function(){return y()}),z());';
  const expected = ['x', 'z', 'y'];
  run(code, expected);
});

test('issue_1670_1', () => {
  const code =
    '(function f(){switch(1){case 0:var a=true;break;default:if(typeof a==="undefined")console.log("PASS");else console.log("FAIL")}})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1670_2', () => {
  const code =
    '(function f(){switch(1){case 0:var a=true;break;default:if(typeof a==="undefined")console.log("PASS");else console.log("FAIL")}})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1670_3', () => {
  const code =
    '(function f(){switch(1){case 0:var a=true;break;case 1:if(typeof a==="undefined")console.log("PASS");else console.log("FAIL")}})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1670_4', () => {
  const code =
    '(function f(){switch(1){case 0:var a=true;break;case 1:if(typeof a==="undefined")console.log("PASS");else console.log("FAIL")}})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1670_5', () => {
  const code = '(function(a){switch(1){case a:console.log(a);break;default:console.log(2);break}})(1);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1670_6', () => {
  const code = '(function(a){switch(1){case a=1:console.log(a);break;default:console.log(2);break}})(1);';
  const expected = ['1'];
  run(code, expected);
});

test('redefine_arguments_1', () => {
  const code =
    'function f(){var arguments;return typeof arguments}function g(){var arguments=42;return typeof arguments}function h(x){var arguments=x;return typeof arguments}console.log(f(),g(),h());';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('redefine_arguments_2', () => {
  const code =
    'function f(){var arguments;return typeof arguments}function g(){var arguments=42;return typeof arguments}function h(x){var arguments=x;return typeof arguments}console.log(f(),g(),h());';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('redefine_arguments_3', () => {
  const code =
    'function f(){var arguments;return typeof arguments}function g(){var arguments=42;return typeof arguments}function h(x){var arguments=x;return typeof arguments}console.log(f(),g(),h());';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('redefine_farg_1', () => {
  const code =
    'function f(a){var a;return typeof a}function g(a){var a=42;return typeof a}function h(a,b){var a=b;return typeof a}console.log(f([]),g([]),h([]));';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('redefine_farg_2', () => {
  const code =
    'function f(a){var a;return typeof a}function g(a){var a=42;return typeof a}function h(a,b){var a=b;return typeof a}console.log(f([]),g([]),h([]));';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('redefine_farg_3', () => {
  const code =
    'function f(a){var a;return typeof a}function g(a){var a=42;return typeof a}function h(a,b){var a=b;return typeof a}console.log(f([]),g([]),h([]));';
  const expected = ['object number undefined'];
  run(code, expected);
});

test('delay_def_lhs', () => {
  const code = 'console.log(function(){long_name++;return long_name;var long_name}());';
  const expected = ['NaN'];
  run(code, expected);
});

test('booleans', () => {
  const code = 'console.log(function(a){if(a!=0);switch(a){case 0:return"FAIL";case false:return"PASS"}}(false));';
  const expected = ['PASS'];
  run(code, expected);
});

test('side_effects_assign', () => {
  const code = 'var a=typeof void(a&&a.in==1,0);console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('pure_getters_1', () => {
  const code = 'try{var a=(a.b,2)}catch(e){}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('catch_var', () => {
  const code = 'try{throw{}}catch(e){var e;console.log(!!e)}';
  const expected = ['true'];
  run(code, expected);
});

test('var_assign_1', () => {
  const code = '!function(){var a;a=2;console.log(a)}();';
  const expected = ['2'];
  run(code, expected);
});

test('var_assign_2', () => {
  const code = '!function(){var a;if(a=2)console.log(a)}();';
  const expected = ['2'];
  run(code, expected);
});

test('var_assign_5', () => {
  const code = '!function(){var a;!function(b){a=2;console.log(a,b)}(a)}();';
  const expected = ['2 undefined'];
  run(code, expected);
});

test('var_assign_6', () => {
  const code = '!function(){var a=function(){}(a=1);console.log(a)}();';
  const expected = ['undefined'];
  run(code, expected);
});

test('immutable', () => {
  const code = '!function(){var a="test";console.log(a.indexOf("e"))}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1814_1', () => {
  const code = 'const a=42;!function(){var b=a;!function(a){console.log(a++,b)}(0)}();';
  const expected = ['0 42'];
  run(code, expected);
});

test('issue_1814_2', () => {
  const code = 'const a="32";!function(){var b=a+1;!function(a){console.log(b,a++)}(0)}();';
  const expected = ['321 0'];
  run(code, expected);
});

test('try_abort', () => {
  const code = '!function(){try{var a=1;throw"";var b=2}catch(e){}console.log(a,b)}();';
  const expected = ['1 undefined'];
  run(code, expected);
});

test('boolean_binary_assign', () => {
  const code = '!function(){var a;void 0&&(a=1);console.log(a)}();';
  const expected = ['undefined'];
  run(code, expected);
});

test('cond_assign', () => {
  const code = '!function(){var a;void 0?a=1:0;console.log(a)}();';
  const expected = ['undefined'];
  run(code, expected);
});

test('iife_assign', () => {
  const code = '!function(){var a=1,b=0;!function(){b++;return;a=2}();console.log(a)}();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_1922_1', () => {
  const code = 'console.log(function(a){arguments[0]=2;return a}(1));';
  const expected = ['2'];
  run(code, expected);
});

// TODO: mangler does not support eval yet
test('issue_1922_2', { todo: true }, () => {
  const code = 'console.log(function(){var a;eval("a = 1");return a}(1));';
  const expected = ['1'];
  run(code, expected);
});

test('accessor_1', () => {
  const code = 'var a=1;console.log({get a(){a=2;return a},b:1}.b,a);';
  const expected = ['1 1'];
  run(code, expected);
});

test('accessor_2', () => {
  const code = 'var A=1;var B={get c(){console.log(A)}};B.c;';
  const expected = ['1'];
  run(code, expected);
});

test('method_1', () => {
  const code = 'var a=1;console.log((new class{a(){a=2;return a}}).a(),a);';
  const expected = ['2 2'];
  run(code, expected);
});

test('method_2', () => {
  const code = 'var A=1;var B=class{c(){console.log(A)}};(new B).c();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2090_1', () => {
  const code = 'console.log(function(){var x=1;[].forEach(()=>x=2);return x}());';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2090_2', () => {
  const code = 'console.log(function(){var x=1;[].forEach(()=>{x=2});return x}());';
  const expected = ['1'];
  run(code, expected);
});

test('for_in_prop', () => {
  const code = 'var a={foo:function(){for(this.b in[1,2]);}};a.foo();console.log(a.b);';
  const expected = ['1'];
  run(code, expected);
});

test('obj_var_1', () => {
  const code = 'var C=1;var obj={bar:function(){return C+C}};console.log(obj.bar());';
  const expected = ['2'];
  run(code, expected);
});

test('obj_var_2', () => {
  const code = 'var C=1;var obj={bar:function(){return C+C}};console.log(obj.bar());';
  const expected = ['2'];
  run(code, expected);
});

test('obj_arg_1', () => {
  const code = 'var C=1;function f(obj){return obj.bar()}console.log(f({bar:function(){return C+C}}));';
  const expected = ['2'];
  run(code, expected);
});

test('obj_arg_2', () => {
  const code = 'var C=1;function f(obj){return obj.bar()}console.log(f({bar:function(){return C+C}}));';
  const expected = ['2'];
  run(code, expected);
});

test('func_arg_1', () => {
  const code = 'var a=42;!function(a){console.log(a())}((function(){return a}));';
  const expected = ['42'];
  run(code, expected);
});

test('func_arg_2', () => {
  const code = 'var a=42;!function(a){console.log(a())}((function(a){return a}));';
  const expected = ['undefined'];
  run(code, expected);
});

test('obj_for_1', () => {
  const code = 'var o={a:1};for(var i=o.a--;i;i--)console.log(i);';
  const expected = ['1'];
  run(code, expected);
});

test('obj_for_2', () => {
  const code = 'var o={a:1};for(var i;i=o.a--;)console.log(i);';
  const expected = ['1'];
  run(code, expected);
});

test('array_forin_1', () => {
  const code = 'var a=[1,2,3];for(var b in a)console.log(b);';
  const expected = ['0', '1', '2'];
  run(code, expected);
});

test('array_forin_2', () => {
  const code = 'var a=[];for(var b in[1,2,3])a.push(b);console.log(a.length);';
  const expected = ['3'];
  run(code, expected);
});

test('array_forof_1', () => {
  const code = 'var a=[1,2,3];for(var b of a)console.log(b);';
  const expected = ['1', '2', '3'];
  run(code, expected);
});

test('array_forof_2', () => {
  const code = 'var a=[];for(var b of[1,2,3])a.push(b);console.log(a.length);';
  const expected = ['3'];
  run(code, expected);
});

test('const_expr_1', () => {
  const code = 'var o={a:1,b:2};o.a++;console.log(o.a,o.b);';
  const expected = ['2 2'];
  run(code, expected);
});

test('const_expr_2', () => {
  const code = 'Object.prototype.c=function(){this.a++};var o={a:1,b:2};o.c();console.log(o.a,o.b);';
  const expected = ['2 2'];
  run(code, expected);
});

test('escaped_prop_1', () => {
  const code = 'var obj={o:{a:1}};(function(o){o.a++})(obj.o);(function(o){console.log(o.a)})(obj.o);';
  const expected = ['2'];
  run(code, expected);
});

test('escaped_prop_2', () => {
  const code = 'var obj={o:{a:1}};(function(o){o.a++})(obj.o);(function(o){console.log(o.a)})(obj.o);';
  const expected = ['2'];
  run(code, expected);
});

test('escaped_prop_3', () => {
  const code = 'var a;function f(b){if(a)console.log(a===b.c);a=b.c}function g(){}function h(){f({c:g})}h();h();';
  const expected = ['true'];
  run(code, expected);
});

test('issue_2420_1', () => {
  const code =
    'function run(){var self=this;if(self.count++)self.foo();else self.bar()}var o={count:0,foo:function(){console.log("foo")},bar:function(){console.log("bar")}};run.call(o);run.call(o);';
  const expected = ['bar', 'foo'];
  run(code, expected);
});

test('issue_2420_2', () => {
  const code =
    'function f(){var that=this;if(that.bar)that.foo();else!function(that,self){console.log(this===that,self===this,that===self)}(that,this)}f.call({bar:1,foo:function(){console.log("foo",this.bar)}});f.call({});';
  const expected = ['foo 1', 'false false true'];
  run(code, expected);
});

test('issue_2420_3', () => {
  const code =
    'function f(){var that=this;if(that.bar)that.foo();else((that,self)=>{console.log(this===that,self===this,that===self)})(that,this)}f.call({bar:1,foo:function(){console.log("foo",this.bar)}});f.call({});';
  const expected = ['foo 1', 'true true true'];
  run(code, expected);
});

test('issue_2423_1', () => {
  const code = 'function c(){return 1}function p(){console.log(c())}p();p();';
  const expected = ['1', '1'];
  run(code, expected);
});

test('issue_2423_2', () => {
  const code = 'function c(){return 1}function p(){console.log(c())}p();p();';
  const expected = ['1', '1'];
  run(code, expected);
});

test('issue_2423_3', () => {
  const code = 'function c(){return 1}function p(){console.log(c())}p();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2423_4', () => {
  const code = 'function c(){return 1}function p(){console.log(c())}p();';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2423_5', () => {
  const code = 'function x(){y()}function y(){console.log(1)}function z(){function y(){console.log(2)}x()}z();z();';
  const expected = ['1', '1'];
  run(code, expected);
});

test('issue_2423_6', () => {
  const code = 'function x(){y()}function y(){console.log(1)}function z(){function y(){console.log(2)}x();y()}z();z();';
  const expected = ['1', '2', '1', '2'];
  run(code, expected);
});

test('recursive_inlining_1', () => {
  const code = '!function(){function foo(){bar()}function bar(){foo()}console.log("PASS")}();';
  const expected = ['PASS'];
  run(code, expected);
});

test('recursive_inlining_2', () => {
  const code = '!function(){function foo(){qux()}function bar(){foo()}function qux(){bar()}console.log("PASS")}();';
  const expected = ['PASS'];
  run(code, expected);
});

test('recursive_inlining_3', () => {
  const code =
    '!function(){function foo(x){console.log("foo",x);if(x)bar(x-1)}function bar(x){console.log("bar",x);if(x)qux(x-1)}function qux(x){console.log("qux",x);if(x)foo(x-1)}qux(4)}();';
  const expected = ['qux 4', 'foo 3', 'bar 2', 'qux 1', 'foo 0'];
  run(code, expected);
});

test('recursive_inlining_4', () => {
  const code =
    '!function(){function foo(x){console.log("foo",x);if(x)bar(x-1)}function bar(x){console.log("bar",x);if(x)qux(x-1)}function qux(x){console.log("qux",x);if(x)foo(x-1)}qux(4);bar(5)}();';
  const expected = ['qux 4', 'foo 3', 'bar 2', 'qux 1', 'foo 0', 'bar 5', 'qux 4', 'foo 3', 'bar 2', 'qux 1', 'foo 0'];
  run(code, expected);
});

test('recursive_inlining_5', () => {
  const code =
    '!function(){function foo(x){console.log("foo",x);if(x)bar(x-1)}function bar(x){console.log("bar",x);if(x)qux(x-1)}function qux(x){console.log("qux",x);if(x)foo(x-1)}qux(4);bar(5);foo(3)}();';
  const expected = [
    'qux 4',
    'foo 3',
    'bar 2',
    'qux 1',
    'foo 0',
    'bar 5',
    'qux 4',
    'foo 3',
    'bar 2',
    'qux 1',
    'foo 0',
    'foo 3',
    'bar 2',
    'qux 1',
    'foo 0',
  ];
  run(code, expected);
});

test('issue_2450_1', () => {
  const code = 'function f(){}function g(){return f}console.log(g()===g());';
  const expected = ['true'];
  run(code, expected);
});

test('issue_2450_2', () => {
  const code = 'function g(){function f(){}return f}console.log(g()===g());';
  const expected = ['false'];
  run(code, expected);
});

test('issue_2450_3', () => {
  const code =
    'var x=function(){function test(){return"foo"}return function b(){return[1,test]}}();console.log(x()[1]===x()[1]);';
  const expected = ['true'];
  run(code, expected);
});

test('issue_2450_4', () => {
  const code = 'var a;function f(b){console.log(a===b);a=b}function g(){}for(var i=3;--i>=0;)f(g);';
  const expected = ['false', 'true', 'true'];
  run(code, expected);
});

test('issue_2450_5', () => {
  const code = 'var a;function f(b){console.log(a===b);a=b}function g(){}[1,2,3].forEach((function(){f(g)}));';
  const expected = ['false', 'true', 'true'];
  run(code, expected);
});

test('issue_2449', () => {
  const code =
    'var a="PASS";function f(){return a}function g(){return f()}(function(){var a="FAIL";if(a==a)console.log(g())})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('perf_1', () => {
  const code =
    'function foo(x,y,z){return x<y?x*y+z:x*z-y}function indirect_foo(x,y,z){return foo(x,y,z)}var sum=0;for(var i=0;i<100;++i){sum+=indirect_foo(i,i+1,3*i)}console.log(sum);';
  const expected = ['348150'];
  run(code, expected);
});

test('perf_3', () => {
  const code =
    'var foo=function(x,y,z){return x<y?x*y+z:x*z-y};var indirect_foo=function(x,y,z){return foo(x,y,z)};var sum=0;for(var i=0;i<100;++i)sum+=indirect_foo(i,i+1,3*i);console.log(sum);';
  const expected = ['348150'];
  run(code, expected);
});

test('perf_5', () => {
  const code =
    'function indirect_foo(x,y,z){function foo(x,y,z){return x<y?x*y+z:x*z-y}return foo(x,y,z)}var sum=0;for(var i=0;i<100;++i){sum+=indirect_foo(i,i+1,3*i)}console.log(sum);';
  const expected = ['348150'];
  run(code, expected);
});

test('perf_7', () => {
  const code =
    'var indirect_foo=function(x,y,z){var foo=function(x,y,z){return x<y?x*y+z:x*z-y};return foo(x,y,z)};var sum=0;for(var i=0;i<100;++i)sum+=indirect_foo(i,i+1,3*i);console.log(sum);';
  const expected = ['348150'];
  run(code, expected);
});

test('issue_2485', () => {
  const code =
    'var foo=function(bar){var n=function(a,b){return a+b};var sumAll=function(arg){return arg.reduce(n,0)};var runSumAll=function(arg){return sumAll(arg)};bar.baz=function(arg){var n=runSumAll(arg);return n.get=1,n};return bar};var bar=foo({});console.log(bar.baz([1,2,3]));';
  const expected = ['6'];
  run(code, expected);
});

test('issue_2496', () => {
  const code =
    'function execute(callback){callback()}class Foo{constructor(message){this.message=message}go(){this.message="PASS";console.log(this.message)}run(){execute(()=>{this.go()})}}new Foo("FAIL").run();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2416', () => {
  const code = 'class Foo{}console.log(Foo.name);';
  const expected = ['Foo'];
  run(code, expected);
});

test('escape_conditional', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("FAIL");else console.log("PASS")}function baz(s){return s?foo:bar}function foo(){}function bar(){}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_throw', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("FAIL");else console.log("PASS")}function baz(){try{throw foo}catch(bar){return bar}}function foo(){}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_local_conditional', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("PASS");else console.log("FAIL")}function baz(s){function foo(){}function bar(){}return s?foo:bar}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_local_sequence', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("PASS");else console.log("FAIL")}function baz(){function foo(){}function bar(){}return foo,bar}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_local_throw', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("PASS");else console.log("FAIL")}function baz(){function foo(){}try{throw foo}catch(bar){return bar}}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_yield', () => {
  const code =
    'function main(){var thing=gen.next().value;if(thing!==(thing=gen.next().value))console.log("FAIL");else console.log("PASS")}function foo(){}function*baz(s){for(;;)yield foo}var gen=baz();main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('escape_expansion', () => {
  const code =
    'function main(){var thing=baz();if(thing!==(thing=baz()))console.log("FAIL");else console.log("PASS")}function foo(){}function bar(...x){return x[0]}function baz(){return bar(...[foo])}main();';
  const expected = ['PASS'];
  run(code, expected);
});

test('defun_single_use_loop', () => {
  const code = 'for(var x,i=2;--i>=0;){var y=x;x=f;console.log(x===y)}function f(){}';
  const expected = ['false', 'true'];
  run(code, expected);
});

test('do_while', () => {
  const code = 'function f(a){do{(function(){a&&(c="PASS")})()}while(a=0)}var c="FAIL";f(1);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2598', () => {
  const code = 'function f(){}function g(a){return a||f}console.log(g(false)===g(null));';
  const expected = ['true'];
  run(code, expected);
});

test('issue_2669', () => {
  const code = 'let foo;console.log(([foo]=["PASS"])&&foo);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2670', () => {
  const code = 'const obj={};obj.prop="PASS";const{prop:value}=obj;console.log(value);';
  const expected = ['PASS'];
  run(code, expected);
});

test('defun_assign', () => {
  const code = 'console.log(typeof a);a=42;console.log(typeof a);function a(){}console.log(typeof a);';
  const expected = ['function', 'number', 'number'];
  run(code, expected);
});

test('defun_var_1', () => {
  const code = 'var a=42,b;function a(){}function b(){}console.log(typeof a,typeof b);';
  const expected = ['number function'];
  run(code, expected);
});

test('defun_var_2', () => {
  const code = 'function a(){}function b(){}var a=42,b;console.log(typeof a,typeof b);';
  const expected = ['number function'];
  run(code, expected);
});

test('defun_var_3', () => {
  const code = 'function a(){}function b(){}console.log(typeof a,typeof b);var a=42,b;';
  const expected = ['function function'];
  run(code, expected);
});

test('defun_catch_1', () => {
  const code = 'function a(){}try{throw 42}catch(a){console.log(a)}';
  const expected = ['42'];
  run(code, expected);
});

test('defun_catch_2', () => {
  const code = 'try{function a(){}throw 42}catch(a){console.log(a)}';
  const expected = ['42'];
  run(code, expected);
});

test('defun_catch_3', () => {
  const code = 'try{throw 42;function a(){}}catch(a){console.log(a)}';
  const expected = ['42'];
  run(code, expected);
});

test('defun_catch_6', () => {
  const code = 'try{throw 42}catch(a){console.log(a)}function a(){}';
  const expected = ['42'];
  run(code, expected);
});

test('duplicate_lambda_defun_name_1', () => {
  const code = 'console.log(function f(a){function f(){}return f.length}());';
  const expected = ['0'];
  run(code, expected);
});

test('duplicate_lambda_defun_name_2', () => {
  const code = 'console.log(function f(a){function f(){}return f.length}());';
  const expected = ['0'];
  run(code, expected);
});

test('issue_2757_1', () => {
  const code = 'let u;(function(){let v;console.log(u,v)})();';
  const expected = ['undefined undefined'];
  run(code, expected);
});

test('issue_2757_2', () => {
  const code = '(function(){let bar;const unused=function(){bar=true};if(!bar){console.log(1)}console.log(2)})();';
  const expected = ['1', '2'];
  run(code, expected);
});

test('issue_2774', () => {
  const code = 'console.log({get a(){var b;(b=true)&&b.c;b=void 0}}.a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2799_1', () => {
  const code =
    'console.log(function(){return f;function f(n){function g(i){return i&&i+g(i-1)}function h(j){return g(j)}return h(n)}}()(5));';
  const expected = ['15'];
  run(code, expected);
});

test('issue_2799_2', () => {
  const code = '(function(){function foo(){Function.prototype.call.apply(console.log,[null,"PASS"])}foo()})();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2836', () => {
  const code = 'function f(){return"FAIL"}console.log(f());function f(){return"PASS"}';
  const expected = ['PASS'];
  run(code, expected);
});

test('lvalues_def_1', () => {
  const code = 'var b=1;var a=b++,b=NaN;console.log(a,b);';
  const expected = ['1 NaN'];
  run(code, expected);
});

test('lvalues_def_2', () => {
  const code = 'var b=1;var a=b+=1,b=NaN;console.log(a,b);';
  const expected = ['2 NaN'];
  run(code, expected);
});

test('chained_assignments', () => {
  const code =
    'function f(){var a=[94,173,190,239];var b=0;b|=a[0];b<<=8;b|=a[1];b<<=8;b|=a[2];b<<=8;b|=a[3];return b}console.log(f().toString(16));';
  const expected = ['5eadbeef'];
  run(code, expected);
});

test('issue_2860_1', () => {
  const code = 'console.log(function(a){return a^=1;a^=2}());';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2860_2', () => {
  const code = 'console.log(function(a){return a^=1;a^=2}());';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2869', () => {
  const code = 'var c="FAIL";(function f(a){var a;if(!f)a=0;if(a)c="PASS"})(1);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3042_1', () => {
  const code =
    'function f(){}var a=[1,2].map((function(){return new f}));console.log(a[0].constructor===a[1].constructor);';
  const expected = ['true'];
  run(code, expected);
});

test('issue_3042_2', () => {
  const code =
    'function Foo(){this.isFoo=function(o){return o instanceof Foo}}function FooCollection(){this.foos=[1,1].map((function(){return new Foo}))}var fooCollection=new FooCollection;console.log(fooCollection.foos[0].isFoo(fooCollection.foos[0]));console.log(fooCollection.foos[0].isFoo(fooCollection.foos[1]));console.log(fooCollection.foos[1].isFoo(fooCollection.foos[0]));console.log(fooCollection.foos[1].isFoo(fooCollection.foos[1]));';
  const expected = ['true', 'true', 'true', 'true'];
  run(code, expected);
});

test('issue_2919', () => {
  const code = 'var arr=[function(){}];console.log(typeof arr[0]);';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2916', () => {
  const code = 'var c="FAIL";(function(b){(function(d){d[0]=1})(b);+b&&(c="PASS")})([]);console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3125', () => {
  const code = 'var o;console.log((function(){this.p++}.call(o={p:6}),o.p));';
  const expected = ['7'];
  run(code, expected);
});

test('issue_2992', () => {
  const code = 'var c="PASS";(function f(b){switch(0){case 0:case b=1:b&&(c="FAIL")}})();console.log(c);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3110_1', () => {
  const code =
    '(function(){function foo(){return isDev?"foo":"bar"}var isDev=true;var obj={foo:foo};console.log(foo());console.log(obj.foo())})();';
  const expected = ['foo', 'foo'];
  run(code, expected);
});

test('issue_3110_2', () => {
  const code =
    '(function(){function foo(){return isDev?"foo":"bar"}var isDev=true;console.log(foo());var obj={foo:foo};console.log(obj.foo())})();';
  const expected = ['foo', 'foo'];
  run(code, expected);
});

test('issue_3110_3', () => {
  const code =
    '(function(){function foo(){return isDev?"foo":"bar"}console.log(foo());var isDev=true;var obj={foo:foo};console.log(obj.foo())})();';
  const expected = ['bar', 'foo'];
  run(code, expected);
});

test('issue_3113_1', () => {
  const code =
    'var c=0;(function(){function f(){while(g());}var a=f();function g(){a&&a[c++]}g(a=1)})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3113_2', () => {
  const code =
    'var c=0;(function(){function f(){while(g());}var a=f();function g(){a&&a[c++]}a=1;g()})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3113_3', () => {
  const code = 'var c=0;(function(){function f(){while(g());}var a;function g(){a&&a[c++]}g(a=1)})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_3113_4', () => {
  const code = 'var a=0,b=0;function f(){b+=a}f(f(),++a);console.log(a,b);';
  const expected = ['1 1'];
  run(code, expected);
});

test('issue_3113_5', () => {
  const code = 'function f(){console.log(a)}function g(){f()}while(g());var a=1;f();';
  const expected = ['undefined', '1'];
  run(code, expected);
});

test('conditional_nested_1', () => {
  const code =
    'var a=1,b=0;(function f(c){function g(){c&&(c.a=0);c&&(c.a=0);c&&(c[b++]*=0)}g(a--&&f(g(c=42)))})();console.log(b);';
  const expected = ['2'];
  run(code, expected);
});

test('conditional_nested_2', () => {
  const code = 'var c=0;(function(a){function f(){a&&c++}f(!c&&f(),a=1)})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('conditional_nested_3', () => {
  const code = 'var n=2,c=0;(function f(a){0<n--&&g(a=1);function g(){a&&c++}g();0<n--&&f()})();console.log(c);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_3140_1', () => {
  const code =
    '(function(){var a;function f(){}f.g=function g(){function h(){console.log(a?"PASS":"FAIL")}a=true;this();a=false;h.g=g;return h};return f})().g().g();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3140_2', () => {
  const code =
    '(function(){var a;function f(){}f.g=function g(){var self=this;function h(){console.log(a?"PASS":"FAIL")}a=true;self();a=false;h.g=g;return h};return f})().g().g();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3140_3', () => {
  const code =
    '(function(){var a;function f(){}f.g=function g(){var self=this;function h(){console.log(a?"PASS":"FAIL")}a=true;(function(){return self})()();a=false;h.g=g;return h};return f})().g().g();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3140_4', () => {
  const code =
    '(function(){var a;function f(){}f.g=function g(){var o={p:this};function h(){console.log(a?"PASS":"FAIL")}a=true;o.p();a=false;h.g=g;return h};return f})().g().g();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_3140_5', () => {
  const code =
    'var n=1,c=0;(function(a){var b=function(){this;n--&&h()}();function h(){b&&c++}h(b=1)})();console.log(c);';
  const expected = ['1'];
  run(code, expected);
});

test('reduce_funcs_in_array_1', () => {
  const code =
    '(function(){function Foo(){return 123}function bar(){return[Foo].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],a[0][0]())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('reduce_funcs_in_array_2', () => {
  const code =
    '(function(){function Foo(){return 123}function bar(val){return[val||Foo].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],a[0][0]())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('reduce_funcs_in_object_literal_1', () => {
  const code =
    '(function(){function Foo(){return 123}function bar(){return[{prop:Foo}.prop].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],a[0][0]())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('reduce_funcs_in_object_literal_2', () => {
  const code =
    '(function(){function Foo(){return 123}function bar(val){return[val||Foo].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],a[0][0]())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('single_use_class_referenced_in_array', () => {
  const code =
    '(function(){class Foo{data(){return 123}}function bar(val){return[val||Foo].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],(new a[0][0]).data())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('single_use_class_referenced_in_object_literal', () => {
  const code =
    '(function(){class Foo{data(){return 123}}function bar(val){return[{prop:val||Foo}.prop].concat([2])}var a=[bar(),bar()];console.log(a[0][0]===a[1][0],(new a[0][0]).data())})();';
  const expected = ['true 123'];
  run(code, expected);
});

test('issue_369', () => {
  const code =
    'var printTest=function(ret){function ret(){console.log("Value after override")}return ret}("Value before override");printTest();';
  const expected = ['Value after override'];
  run(code, expected);
});

test('issue_432_1', () => {
  const code =
    'const selectServer=()=>{selectServers()};function selectServers(){const retrySelection=()=>{var descriptionChangedHandler=()=>{selectServers()}};retrySelection()}leak(()=>Topology);console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_432_2', () => {
  const code =
    'const selectServer=()=>{selectServers()};function selectServers(){function retrySelection(){var descriptionChangedHandler=()=>{selectServers()};leak(descriptionChangedHandler)}retrySelection()}leak(()=>Topology);console.log("PASS");';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_443', () => {
  const code =
    'const one_name="PASS";var get_one=()=>{if(one_name)return one_name};{let one_name=get_one();console.log(one_name)}';
  const expected = ['PASS'];
  run(code, expected);
});

test('reduce_class_with_side_effects_in_extends', () => {
  const code =
    'let x="";class Y extends(x+="PA",Array){}class X extends(x+="SS",Array){}global.something=[new X,new Y];console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('reduce_class_with_side_effects_in_properties', () => {
  const code =
    'let x="";class Y{static _=x+="PA"}class X{static _=x+="SS"}global.something=[new X,new Y];console.log(x);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_581', () => {
  const code =
    'class Yellow{method(){const errorMessage="FAIL";return applyCb(errorMessage,()=>console.log(this.message()))}message(){return"PASS"}}function applyCb(errorMessage,callback){return callback(errorMessage)}(new Yellow).method();';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_581_2', () => {
  const code =
    '(function(){return function(callback){return callback()}(()=>{console.log(this.message)})}).call({message:"PASS"});';
  const expected = ['PASS'];
  run(code, expected);
});

test('regexp_1', () => {
  const code = 'console.log(JSON.stringify("COMPASS? Overpass.".match(/([Sap]+)/gi)));';
  const expected = ['["PASS","pass"]'];
  run(code, expected);
});

test('regexp_2', () => {
  const code = 'console.log(JSON.stringify("COMPASS? Overpass.".match(new RegExp("([Sap]+)","ig"))));';
  const expected = ['["PASS","pass"]'];
  run(code, expected);
});

test('mangle_catch', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_ie8', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_ie8', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_ie8_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_var_ie8_toplevel', () => {
  const code = 'var a="FAIL";try{throw 1}catch(args){var a="PASS"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_ie8', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_toplevel', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_1_ie8_toplevel', () => {
  const code = 'var a="PASS";try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('mangle_catch_redef_2', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_ie8', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_toplevel', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('mangle_catch_redef_2_ie8_toplevel', () => {
  const code = 'try{throw"FAIL1"}catch(a){var a="FAIL2"}console.log(a);';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2120_1', () => {
  const code = '"aaaaaaaa";var a=1,b="FAIL";try{throw 1}catch(c){try{throw 0}catch(a){if(c)b="PASS"}}console.log(b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_2120_2', () => {
  const code = '"aaaaaaaa";var a=1,b="FAIL";try{throw 1}catch(c){try{throw 0}catch(a){if(c)b="PASS"}}console.log(b);';
  const expected = ['PASS'];
  run(code, expected);
});

test('function_iife_catch', () => {
  const code = 'function f(n){!function(){try{throw 0}catch(n){var a=1;console.log(n,a)}}()}f();';
  const expected = ['0 1'];
  run(code, expected);
});

test('function_iife_catch_ie8', () => {
  const code = 'function f(n){!function(){try{throw 0}catch(n){var a=1;console.log(n,a)}}()}f();';
  const expected = ['0 1'];
  run(code, expected);
});

test('function_catch_catch', () => {
  const code =
    'var o=0;function f(){try{throw 1}catch(c){try{throw 2}catch(o){var o=3;console.log(o)}}console.log(o)}f();';
  const expected = ['3', 'undefined'];
  run(code, expected);
});

test('function_catch_catch_ie8', () => {
  const code =
    'var o=0;function f(){try{throw 1}catch(c){try{throw 2}catch(o){var o=3;console.log(o)}}console.log(o)}f();';
  const expected = ['3', 'undefined'];
  run(code, expected);
});

test('console_log', () => {
  const code = 'console.log("%% %s");console.log("%% %s","%s");';
  const expected = ['%% %s', '%% %s %s'];
  run(code, expected);
});

test('lift_sequences_5', () => {
  const code = 'var a=2,b;a*=(b,a=4,3);console.log(a);';
  const expected = ['6'];
  run(code, expected);
});

test('func_def_1', () => {
  const code = 'function f(){return f=0,!!f}console.log(f());';
  const expected = ['false'];
  run(code, expected);
});

test('func_def_2', () => {
  const code = 'console.log(function f(){return f=0,!!f}());';
  const expected = ['true'];
  run(code, expected);
});

test('func_def_3', () => {
  const code = 'function f(){function g(){}return g=0,!!g}console.log(f());';
  const expected = ['false'];
  run(code, expected);
});

test('func_def_4', () => {
  const code = 'function f(){function g(){return g=0,!!g}return g()}console.log(f());';
  const expected = ['false'];
  run(code, expected);
});

test('func_def_5', () => {
  const code = 'function f(){return function g(){return g=0,!!g}()}console.log(f());';
  const expected = ['true'];
  run(code, expected);
});

test('issue_1758', () => {
  const code = 'console.log(function(c){var undefined=42;return function(){c--;c--,c.toString();return}()}());';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2062', () => {
  const code = 'var a=1;if([a||a+++a--,a+++a--,a&&a.var]);console.log(a);';
  const expected = ['1'];
  run(code, expected);
});

test('issue_2313', () => {
  const code =
    'var a=0,b=0;var foo={get c(){a++;return 42},set c(c){b++},d:function(){this.c++;if(this.c)console.log(a,b)}};foo.d();';
  const expected = ['2 1'];
  run(code, expected);
});

test('for_init_var', () => {
  const code = 'var a="PASS";(function(){var b=42;for(var c=5;c>0;)c--;a="FAIL";var a})();console.log(a);';
  const expected = ['PASS'];
  run(code, expected);
});

test('forin', () => {
  const code = 'var o=[];o.push("PASS");for(var a in o)console.log(o[a]);';
  const expected = ['PASS'];
  run(code, expected);
});

test('call', () => {
  const code =
    'var a=function(){return this}();function b(){console.log("foo")}b.c=function(){console.log(this===b?"bar":"baz")};(a,b)();(a,b.c)();(a,function(){console.log(this===a)})();new(a,b);new(a,b.c);new(a,function(){console.log(this===a)});';
  const expected = ['foo', 'baz', 'true', 'foo', 'baz', 'false'];
  run(code, expected);
});

test('issue_1929', () => {
  const code =
    'function f(s){return s.split(/[\\\\/]/)}var r=f("A/B\\\\C\\\\D/E\\\\F");console.log(r[5],r[4],r[3],r[2],r[1],r[0],r.length);';
  const expected = ['F E D C B A 6'];
  run(code, expected);
});

test('issue_1674', () => {
  const code = 'switch(0){default:console.log("FAIL");break;case 0:console.log("PASS");break}';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1680_1', () => {
  const code =
    'function f(x){console.log(x);return x+1}switch(2){case f(0):case f(1):f(2);case 2:case f(3):case f(4):f(5)}';
  const expected = ['0', '1', '2', '5'];
  run(code, expected);
});

test('issue_1690_1', () => {
  const code = 'switch(console.log("PASS")){}';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1690_2', () => {
  const code = 'switch(console.log("PASS")){}';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_1698', () => {
  const code = 'var a=1;!function(){switch(a++){}}();console.log(a);';
  const expected = ['2'];
  run(code, expected);
});

test('issue_1758', () => {
  const code = 'var a=1,b=2;switch(a--){default:b++}console.log(a,b);';
  const expected = ['0 3'];
  run(code, expected);
});

test('issue_1750', () => {
  const code = 'var a=0,b=1;switch(true){case a,true:default:b=2;case true:}console.log(a,b);';
  const expected = ['0 2'];
  run(code, expected);
});

test('issue_445', () => {
  const code =
    'const leak=()=>{};function scan(){let len=leak();let ch=0;switch(ch=123){case"never-reached":const ch=leak();leak(ch)}return len===123?"FAIL":"PASS"}console.log(scan());';
  const expected = ['PASS'];
  run(code, expected);
});

test('simple_string', () => {
  const code = 'console.log(`world`,{[`foo`]:1}[`foo`],`hi`=="hi");';
  const expected = ['world 1 true'];
  run(code, expected);
});

test('regex_1', () => {
  const code = 'console.log(`${/a/} ${6/2} ${/b/.test("b")} ${1?/c/:/d/}`);';
  const expected = ['/a/ 3 true /c/'];
  run(code, expected);
});

test('regex_2', () => {
  const code = 'console.log(`${/a/} ${6/2} ${/b/.test("b")} ${1?/c/:/d/}`);';
  const expected = ['/a/ 3 true /c/'];
  run(code, expected);
});

test('sequence_1', () => {
  const code = 'console.log(`${1,2} ${/a/,/b/}`);';
  const expected = ['2 /b/'];
  run(code, expected);
});

test('sequence_2', () => {
  const code = 'console.log(`${1,2} ${/a/,/b/}`);';
  const expected = ['2 /b/'];
  run(code, expected);
});

test('return_template_string_with_trailing_backslash', () => {
  const code =
    'function a(){return`foo`}function b(){return`\\nbar`}function c(){return;`baz`}function d(){return;`qux`}function e(){return`\\nfin`}console.log(a(),b(),c(),d(),e());';
  const expected = ['foo \nbar undefined undefined \nfin'];
  run(code, expected);
});

test('tagged_template_with_invalid_escape', () => {
  const code = 'function x(s){return s.raw[0]}console.log(String.raw`\\u`);console.log(x`\\u`);';
  const expected = ['\\u', '\\u'];
  run(code, expected);
});

test('tagged_call_with_invalid_escape_2', () => {
  const code =
    'var x={y:()=>String.raw};console.log(x.y()`\\4321\\u\\x`);let z=()=>String.raw;console.log(z()`\\4321\\u\\x`);';
  const expected = ['\\4321\\u\\x', '\\4321\\u\\x'];
  run(code, expected);
});

test('es2018_revision_of_template_escapes_1', () => {
  const code = 'console.log(String.raw`\\unicode \\xerces \\1234567890`);';
  const expected = ['\\unicode \\xerces \\1234567890'];
  run(code, expected);
});

test('tagged_call_with_invalid_escape', () => {
  const code = 'let z=()=>String.raw;console.log(z()`\\4321\\u\\x`);';
  const expected = ['\\4321\\u\\x'];
  run(code, expected);
});

test('tagged_template_with_ill_formed_unicode_escape', () => {
  const code = 'console.log(String.raw`\\u{-1}`);';
  const expected = ['\\u{-1}'];
  run(code, expected);
});

test('tagged_template_with_comment', () => {
  const code = 'console.log(String.raw`\\u`);console.log((()=>String.raw)()`\\x`);';
  const expected = ['\\u', '\\x'];
  run(code, expected);
});

test('tagged_template_valid_strict_legacy_octal', () => {
  const code = '"use strict";console.log(String.raw`\\u\\x\\567`);';
  const expected = ['\\u\\x\\567'];
  run(code, expected);
});

test('broken_safari_catch_scope', () => {
  const code = '"AAAAAAAA";"BBBBBBB";(new class{f(x){try{throw{m:"PASS"}}catch({m:s}){console.log(s)}}}).f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('broken_safari_catch_scope_caveat', () => {
  const code = '"AAAAAAAA";"BBBBBBB";(new class{f(x){try{throw{m:"PASS"}}catch({m:x}){console.log(x)}}}).f();';
  const expected = ['PASS'];
  run(code, expected);
});

test('parameterless_catch', () => {
  const code = 'try{unknown()}catch{console.log("PASS")}';
  const expected = ['PASS'];
  run(code, expected);
});

test('parent_scope_of_catch_block_is_not_the_try_block', () => {
  const code =
    'function test(foo,bar){try{const bar={};throw"PASS"}catch(error){return bar(error)}}console.log(test(null,x=>x));';
  const expected = ['PASS'];
  run(code, expected);
});

test('issue_452', () => {
  const code = 'try{const arr=["PASS"];for(const x of arr){console.log(x)}}catch(e){}';
  const expected = ['PASS'];
  run(code, expected);
});

test('typeof_defun_1', () => {
  const code =
    'function f(){console.log("YES")}function g(){h=42;console.log("NOPE")}function h(){console.log("YUP")}g=42;"function"==typeof f&&f();"function"==typeof g&&g();"function"==typeof h&&h();';
  const expected = ['YES', 'YUP'];
  run(code, expected);
});

test('typeof_defun_2', () => {
  const code =
    'var f=function(){console.log(x)};var x=0;x++<2&&typeof f=="function"&&f();x++<2&&typeof f=="function"&&f();x++<2&&typeof f=="function"&&f();';
  const expected = ['1', '2'];
  run(code, expected);
});

test('duplicate_defun_arg_name', () => {
  const code = 'function long_name(long_name){return typeof long_name}console.log(typeof long_name,long_name());';
  const expected = ['function undefined'];
  run(code, expected);
});

test('duplicate_lambda_arg_name', () => {
  const code = 'console.log(function long_name(long_name){return typeof long_name}());';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2728_1', () => {
  const code = '(function arguments(){console.log(typeof arguments)})();';
  const expected = ['object'];
  run(code, expected);
});

test('issue_2728_2', () => {
  const code = 'function arguments(){return typeof arguments}console.log(typeof arguments,arguments());';
  const expected = ['function object'];
  run(code, expected);
});

test('issue_2728_3', () => {
  const code = '(function(){function arguments(){}console.log(typeof arguments)})();';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2728_4', () => {
  const code = 'function arguments(){}console.log(typeof arguments);';
  const expected = ['function'];
  run(code, expected);
});

test('issue_2728_5', () => {
  const code = '(function arguments(arguments){console.log(typeof arguments)})();';
  const expected = ['undefined'];
  run(code, expected);
});

test('issue_2728_6', () => {
  const code = 'function arguments(arguments){return typeof arguments}console.log(typeof arguments,arguments());';
  const expected = ['function undefined'];
  run(code, expected);
});

test('issue_3271', () => {
  const code =
    'function string2buf(str){var i=0,buf=new Array(2),c=str.charCodeAt(0);if(c<2048){buf[i++]=192|c>>>6;buf[i++]=128|c&63}else{buf[i++]=224|c>>>12;buf[i++]=128|c>>>6&63;buf[i++]=128|c&63}return buf}console.log(string2buf("é"));';
  const expected = ['[195,169]'];
  run(code, expected);
});

test('yield_as_ES5_property', () => {
  const code = '"use strict";console.log({yield:42}.yield);';
  const expected = ['42'];
  run(code, expected);
});

test('issue_2832', () => {
  const code =
    'function*gen(i){const result=yield(x=i,-x);var x;console.log(x);console.log(result);yield 2}var x=gen(1);console.log(x.next("first").value);console.log(x.next("second").value);';
  const expected = ['-1', '1', 'second', '2'];
  run(code, expected);
});

test('issue_t60', () => {
  const code = 'function*t(){const v=yield 1;yield 2;return v}var g=t();console.log(g.next().value,g.next().value);';
  const expected = ['1 2'];
  run(code, expected);
});
