use std::net::{SocketAddr,Ipv4Addr};
use std::fs;
use crate::networking::{get_random_unused_port, generate_random_data_base64url, base64url_encode_no_padding, send_https, listen_https};
use crate::validate;

pub struct ClientSecrets {
    pub token_uri:String,
    pub client_id:String,
    pub client_secret:String,
    pub redirect_uris:String
}
impl ClientSecrets {
   pub fn new(client_secrets:&str) -> Result<ClientSecrets,String> {
        let client_secrets_json = match jsonic::parse(client_secrets) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error parsing client_secrets ,check if file valid\n{}",e);
                return Err(format!("Error parsing client secrets\n{}",client_secrets));
            },
        };
        let token_uri = client_secrets_json["installed"]["token_uri"].as_str()
                .expect("Could not find \"token_uri\" in client_secrets_json");
        let client_id = client_secrets_json["installed"]["client_id"].as_str()
                .expect("Could not find \"client_id\" in client_secrets_json");
        let client_secret = client_secrets_json["installed"]["client_secret"].as_str()
                .expect("Could not find \"client_secret\" in client_secrets_json");
        let redirect_uris = client_secrets_json["installed"]["redirect_uris"].as_str()
                .expect("Could not find \"redirect_uri\" in client_secrets_json");
        Ok(ClientSecrets {
            token_uri: token_uri.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uris: redirect_uris.to_string(),
        })
    }


    pub fn to_string(&self) -> String {
       format!("ClientSecrets {{ token_uri: {} client_id: {} client_secret: {} redirect_uris: {} }}"
            ,self.token_uri, self.client_id, self.client_secret, self.redirect_uris) 
    }
}

pub async fn create_access_token(client_secrets:&ClientSecrets) {
    do_oauth_async(client_secrets).await;
}

pub async fn recurse_async_get_access_token(renew:bool,client_secrets:&ClientSecrets) -> String {
    Box::pin(async move {        
        get_access_token(renew,client_secrets).await
    }).await
}

pub async fn get_access_token(renew: bool , client_secrets:&ClientSecrets) -> String {
    if validate::if_token_exists() == true{
        let token_raw_json = match fs::read_to_string("auth/token") {
            Ok(v) => v,
            Err(e) => { eprintln!("Could not read token file, Err:{}",e);
                        panic!();
            },
        };
        let token_json = match jsonic::parse(&token_raw_json) {
            Ok(v) => v,
            Err(..) => { eprintln!("Token file not valid json, recreating...");
                        create_access_token(client_secrets).await;
                        return recurse_async_get_access_token(renew,client_secrets).await;
            },
        };
        
        let token_type = match token_json["token_type"].as_str() {
            Some(v) => v,
            None => { eprintln!("Could not find `token_type`, recreating token...");
                        create_access_token(client_secrets).await;
                        return recurse_async_get_access_token(renew,client_secrets).await;
            }
        };

        let to_get = if renew == false { "access_token"} else {"refresh_token"};
        let token_value = match token_json[to_get].as_str() {
            Some(v) => v,
            None => { eprintln!("Could not find `access_token`, recreating token...");
                      create_access_token(client_secrets).await;
                      return recurse_async_get_access_token(renew,client_secrets).await;
            },
        };

        if renew == false {
            return token_type.to_owned() + " " + token_value
        } else {
            return token_value.to_string()
        }
    }
    else {
        create_access_token(client_secrets).await;
        recurse_async_get_access_token(renew,client_secrets).await
    }
}


pub fn get_client_secrets() -> ClientSecrets {
    validate::check_if_auth_dir();
    let path = validate::choose_client_secrets();
    let client_secrets_raw_json = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) => { eprintln!("Could not read client_secrets file\n{}
                    \n make sure the client_secrets file in in the `auth`
                    directory",e);
                    std::process::exit(1);
        },
    };
    match ClientSecrets::new(&client_secrets_raw_json) {
        Ok(v) => v,
        Err(e) => { eprintln!("Please reinstall your client_secrets file\n 
                            {}\n",e);
                    std::process::exit(1);
        }
    }
}

#[allow(unused)]
pub async fn do_oauth_async(client_secrets:&ClientSecrets) {
    let listen_port = get_random_unused_port();
    let redirect_uri = format!("redirect_uri=https://localhost:{}/",listen_port);
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), listen_port);
    //Generate state and PKCE values
    let state = generate_random_data_base64url(64);
    let code_verifier = generate_random_data_base64url(64);
    let code_challenge = base64url_encode_no_padding(sha256::digest(&code_verifier).as_bytes());
    const CODE_CHALLENGE_METHOD:&str = "S256";
  
    let url_base = "https://accounts.google.com/o/oauth2/v2/auth?";
    let autorization_request =
    /*format!("{}response_type=code&scope=https://www.googleapis.com/auth/documents&{}&client_id={}&state={}&code_challenge={}&code_challenge_method={}"
        ,url_base,redirect_uri,client_secrets.client_id,state,code_challenge
        ,CODE_CHALLENGE_METHOD);*/

   format!("{}response_type=code&scope=https://www.googleapis.com/auth/documents&{}&client_id={}"
        ,url_base,redirect_uri,client_secrets.client_id);


    //println!("autorization_request = {}",autorization_request);

    
    tokio::spawn(async move {    
        let _ = std::process::Command::new("xdg-open")
                        .arg(autorization_request)
                        .output()
                        .expect("Failed to execute xdg-open");
    });
    
    let response = "HTTP/1.1 200 ok\r\nConnection: close\r\nContent-length: 20\r\n\r\nSignin Successfull !";
    let code = parse_auth_code_from_response( &listen_https(addr,response).await );  
   
    /*println!("code = {}",code);
    let mut out_file = match std::fs::File::create("auth/code") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to create auth code file\n{}",e); //DEBUGGING BLOCK
            return ;
        }
    };
    use std::io::Write;
    write!(out_file, "{}", code).unwrap();
    let code = std::fs::read_to_string("auth/code").unwrap();*/

    exchange_code_for_tokens_async(&client_secrets,&code,&code_verifier).await;
}


