pub mod networking;
pub mod googledocs;
pub mod validate;
pub mod auth;
pub mod script;

#[tokio::main]
async fn main() {
    let test_doc_id = "1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo";
    googledocs::get_document_by_id(test_doc_id).await;
}


