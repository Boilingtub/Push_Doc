pub mod networking;
pub mod docs_types;
pub mod googledocs;
pub mod validate;
pub mod auth;
pub mod script;

fn write_to_file(file_path:&str, data:&str) {
    let mut out_file = match std::fs::File::create(file_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to create {} file\n{}",file_path,e);
            return ;
        }
    };
    use std::io::Write;
    write!(out_file, "{}", data).unwrap();

}

#[tokio::main]
async fn main() {
    let test_doc_id = "1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo";
    let doc = googledocs::get_document_by_id(test_doc_id).await;

    //write_to_file("examples/json", doc.json.as_str().unwrap());
    write_to_file("examples/doc", &doc.to_string());
    
}


