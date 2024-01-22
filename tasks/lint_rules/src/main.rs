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

    let markdown = generate_list::run(plugin_name).unwrap();

    if !is_update {
        println!("{markdown}");
        return;
    }

    let url = update_comment::run(plugin_name, &markdown).unwrap();
    println!("✨ Done! Status for {plugin_name} is updated!");
    println!("See {url}");
}
