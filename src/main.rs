
pub mod networking;
use networking::ClientSecrets;
use std::{fs,env};

#[tokio::main]
async fn main() {
    let client_secrets = fs::read_to_string("auth/client_secret_1014602435348-jm9bsfir9g662f3qahjlj8il1an3g3gd.apps.googleusercontent.com.json")
        .expect("Could not find client secrets file");
    let client_secrets = match ClientSecrets::new(&client_secrets) {
        Ok(v) => v,
        Err(e) => panic!("{}",e),
    };

        println!("starting google authentication...");    
        networking::do_oauth_async(client_secrets).await;
        println!("\n\ngoogle authentication End ...");

    /*let args: Vec<String> = env::args().collect(); 
    if args.len() == 0 {
        println!("starting google authentication...");    
        networking::do_oauth_async(client_secrets).await;
        println!("\n\ngoogle authentication End ...");
    } else if args.len() == 1 {
        if args[1] == "code" {
            networking::do_oauth_async(client_secrets).await;
        }
        else if args[1] == "token" {
            networking::exchange_code_for_tokens_async()
        }
    }*/


    /*let document_url = "https://docs.googleapis.com/v1/documents/1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo/";
    
    let api_key = fs::read_to_string("auth/api_key")
        .expect("could not find API_KEY file");
    */


    //println!("google authenticatio successfull!");
    

    //let user_consent_code = google::get_user_consent(&client_secrets);
    //let user_consent_code = fs::read_to_string("auth/code").expect("Cannot find auth/code file");
    //let use_access_token = google::get_autorization_code(&client_secrets, &user_consent_code);
    /*
    
    let client_autorization_string = google::get_autorization_code(&client_secrets,&api_key);
    let client_autorization = match jsonic::parse(&client_autorization_string) {
        Ok(v) => v,
        Err(e) => {
            println!("Error parsing client_autorization_string ,check if file valid\n{}",e);
            return ;
        },
    };
    
    let headers:Vec<(header::HeaderName,&str)> = vec![];

    let body = "{}";

    if let Err(e) = send_overhttps("GET",document_url,headers,body.to_string()) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
*/
    
}

