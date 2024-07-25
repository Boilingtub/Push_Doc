pub mod networking;
pub mod docs_types;
pub mod docs_update;
pub mod googledocs;
pub mod validate;
pub mod auth;
pub mod script;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        script::Script::interpret_file(&args[1]).await
    } else {
        println!("No Script file specified");
    }
}
