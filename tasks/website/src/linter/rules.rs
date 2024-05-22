// <https://oxc-project.github.io/docs/guide/usage/linter/rules.html>
pub fn generate_rules() {
    use oxc_linter::Linter;
    let mut v = vec![];
    Linter::print_rules(&mut v);
    println!("{}", String::from_utf8(v).unwrap());
}
