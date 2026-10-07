pub fn inputs() -> [(&'static str, String); 7] {
    const STATEMENTS: usize = 1024;
    const STATEMENT: &str = "value = left + right;\n";
    const DEPTH: usize = 128;

    let comment_free = STATEMENT.repeat(STATEMENTS);
    let mut sparse = String::with_capacity(comment_free.len());
    for index in 0..STATEMENTS {
        if index % 128 == 0 {
            sparse.push_str("/* statement */\n");
        }
        sparse.push_str(STATEMENT);
    }
    let dense =
        "/* statement */ value = /* left */ left + /* right */ right; // tail\n".repeat(STATEMENTS);

    let mut nested = "{\n".repeat(DEPTH);
    nested.push_str("/* leaf */ value = /* operand */ left + right; // tail\n");
    nested.push_str(&"}\n".repeat(DEPTH));

    let run = "/* comment in a run */\n".repeat(512);
    let long_runs = format!("{run}{STATEMENT}{run}");

    let templates = concat!(
        "tag`head${/* before */ left /* after */}",
        "middle${/* before */ right /* after */}",
        "middle${/* before */ left + right /* after */}",
        "tail${/* before */ fn(value) /* after */}end`;\n",
    )
    .repeat(128);

    let function_bodies = format!(
        "/* first */ function first() {{ {} }} /* second */ function second() {{ {} }}",
        STATEMENT.repeat(STATEMENTS / 2),
        STATEMENT.repeat(STATEMENTS / 2),
    );

    [
        ("comment_free", comment_free),
        ("sparse", sparse),
        ("dense", dense),
        ("deeply_nested", nested),
        ("long_comment_runs", long_runs),
        ("template_substitutions", templates),
        ("function_bodies", function_bodies),
    ]
}
