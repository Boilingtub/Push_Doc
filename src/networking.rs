/*
use bytes::Bytes;
use hyper::header;
use std::net::Ipv4Addr;
use std::{fs, io};

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor; 
*/
/*
    use std::io;
    use std::io::{Write, Read, BufReader, BufRead};
    use std::sync::Arc;

    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::server::Acceptor;
    use rustls::ServerConfig; 

    use std::net::{Ipv4Addr,SocketAddr};
*/
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::net::{SocketAddr,ToSocketAddrs,Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tokio::io::{copy, sink, split, AsyncWriteExt,AsyncReadExt};
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

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
                println!("Error parsing client_secrets ,check if file valid\n{}",e);
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



pub struct Httpresponse {
    status: String,
    headers: Vec<(String,String)>,
    body: String,
}

pub enum ServiceType {
    Echo,
    GoogleAuth,
}


fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}


pub fn get_random_unused_port() -> u16 {
     fn port_is_available(port:u16) -> bool {
        match std::net::TcpListener::bind( SocketAddr::new(Ipv4Addr::LOCALHOST.into(),port) ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    let avail_port = (8000..9000).find(|port| port_is_available(*port));
    match avail_port {
        Some(v) => Some(v).unwrap(),
        None => {
            eprintln!("No port could be found");
            return 5444;
        }
    }
}

pub async fn do_oauth_async(client_secrets:ClientSecrets) {
    let listen_port = get_random_unused_port();
    let redirect_uri = format!("redirect_uri=https%3A//localhost%3A{}&",listen_port);
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), listen_port);

    

    let url_base = "https://accounts.google.com/o/oauth2/v2/auth?";
    let autorization_request = format!("{}response_type=code&scope=https://www.googleapis.com/auth/documents&{}&client_id={}&",url_base,redirect_uri,client_secrets.client_id);

    println!("autorization_request = {}",autorization_request);

   
    let _ = std::process::Command::new("xdg-open")
                        .arg(autorization_request)
                        .output()
                        .expect("Failed to execute xdg-open");
   
    let response = "HTTP/1.1 200 ok\r\nConnection: close\r\nContent-length: 20\r\n\r\nSignin Successfull !";
    let code = parse_auth_code_from_response( &listen_https(addr,response).await );  
    println!("code = {}",code);
    
    exchange_code_for_tokens_async(&client_secrets,&code);
}

pub fn exchange_code_for_tokens_async(client_secrets:&ClientSecrets, code:&str) {
    let listen_port = get_random_unused_port();
    let redirect_uri = format!("redirect_uri=https%3A//localhost%3A{}&",listen_port);

    let token_request_body = format!("code={}&redirect_uri={}&client_id={}&client_secrets={}&scope=&grant_type=authorization_code"
                                ,code,redirect_uri,client_secrets.client_id,client_secrets.client_secret);

    send_https();
}

pub async fn send_https(method:&str, url:&str, headers:<>) {
    
}

