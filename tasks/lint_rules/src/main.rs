mod generate_list;
mod update_comment;

fn main() {
    let mut args = std::env::args();
    args.next();

    let plugin_name = args.next().expect("Plugin name");
    let is_update = args.any(|arg| arg == "--update");

    let list = generate_list::run(&plugin_name).unwrap();

    if !is_update {
        println!("{list}");
        return;
    }

    let url = update_comment::run(&plugin_name, &list).unwrap();
    println!("✨ Done! Status for {plugin_name} is updated!");
    println!("See {url}");
}
