use crate::networking;
use crate::auth;
use jsonic::json_item::JsonItem;
const DOCS_URL:&str = "https://docs.googleapis.com/v1/documents/";

pub struct Document<'a> {
    pub id: &'a str,
    pub title: String,
    pub json: JsonItem,
    //pub body_lines: Vec<String>,
}

impl <'a>Document<'_> {
    pub fn new(raw_json:String,id:&'a str) -> Document<'a> {
        let json = match jsonic::parse(raw_json.as_str()) {
            Ok(v) => v,
            Err(e) => {println!("error parsing document `{}`",e);
                      panic!("could not parse document json in Document::new(raw_json:&str)");
            },
        };
        let title = Document::get_title_from_raw_json(&raw_json);
        Document {
            id,
            title,
            json,
        }
    }

    fn get_title_from_raw_json(raw_json:&str) -> String {
        let key_title = "\"title\": ";
        match raw_json.find(key_title) {
            Some(v) => {
                let last_index = match raw_json[v..].find("\",\n") {
                    Some(v) => v+5,
                    None => {
                        0
                    },
                };
                if last_index > 0 {
                    &raw_json[v+key_title.len()..last_index]
                }
                else {"Unspecified Title"}
            },
            None => {
                "Unspecified Title"
            },
        }.to_string()
    }


    pub fn json_as_str(&self) -> &str {
        match self.json.as_str() {
            Some(v) => v,
            None => {println!("Document Could not be parsed as a string , object corrupted !");
                       panic!("object corrupted !");
            },
        }
    }

}

pub async fn get_document_by_id(doc_id:&str) -> Document { 
    //println!("get_document_by_id({})",&doc_id);
    let url = DOCS_URL.to_owned() + doc_id;
    let acces_token = auth::get_access_token(false).await;

    let headers:Vec<(&str,&str)> = vec![
        ("Authorization",&acces_token),
    ];
    let response = networking::send_https("GET",&url,headers,"",None).await;
    
    //println!("resonse:\n\\n\n{}\n\\n\n",response);
    let data = auth::parse_auth_data_from_response(&response);
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