pub async fn listen_https(addr:SocketAddr, response:&str) -> String{
    let certs = load_certs(Path::new("certs/root.crt")).unwrap();
    let key = load_key(Path::new("certs/root.key")).unwrap();

    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err)).unwrap();

    config.key_log = Arc::new(rustls::KeyLogFile::new());
    config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(&addr).await.unwrap();

    loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();

        let fut = async move {
            let mut stream = acceptor.accept(stream).await.unwrap();
            let mut output = sink();
            /*stream.write_all(
                &b"HTTP/1.1 200 ok\r\n\
                Connection: close\r\n\
                Content-length: 20\r\n\
                \r\n\
                Signin Successfull !"[..],
            ).await.unwrap();*/
            stream.write_all(response.as_bytes()).await.unwrap();
                
            let mut buf = Vec::new();   
            let size = stream.read_to_end(&mut buf).await.unwrap();
            //println!("size = `{}`\ncontent = `{}`",size,String::from_utf8_lossy(&buf));

            stream.shutdown().await.unwrap();
            copy(&mut stream, &mut output).await.unwrap();
            println!("successfully connected to: {}", peer_addr);
            String::from_utf8_lossy(&buf).to_string()
            //Ok(()) as io::Result<()>
        };

        return fut.await
        /*tokio::spawn(async move {
            if let Err(err) = fut.await {
                eprintln!("{:?}", err);
            }
        });*/
    }
}

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
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
/*
pub fn listen_https(addr:SocketAddr) {
    //env_logger::init();
    let msg = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Connection: Closed\r\n",
        "Content-Type: text/html\r\n",
        "\r\n",
        "<h1>Success</h1>\r\n"
    )
    .as_bytes();

    //let pki = Pki::new();
    let pki = Pki::from_file("certs/root.crt" , "certs/root.key");
    let server_config = pki.server_config();

    let listener = std::net::TcpListener::bind(addr).unwrap();
        for incoming_stream in listener.incoming() {
            let mut stream = match incoming_stream {
                Ok(incoming_stream) => incoming_stream,
                Err(e) => { 
                    eprintln!("error with incoming stream : `{}`",e);
                    return;
                }
            };

            stream.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap(); 
            let mut acceptor = Acceptor::default();

            let accepted =  {
                acceptor.read_tls(&mut stream).unwrap();
                let accept = match acceptor.accept() {
                    Ok(v) => v,
                    Err(e) => { eprintln!("acceptor got error `{:?}`",e);
                        eprintln!("cannot read form socket");
                        None
                    }
                };

                let accepted = match accept {
                    Some(a) => a,
                    None => {
                        println!("Could not accept socket !, no accepted socket");
                        continue;
                    }
                };   
                 accepted
            };
      
            let mut conn = match accepted.into_connection(server_config.clone()) {
                Ok(c) => c,
                Err(e) => { 
                    eprintln!("{e:?}");
                    return;
                }
            };
       


            conn.writer().write_all(msg).unwrap();
            conn.write_tls(&mut stream).unwrap();
            conn.complete_io(&mut stream).unwrap();

            read_from_conn(&mut conn,&mut stream); 
            
            conn.send_close_notify();
            conn.write_tls(&mut stream).unwrap();
            conn.complete_io(&mut stream).unwrap();

            println!("Connection Completed !");
    }
}

fn read_from_conn(conn : &mut rustls::ServerConnection,stream: &mut std::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut loop_ctrl = 0;
    while loop_ctrl < 5{
        println!("[{}] attempting to read stream...",loop_ctrl);
        let size = match conn.reader().read_to_end(&mut buf) {
            Ok(v) => {
                loop_ctrl = 10;
                v
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                eprintln!("ERR( `std::io::ErrorKind::WouldBlock` ) , while reading stream , {}",e);
                std::thread::sleep(std::time::Duration::from_millis(1000));
                loop_ctrl += 1;
                0
            }
            Err(e) => {
                eprintln!("ERROR encountered while reading stream `{}`",e);
                return Default::default();
            }
        };
        //conn.read_tls(stream).unwrap();
        //conn.complete_io(stream).unwrap();
        println!("size={}:content={}",size,String::from_utf8_lossy(&buf));
    }

    String::from_utf8_lossy(&buf).to_string()

}

struct Pki {
    server_cert_der: Vec<CertificateDer<'static>>,
    server_key_der: PrivateKeyDer<'static>,
}

impl Pki {
    fn new() -> Self {
        let alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        /*ca_params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "janhendrik");*/
        ca_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "localhost");
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];
        let ca_key = rcgen::KeyPair::generate_for(alg).unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        // Create a server end entity cert issued by the CA.
        let mut server_ee_params =
            rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
        server_ee_params.is_ca = rcgen::IsCa::NoCa;
        server_ee_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        let server_key = rcgen::KeyPair::generate_for(alg).unwrap();
        let server_cert = server_ee_params
            .signed_by(&server_key, &ca_cert, &ca_key)
            .unwrap();
        Self {
            server_cert_der: vec![server_cert.into()],
            // TODO(XXX): update below once https://github.com/rustls/rcgen/issues/260 is resolved.
            server_key_der: PrivatePkcs8KeyDer::from(server_key.serialize_der()).into(),
        }
    }

    fn from_file(crt_path:&str , key_path:&str) -> Self {
        use std::{fs,io};
        let certfile = fs::File::open(crt_path).expect(format!("Could not open {}",crt_path).as_str());
        let mut cert_reader = io::BufReader::new(certfile);
        let cert: io::Result<Vec<CertificateDer<'static>>> = rustls_pemfile::certs(&mut cert_reader).collect();

        let keyfile = fs::File::open(key_path).expect(format!("Could not open {}",key_path).as_str());
        let mut key_reader = io::BufReader::new(keyfile);
        let key = rustls_pemfile::private_key(&mut key_reader).map(|key| key.unwrap()).unwrap();

        Self {
            server_cert_der: cert.unwrap(),
            server_key_der: key,
        }

    }

    fn server_config(self) -> Arc<ServerConfig> {
        let mut server_config =
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(self.server_cert_der, self.server_key_der)
                .unwrap();

        server_config.key_log = Arc::new(rustls::KeyLogFile::new());
        server_config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

        Arc::new(server_config)
    }
}


