use oxc_transform_conformance::babel;

fn main() {
    let names = ["babel-plugin-transform-optional-catch-binding"];
    for name in names {
        babel(name);
        println!("Passed: {name}");
    }
}
