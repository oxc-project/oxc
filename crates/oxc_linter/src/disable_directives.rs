pub use oxc_linter_core::disable_directives::*;

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::EslintNoDebugger, tester::Tester};

    for prefix in ["eslint", "oxlint"] {
        // [Disabling Rules](https://eslint.org/docs/latest/use/configure/rules#disabling-rules)
        // Using configuration comments
        let pass = vec![
            // To disable rule warnings in a part of a file, use block comments in the following format:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        /* {prefix}-enable */
        "
            ),
            // You can also disable or enable warnings for specific rules:
            format!(
                "
        /* {prefix}-disable no-debugger, no-console */
            debugger;
        /* {prefix}-enable no-debugger, no-console */
        "
            ),
            // To disable rule warnings in an entire file, put a /* eslint-disable */ block comment at the top of the file:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        "
            ),
            // You can also disable or enable specific rules for an entire file:
            format!(
                "
        /* {prefix}-disable no-debugger */
            debugger;
        "
            ),
            // To ensure that a rule is never applied (regardless of any future enable/disable lines):
            // This is not supported.
            // "
            // /* eslint no-debugger: \"off\" */
            //     debugger;
            // "),
            // To disable all rules on a specific line, use a line or block comment in one of the following formats:
            format!(
                "debugger; // {prefix}-disable-line
            debugger; // {prefix}-disable-line

            // {prefix}-disable-next-line
            debugger;

            /* {prefix}-disable-next-line */
            debugger;

            debugger; /* {prefix}-disable-line */
        "
            ),
            // To disable a specific rule on a specific line:
            format!(
                "
            debugger; // {prefix}-disable-line no-debugger

            // {prefix}-disable-next-line no-debugger
            debugger;

            debugger; /* {prefix}-disable-line no-debugger */

            /* {prefix}-disable-next-line no-debugger */
            debugger;
        "
            ),
            // To disable multiple rules on a specific line:
            format!(
                "
            debugger; // {prefix}-disable-line no-debugger, quotes, semi

            // {prefix}-disable-next-line no-debugger, quotes, semi
            debugger;

            debugger; /* {prefix}-disable-line no-debugger, quotes, semi */

            /* {prefix}-disable-next-line no-debugger, quotes, semi */
            debugger;

            /* {prefix}-disable-next-line
              no-debugger,
              quotes,
              semi
            */
            debugger;
        "
            ),
            // To disable all rules twice:
            format!(
                "
        /* {prefix}-disable */
            debugger;
        /* {prefix}-disable */
            debugger;
        "
            ),
            // To disable a rule twice:
            format!(
                "
        /* {prefix}-disable no-debugger */
            debugger;
        /* {prefix}-disable no-debugger */
            debugger;
        "
            ),
            // Comment descriptions
            format!(
                "
            // {prefix}-disable-next-line no-debugger -- Here's a description about why this configuration is necessary.
            debugger;

            /* {prefix}-disable-next-line no-debugger --
             * Here's a very long description about why this configuration is necessary
             * along with some additional information
            **/
            debugger;
        "
            ),
            // Single-hyphen descriptions and whitespace-separated rule lists.
            format!(
                "
            // {prefix}-disable-next-line no-debugger - Here's a description about why this configuration is necessary.
            debugger;

            // {prefix}-disable-next-line no-alert no-debugger - Here's a description about why this configuration is necessary.
            debugger;
        "
            ),
            // Should only match `eslint-enable` comments, not `eslint-enablefoo`
            format!("
            /* {prefix}-disable */
                debugger;
            /* {prefix}-enablefoo */
                debugger;
            "
            ),
            format!("
            /* {prefix}-disable no-debugger, no-console */
                debugger;
            /* {prefix}-enablefoo no-debugger, no-console */
                debugger;
            "
            ),
            // Handles no spaces in comment
            format!(
                "debugger; //{prefix}-disable-line
            debugger; //{prefix}-disable-line

            //{prefix}-disable-next-line
            debugger;

            /*{prefix}-disable-next-line*/
            debugger;

            debugger; /*{prefix}-disable-line*/

            debugger; //{prefix}-disable-line no-debugger

            //{prefix}-disable-next-line no-debugger
            debugger;

            debugger; /*{prefix}-disable-line no-debugger*/

            /*{prefix}-disable-next-line no-debugger*/
            debugger;
        "
            ),
            // Handles extra spaces in comment
            format!(
                "debugger; //       {prefix}-disable-line
            debugger; // \t\t {prefix}-disable-line

            //         {prefix}-disable-next-line
            debugger;

            /*      {prefix}-disable-next-line        */
            debugger;

            debugger; /*    {prefix}-disable-line       */

            debugger; //            {prefix}-disable-line no-debugger

            //          {prefix}-disable-next-line no-debugger
            debugger;

            debugger; /*     \t   {prefix}-disable-line no-debugger*/

            /*    \t   {prefix}-disable-next-line no-debugger       */
            debugger;
        "
            ),
            // Handles whitespace character before rule name in comment
            format!(
                "/*{prefix}-disable
no-debugger
*/
            debugger;
        "
            ),
            // Extra commas
            format!(
                "
            debugger // {prefix}-disable-line no-debugger,
            debugger // {prefix}-disable-line ,no-debugger
            debugger // {prefix}-disable-line no-debugger,,
            debugger // {prefix}-disable-line ,,no-debugger,,
            debugger // {prefix}-disable-line ,,no-debugger,,semi,,
            debugger // {prefix}-disable-line ,,no-debugger,,no-debugger,,
            debugger // {prefix}-disable-line ,  , ,,no-debugger, , ,

            // {prefix}-disable-next-line no-debugger,
            debugger
            // {prefix}-disable-next-line ,no-debugger,
            debugger
            // {prefix}-disable-next-line no-debugger,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,semi,,
            debugger
            // {prefix}-disable-next-line ,,no-debugger,,no-debugger,,
            debugger
            // {prefix}-disable-next-line ,  , ,,no-debugger, , ,
        "
            ),
            format!("
                /* {prefix}-disable , ,no-debugger, , */
                debugger;
            "),
            format!("debugger;//{prefix}-disable-line"),
            format!("
            /* {prefix}-disable no-debugger, no-console */
                debugger;
            /* {prefix}-enablefoo, no-debugger, no-console */
                debugger;
            "
            )
        ];

        let fail = vec![
            "debugger".to_string(),
            format!(
                "
            debugger; // {prefix}-disable-line no-alert

            // {prefix}-disable-next-line no-alert
            debugger;

            debugger; /* {prefix}-disable-line no-alert */

            /* {prefix}-disable-next-line no-alert */
            debugger;
        "
            ),
            format!(
                "
            debugger; // {prefix}-disable-line no-alert, quotes, semi

            // {prefix}-disable-next-line no-alert, quotes, semi
            debugger;

            debugger; /* {prefix}-disable-line no-alert, quotes, semi */

            /* {prefix}-disable-next-line no-alert, quotes, semi */
            debugger;

            /* {prefix}-disable-next-line
              no-alert,
              quotes,
              semi
            */
            debugger;
        "
            ),
            format!(
                "
            /* {prefix}-disable-next-line no-debugger --
             * Here's a very long description about why this configuration is necessary
             * along with some additional information
            **/
            debugger;
            debugger;
        "
            ),
            format!(
                "
            // {prefix}-disable-next-line no-debugger
            debugger;
            debugger;
        "
            ),
            // Should not match invalid directives
            // https://github.com/oxc-project/oxc/issues/6041
            format!(
                "// {prefix}-disable-lext-nine no-debugger
                debugger;
                "
            ),
            format!(
                "// {prefix}-disabled no-debugger
                debugger;
                "
            ),
            format!(
                "// {prefix}-disabled
                debugger;
                "
            ),
            format!("debugger; // {prefix}-disable-linefoo no-debugger"),
            format!("debugger; /* {prefix}-disable-linefoo no-debugger */"),
            format!(
                "
            debugger; // {prefix}-disable-lext-nine no-debugger

            // {prefix}-disable-lext-nine no-debugger
            debugger;

            debugger; /* {prefix}-disable-lin no-debugger */

            /* {prefix}-disable-next-lin no-debugger */
            debugger;
        "
            ),
            format!(
                "debugger; // {prefix}-disable-linefoo
            debugger; // {prefix}-disable-linefoo
            debugger; // {prefix}-disable-linefoo no-debugger

            // {prefix}-disable-next-linefoo
            debugger;

            /* {prefix}-disable-next-linefoo */
            debugger;

            debugger; /* {prefix}-disable-linefoo */
            debugger; /* {prefix}-disable-linefoo no-debugger */
        "
            ),
            format!(
                "
            /* {prefix}-disable */
                debugger;
            /* {prefix}-enable */
                debugger;
            "
            ),
            format!(
                "
            /* {prefix}-disable no-debugger, no-console */
                debugger;
            /* {prefix}-enable no-debugger, no-console */
                debugger;
            "
            ),
            // Handles no spaces in comment
            format!(
                "
            /*{prefix}-disable*/
                debugger;
            /*{prefix}-enable*/
                debugger;
            "
            ),
            format!(
                "
            /*{prefix}-disable no-debugger,no-console*/
                debugger;
            /*{prefix}-enable no-debugger,no-console*/
                debugger;
            "
            ),
            format!(
                "debugger; //{prefix}-disable-line no-alert,quotes,semi
            //{prefix}-disable-next-line no-alert,quotes,semi
            debugger;
            debugger; /*{prefix}-disable-line no-alert,quotes,semi */
            /*{prefix}-disable-next-line no-alert,quotes,semi */
            debugger;
            /*{prefix}-disable-next-line
no-alert,
quotes,
semi*/
            debugger;
        "
            ),
            // Handles extra spaces in comment
            format!(
                "
            /*   \t\t {prefix}-disable   \t\t*/
                debugger;
            /*   \t\t {prefix}-enable   \t\t*/
                debugger;
            "
            ),
            format!(
                "
            /*   \t\t {prefix}-disable    \t\t no-debugger,   \t\t no-console   \t\t */
                debugger;
            /*   \t\t {prefix}-enable    \t\t no-debugger,   \t\t no-console   \t\t */
                debugger;
            "
            ),
            format!(
                "debugger; //   \t\t {prefix}-disable-line   \t\t  no-alert,   \t\t quotes,   \t\t semi   \t\t
            //   \t\t {prefix}-disable-next-line   \t\t  no-alert,   \t\t quotes,   \t\t semi
            debugger;
            debugger; /*   \t\t {prefix}-disable-line    \t\t no-alert,   \t\t quotes,   \t\t semi   \t\t  */
            /*   \t\t {prefix}-disable-next-line   \t\t  no-alert,   \t\t quotes,   \t\t semi */
            debugger;
            /*  \t\t {prefix}-disable-next-line
  \t\t no-alert,  \t\t
  \t\t quotes,  \t\t
  \t\t semi  \t\t */
            debugger;
        "
            ),
        ];

        Tester::new(EslintNoDebugger::NAME, EslintNoDebugger::PLUGIN, pass, fail)
            .intentionally_allow_no_fix_tests()
            .test();
    }
}
