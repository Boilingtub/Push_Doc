use crate::networking;
use crate::auth;
use crate::docs_types::Document;
const DOCS_URL:&str = "https://docs.googleapis.com/v1/documents/";
use spinners::Spinner;

pub async fn get_document_by_id(doc_id:&str) -> Document { 
    //println!("get_document_by_id({})",&doc_id);
    let mut sp = Spinner::new(spinners::Spinners::Dots, "Getting Document from google cloud...".into());
    
    let url = DOCS_URL.to_owned() + doc_id;
    let acces_token = auth::get_access_token(false).await;

    let headers:Vec<(&str,&str)> = vec![
        ("Authorization",&acces_token),
    ];
    let response = networking::send_https("GET",&url,headers,"",None).await;
    
    //println!("resonse:\n\\n\n{}\n\\n\n",response);
    let data = auth::parse_auth_data_from_response(&response);
    
    sp.stop(); println!();

    if data.contains("\"code\": 401") {
        auth::renew_access_token().await;
        let fut = Box::pin(async move {        
            get_document_by_id(doc_id).await
        });
        fut.await
    } else {
        let parsed_response = auth::parse_auth_data_from_response(&response);
        //println!("parsed_response:\n\\n\n{}\n\\n\n",parsed_response);
        Document::new(parsed_response,doc_id)
    }
}

pub async fn create_document() {
    
}

pub async fn update_document(doc_id:&str,) {
    println!("UPDATE {}",doc_id);
}
