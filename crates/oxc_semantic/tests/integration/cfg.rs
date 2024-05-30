use std::fs;

use oxc_span::SourceType;

use crate::util::SemanticTester;

#[test]
fn test_cfg_files() {
    insta::glob!("cfg_fixtures/*.js", |path| {
        let code = fs::read_to_string(path).unwrap();
        let output = SemanticTester::new(&code, SourceType::from_path(path).unwrap());
        insta::assert_snapshot!(output.basic_blocks_printed());
        insta::assert_snapshot!(output.cfg_dot_diagram());
    });
}

// todo: private identifier from class function name should not be in cfg

// #[test]
// fn test_cfg_expected_basic_block_count() {
//     for test in [
//         ("(a => {}).length", 2),
//         ("typeof (() => {});", 2),
//         ("export default (() => {})();", 2),
//         ("(() => {})()``;", 2),
//         ("(() => {})``;", 2),
//         ("new (() => {});", 2),
//         (
//             "
//         // 1 - global scope
//         if (
//             (() => {}) // 2 - inside arrow function
//             ? 1 // 3 - consequent of conditional expression
//             : 0 // 4 - alternate
//             // 5 - after arrow expression
//         ) {
//             // 6 - inside consequent of if statement
//         }
//         // 7 - after if statement
//         ",
//             7,
//         ),
//         ("let f = () => ({}())", 2),
//         ("let a = () => ({} instanceof a);", 2),
//         ("a = () => ({} && a);", 5),
//         ("a = () => ({}() && a);", 5),
//         ("a = () => ({} && a && b);", 8),
//         ("a = () => ({} + a);", 2),
//         ("a = () => ({}()() && a);", 5),
//         ("a = () => ({}.b && a);", 5),
//         ("a = () => ({}[b] && a);", 5),
//         (" a = () => ({}`` && a);", 5),
//         ("a = () => ({} = 0);", 2),
//         ("a = () => ({}, a);", 2),
//         ("a => a instanceof {};", 2),
//         ("a => ({}().b && 0)", 5),
//         ("a => ({}().c = 0)", 2),
//         ("x => ({}()())", 2),
//         ("x => ({}()``)", 2),
//         ("x => ({}().b);", 2),
//         ("a = b => c;", 2),
//         ("x => (y = z);", 2),
//         ("x => (y += z);", 2),
//         ("f(a => ({})) + 1;", 2),
//         ("(a => ({})) || 0;", 5),
//         ("a = b => c;", 2),
//         (
//             "a = b => {
//                 return c
//             };",
//             3,
//         ),
//     ] {
//         let count = SemanticTester::js(test.0).basic_blocks_count();

//         assert_eq!(
//             test.1, count,
//             "The code: \n{}\nshould have had {} basic blocks\nhowever actually had {} basic blocks.",
//             test.0, test.1, count
//         );
//     }
// }
