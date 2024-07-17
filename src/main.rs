pub mod networking;
pub mod googledocs;
pub mod validate;
pub mod auth;
pub mod script;

#[tokio::main]
async fn main() {
    let test_doc_id = "1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo";
    let doc = googledocs::get_document_by_id(test_doc_id).await;
    //let doc_json_body = doc.json["body"].as_str().unwrap();
    //println!("\n\ndoc_json_body = \n#BODY_START\n{}\n#BODY_END\n",doc_json_body);
    println!("doc_title = {}",doc.title);
}


