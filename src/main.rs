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
        let jobs = script::Script::parse_file(&args[1]).await.jobs;
        for job in jobs {
            googledocs::update_document(&job.id, &job.client_secret , &job.updates.to_string()).await; 
        }  
    } else {
        println!("No Script file specified");
    }
}
