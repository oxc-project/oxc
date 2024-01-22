mod generate_list;
mod update_comment;

const HELP: &str = r"
Usage:
  $ cargo run <plugin_name> [--update]

Arguments:
  plugin_name: Name of the target plugin

Options:
  --update: Update GitHub issue comment
  -h, --help: Show this message
";

fn main() {
    // Consider using CLI parser library?
    let args = std::env::args();
    let (mut positionals, mut options) = (vec![], vec![]);
    for arg in args.skip(1) {
        if arg.starts_with('-') {
            options.push(arg);
        } else {
            positionals.push(arg);
        }
    }

    let is_update = options.contains(&"--update".to_string());
    if options.contains(&"--help".to_string()) || options.contains(&"-h".to_string()) {
        println!("{HELP}");
        return;
    }

    let plugin_name =
        positionals.first().unwrap_or_else(|| panic!("🆖 Target plugin_name is required!\n{HELP}"));

    // Task start!

    let markdown = generate_list::run(plugin_name).unwrap();

    // Print markdown and exit, you can paste it to update comment manually
    if !is_update {
        println!("{markdown}");
        return;
    }

    // This is automatically set by GitHub Apps for CI, your local env should set this manually
    // https://docs.github.com/en/actions/security-guides/automatic-token-authentication
    let token =
        std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| panic!("🆖 env.GITHUB_TOKEN is not set!"));

    let issue_url = update_comment::run(plugin_name, &token, &markdown).unwrap();
    println!("✨ Done! Status for {plugin_name} is updated!");
    println!("See {issue_url}");
}
