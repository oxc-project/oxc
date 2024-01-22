mod generate_list;
mod update_comment;

const HELP: &str = r"
Usage:
  $ cargo run <plugin_name> [--update]

Arguments:
  - plugin_name: Name of the target plugin

Options:
  - --update: Update GitHub issue comment
  - -h, --help: Show this message
";

fn main() {
    let mut args = std::env::args();
    args.next();

    if args.any(|arg| arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return;
    }

    let plugin_name =
        args.next().unwrap_or_else(|| panic!("🆖 Target plugin_name is required!\n{HELP}"));
    let is_update = args.any(|arg| arg == "--update");

    let markdown = generate_list::run(&plugin_name).unwrap();

    if !is_update {
        println!("{markdown}");
        return;
    }

    let url = update_comment::run(&plugin_name, &markdown).unwrap();
    println!("✨ Done! Status for {plugin_name} is updated!");
    println!("See {url}");
}
