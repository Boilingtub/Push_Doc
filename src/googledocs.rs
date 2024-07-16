use crate::networking;
use crate::auth;

const DOCS_URL:&str = "https://docs.googleapis.com/v1/documents/";

pub struct Document<'a> {
    title:&'a str,
}

pub async fn get_document_by_id(doc_id:&str) -> String { 
    println!("get_document_by_id({})",&doc_id);
    let url = DOCS_URL.to_owned() + doc_id;
    let acces_token = auth::get_access_token(false).await;

    let headers:Vec<(&str,&str)> = vec![
        ("Authorization",&acces_token),
    ];
    let response = networking::send_https("GET",&url,headers,"",None).await;
    
    println!("resonse:\n{{\n{}\n}}\n",response);
    let data = auth::parse_auth_data_from_response(&response);
    if data.contains("\"code\": 401") {
        auth::renew_access_token().await;
        let fut = Box::pin(async move {        
            get_document_by_id(doc_id).await
        });
        fut.await

    } else {
        response
    }
}

pub async fn create_document() {
    
}

pub async fn update_document(doc_id:&str,) {

}