#[allow(unreachable_code)]
pub async fn tokio_listen_https(addr:SocketAddr) -> Result<(),Box<dyn std::error::Error + Send + Sync>>  {
    #[cfg(feature = "ring")]
    let _ = rustls::crypto::ring::default_provider().install_default();

    let certs = load_certs("certs/root.crt")?;
    let key = load_private_key("certs/root.key")?;

    println!("Starting to serve on https://{}",addr);

    let listner = TcpListener::bind(&addr).await?;

    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| error(e.to_string()))?;

    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let mutex1 = Arc::new(tokio::sync::Mutex::new(0));

    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));
    loop
    {
        let (tcp_stream, remote_addr) = listner.accept().await?;
        let tls_acceptor = tls_acceptor.clone(); 
        let mutex2 = Arc::clone(&mutex1);
        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
            Ok(tls_stream) => tls_stream,
            Err(err) => {
                eprintln!("failed to perform tls handshake: {err:#}");
                return Ok(());
            }
        };

        let io = TokioIo::new(tls_stream);

        tokio::spawn(async move {
            let mut lock = mutex2.lock().await;
            *lock +=1;

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(io, service_fn(googleauth))
                .await
            {
                eprintln!("failed to serve connection: {err:#}");
            }

        });
        
        let lock = mutex1.lock().await;

        println!("Served lock={lock} {remote_addr} ");
        if *lock >= 1 {break}
    }
    Ok(())
}

#[tokio::main]
pub async fn send_overhttps(method:&str, url:&str, headers:Vec<(header::HeaderName,&str)>, body:String) 
-> Result<Httpresponse, Box<dyn std::error::Error + Send + Sync> > {
    use http_body_util::BodyExt;
    use hyper::Request;
    use hyper_util::{client::legacy::Client, rt::TokioExecutor};

    println!("Attempting Connection to : {}",url);
    let url = url.parse::<hyper::Uri>().unwrap();
    let authority = url.authority().unwrap().clone();

    let mut req = Request::builder()
        .method(method)
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(body)
        .unwrap();

    for h in headers {
        req.headers_mut().append(h.0,header::HeaderValue::from_str(h.1).unwrap());
    }

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("no native root CA certificates found")
        .https_only()
        .enable_http1()
        .build();



    let client = Client::builder(TokioExecutor::new()).build(https);
    println!("Sending Request: {:?}",req);

    let fut = async move {
        let res = client.request(req).await.map_err(|e| error(format!("Could not get: {:?}",e))).unwrap();
        let status = res.status().clone();
        let headers = res.headers().clone();
        println!("Status:\n{}",res.status());
        println!("Headers:\n{:#?}",res.headers());

        let body = res
            .into_body()
            .collect()
            .await
            .map_err(|e| error(format!("Could not get body: {:?}", e))).expect("Could not get body")
            .to_bytes();
        println!("Body:\n{}", String::from_utf8_lossy(&body));

        Ok( Httpresponse {
            status: status.to_string(),
            headers: headers,
            body: String::from_utf8_lossy(&body).to_string(),
        } )
    };

    fut.await
}

async fn googleauth(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> { 
    println!("Got request! , handeling with googleauth()");
    println!("{:?}",req);
    println!("{:?}",req.body()); 
    let response_bad = Response::new(Full::from("Signin Failed"));
    let response_good = Response::new(Full::from("<html><head><meta http-equiv='refresh' content='10;url=https://google.com'></head><body>Please return to the app.</body></html>"));
    
    if req.method() == Method::GET {

        match req.uri().to_string().find("favicon.ico") {
            Some(_) => {
                println!("Google end message found `favico.ico`");
            },
            None => println!("Looking for code"),
        }

        let uri = req.uri().to_string();
        let index1 = match uri.find("code") {
            Some(v) => v,
            None => { eprintln!("there is no code in googleauth URI");
                return Ok(response_bad);
            },
        };
        let index2 = match uri.find("&scope") {
            Some(v) => v,
            None => { eprintln!("there is no code in googleauth Scope");
                return Ok(response_bad);
            },
        };

        let code = &uri[index1+6..index2];
        println!("code = {}",code);
        let mut out_file = match std::fs::File::create("auth/code") {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to create auth code file\n{}",e);
                return Ok(response_bad);
            }
        };
        use std::io::Write;
        write!(out_file, "{}", code).unwrap();
    }

    
    Ok(response_good)
}


// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    rustls_pemfile::certs(&mut reader).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
*/
