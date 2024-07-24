use crate::networking;
use crate::auth;
use crate::docs_types::Document;
use spinners::Spinner;

const DOCS_URL:&str = "https://docs.googleapis.com/v1/documents/";

pub async fn get_document_by_id<'a>(doc_id:&'a str, client_secrets:&'a auth::ClientSecrets) -> Document<'a> { 
    let mut sp = Spinner::new(spinners::Spinners::Dots, "Getting Document from google cloud...".into());
    
    let url = DOCS_URL.to_owned() + doc_id;
    let acces_token = auth::get_access_token(false,client_secrets).await;

    let headers:Vec<(&str,&str)> = vec![
        ("Authorization",&acces_token),
    ];
    let response = networking::send_https("GET",&url,headers,"",true);
    
    //println!("resonse:\n\\n\n{}\n\\n\n",response);
    let data = auth::parse_auth_data_from_response(&response);
    
    sp.stop(); println!();

    if data.contains("\"code\": 401") {
        auth::renew_access_token(client_secrets).await;
        let fut = Box::pin(async move {        
            get_document_by_id(doc_id,client_secrets).await
        });
        fut.await
    } else {
        let parsed_response = auth::parse_auth_data_from_response(&response);
        //println!("parsed_response:\n\\n\n{}\n\\n\n",parsed_response);
        Document::new(parsed_response,doc_id)
    }
}

pub async fn update_document(doc_id:&str, client_secrets:&auth::ClientSecrets, update_body:&str) -> String {
    let mut sp = Spinner::new(spinners::Spinners::Dots, "Uploading Document to google cloud...".into());
    let url = DOCS_URL.to_owned() + doc_id + ":batchUpdate"; 
    let access_token = auth::get_access_token(false,client_secrets).await;
    let content_lengthnum = update_body.len();
    let content_length = content_lengthnum.to_string();

    let headers:Vec<(&str,&str)> = vec![
        ("Authorization",&access_token),
        ("Content-length",&content_length),
        ("Content-type","application/json")
    ];
    let response = networking::send_https("POST",&url,headers,
                        &networking::strip_string(&update_body),true);
     
    let data = auth::parse_auth_data_from_response(&response);

    sp.stop(); println!();

    if data.contains("\"code\": 401") {
        auth::renew_access_token(client_secrets).await;
        let fut = Box::pin(async move {        
            update_document(doc_id,client_secrets,update_body).await
        });
        fut.await
    } else {
        auth::parse_auth_data_from_response(&response)
    }


}