#[allow(unused)]
pub async fn exchange_code_for_tokens_async(client_secrets:&ClientSecrets, code:&str, code_verifier: &str) {
    let listen_port = get_random_unused_port();
    let redirect_uri =  format!("https://localhost:{}/",listen_port) ;

    //println!("\n!NOTE : {} : NOTE!\n",redirect_uri);
    
    let token_request_body =
        format!("code={}&redirect_uri={}&client_id={}&client_secret={}&scope=&grant_type=authorization_code"
                ,code,redirect_uri,client_secrets.client_id ,client_secrets.client_secret);
   
    let body = token_request_body;
    let body_len = body.len().to_string();
    let headers = vec![
        ("Content-Type","application/x-www-form-urlencoded"),
        ("Content-Length", &body_len),
    ];

    let httpresponse = send_https("POST",&client_secrets.token_uri,headers,&body,true);
    //println!("Response From {} :{{\n{}\n}}",client_secrets.token_uri,httpresponse);
    let auth_request = parse_auth_data_from_response(&httpresponse);
    validate::check_if_auth_dir();
    save_auth_token_local(&auth_request);

}

pub async fn renew_access_token(client_secrets:&ClientSecrets) {
    validate::check_if_auth_dir();
    //let client_secrets = get_client_secrets();
    let refresh_token = &get_access_token(true,client_secrets).await;

    let token_request_body =
        format!("client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
                client_secrets.client_id,client_secrets.client_secret,refresh_token);
    let body = token_request_body;
    let body_len = body.len().to_string();
    let headers = vec![
        ("Content-Type","application/x-www-form-urlencoded"),
        ("Content-Length", &body_len),
    ];
    let httpresponse = send_https("POST",&client_secrets.token_uri,headers,&body,true);
    let renew_request = parse_auth_data_from_response(&httpresponse);
    update_auth_token_local(&renew_request,client_secrets).await;
}

fn parse_auth_code_from_response(httpresponse: &str) -> String {
    //println!("httpresponse to parse : \n{}",httpresponse);
    let index1 = match httpresponse.find("code=") {
        Some(v) => v,
        None => { eprintln!("there is no code in googleauth URI");
            return "Invalid google response".to_string();
        },
    };
    let index2 = match httpresponse.find("&scope") {
        Some(v) => v,
        None => { eprintln!("there is no code in googleauth Scope");
            return "Invalid google response".to_string();
        },
    };
    httpresponse[index1+5..index2].to_string() 
}

pub fn parse_auth_data_from_response(httpresponse: &str) -> String {
    let index1 = match httpresponse.find('{') {
        Some(v) => v,
        None => panic!("Could not find `{{` in auth_str , auth_str not valid json"),
    };
    let index2 = match httpresponse.rfind('}') {
        Some(v) => v,
        None => panic!("Could not find `}}` in auth_str , auth_str not valid json"),
    }; 
    httpresponse[index1..index2+1].to_string()
}

fn save_auth_token_local(auth_str: &str) {
    let mut out_file = match std::fs::File::create("auth/token") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to create auth/token file\n{}",e);
            return ;
        }
    };
    use std::io::Write;
    write!(out_file, "{}", auth_str).unwrap();
}

async fn update_auth_token_local(renew_token: &str, client_secrets:&ClientSecrets) {
    let renew_json = match jsonic::parse(renew_token) {
        Ok(v) => v,
        Err(e) => { println!("failed to parse, renew Token, not valid json\n
                    Try restarting the application\n{}",e);
                    std::process::exit(1);
        },
    };
   
    if !validate::if_token_exists() {
        create_access_token(client_secrets).await;
        return;
    } 

    let token_raw_json = match std::fs::read_to_string("auth/token") {
        Ok(v) => v,
        Err(e) => { println!("Could not read token file, Err:{}",e);
                    panic!();
        },
    };


    let token_json = match jsonic::parse(&token_raw_json) {
        Ok(v) => v,
        Err(e) => { println!("failed to parse,  Token file, not valid json\n
                    Try restarting the application\n{}",e);
                    std::process::exit(1);
        },
    };
    
    let org_access_token = token_json["access_token"].as_str()
        .expect("invalid JSON , could not find `access_token field` in org_access_token");
    let new_access_token = renew_json["access_token"].as_str()
        .expect("invalid JSON, could not find `access_token_field in new_access_toeken`");    
    let org_token_str = match token_json.as_str() {
        Some(v) => v,
        None => { println!("FATAL ERROR : token_json invalid json ,cannot be converted to
                        string");
                  std::process::exit(1);
        }
    };
    let new_token_str = org_token_str.replace(org_access_token,new_access_token);
    save_auth_token_local(&new_token_str);
}
