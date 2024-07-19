pub mod networking;
pub mod docs_types;
pub mod docs_update;
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
    const DOCS_URL:&str = "https://docs.googleapis.com/v1/documents/";
    let test_doc_id = "1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo";
    //let doc = googledocs::get_document_by_id(test_doc_id).await;
    /*let addr = std::net::SocketAddr::new(std::net::Ipv4Addr::LOCALHOST.into(), 8000);
    let response = "HTTP/1.1 200 ok\r\nConnection: close\r\nContent-length: 20\r\n\r\nSignin Successfull !";
    redone_networking::example_server(addr,response);*/

  
    let doc = googledocs::get_document_by_id(test_doc_id).await;
    println!("{}",doc.to_string());
    let update_requests = vec![
        docs_update::UpdateRequest::new_replace_all_text_request("hello","idkhow",true),
        docs_update::UpdateRequest::new_insert_text_request("inserted here !",1,"",""),
    ];
    let mut update = docs_update::DocUpdate::new(update_requests,"");
    update.add_request(docs_update::UpdateRequest::new_replace_all_text_request("bye","wohkdi",false)); 
    println!("{}",update.to_string());

    googledocs::update_document(doc.id, &update.to_string()).await;
    //println!("{}",update.to_string())*/

    
    //write_to_file("examples/json", doc.json.as_str().unwrap());
    //write_to_file("examples/doc", &doc.to_string());*/
    
}


