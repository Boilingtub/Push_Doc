use std::io::{self, BufReader, ErrorKind, Read,Write,Error};
use std::net::{SocketAddr,ToSocketAddrs,Ipv4Addr};
use std::sync::Arc;

use tokio_rustls::rustls::RootCertStore;

use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};

use tokio::io::{AsyncWriteExt,AsyncReadExt};
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

use rand::Rng;

struct Url {
    //protocol: String,
    domain: String,
    path: String,
}

impl Url {
    fn parse_from_str(url:&str) -> Result<Url, Box<dyn std::error::Error>> {
        let _protocol;
        let domain;
        let path;

        match url.find("://") {
            Some(index1) => {
                _protocol = url[..index1].to_string();
                match url[index1+3..].find("/") {
                    Some(index2) => {
                        domain = url[index1+3..][..index2].to_string();
                        path = url[index1+3..][index2..].to_string();
                    }
                    None => {
                        eprintln!("path not found url needs atleat one \"/\" after domain");
                        return Err(format!("{:?} is invalid url , url need at learst 1 \"/\" after domain",url))?
                    }
                }
            }
            None => {
                eprintln!("// not found in url , invalid formatting");
                return Err(format!("{:?} is invalid url, \"://\" needed in url",url))?;
            }
        };

        return Ok(Url {
            //protocol: protocol,
            domain,
            path
        })

    } 
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
        Some(v) => v,
        None => {
            eprintln!("No port could be found, Defaulting to port: 5444");
            return 5444;
        }
    }
}

pub fn format_as_url(text:String) -> String {
    text.replace(":","%3A").replace("/","%2F")
}

fn ip_address_resolution_error_escape(domain:&str,e:Error) {
    eprintln!("Could not find corrosponding Ip Addres for\n 
                {}:443\nERROR={}\n 
                Please check if domian is correctly supplied\n 
                STOPPING PROGRAM...\n"
                ,domain,e);
}

pub fn send_https(method:&str,raw_url:&str,headers:Vec<(&str,&str)>,body:&str, expect_output:bool) -> String {
    let url = match Url::parse_from_str(raw_url) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse URL , `{}`",e);
            return "".to_string();
        },
    };

    let addr = (url.domain.as_str(),443)
        .to_socket_addrs().unwrap_or_else(|e|{
            ip_address_resolution_error_escape(url.domain.as_str(),e);
            std::process::exit(1);
        })
        .next()
        .ok_or_else(|| Error::from(ErrorKind::NotFound)).unwrap_or_else(|e| {
            ip_address_resolution_error_escape(url.domain.as_str(),e);
            std::process::exit(1);
        });

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Allow using SSLKEYLOGFILE.
    config.key_log = Arc::new(rustls::KeyLogFile::new());


    let server_name = url.domain.clone().try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
    let mut sock = std::net::TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
   
    
    let mut content = method.to_string() +" "+ &url.path+" HTTP/1.1\r\n"+
                      "Host: "+&url.domain+"\r\n"+"Connection: close\r\n";
    for h in headers {
        content = content + h.0 + ": " + h.1 + "\r\n";
    }
    content = content + "\r\n" + body;
    
    //println!("Content = \n{}\n",content);

    tls.write_all(content.as_bytes()).unwrap();
    
    //writeln!(&mut std::io::stderr(),"Current ciphersuite: {:?}",ciphersuite.suite()).unwrap();
    if expect_output == true {
        let mut plaintext = Vec::new();
        match tls.read_to_end(&mut plaintext) {
            Ok(_) => {},
            Err(e) => if e.kind() != std::io::ErrorKind::UnexpectedEof {
                eprintln!("There was and Error reading the stream\n\nError: `{}`\n",e);
            }
        };

        String::from_utf8_lossy(&mut plaintext).to_string()
    } else {
        "Did not expect output".to_string()
    }
}

pub async fn listen_https(addr:SocketAddr, response:&str) -> String{
    const CERT_CONTENT: &str = include_str!("../certs/root.crt");
    const KEY_CONTENT: &str = include_str!("../certs/root.key");
   
    let certs = load_certs_from_str(CERT_CONTENT).expect("Included certificate is not valid, Or could not be found !");
    let key = load_key_from_str(KEY_CONTENT).expect("Included Private_Key is not valid, Or could not be found !");
    //let certs = load_certs("certs/root.crt").unwrap();
    //let key = load_key("certs/root.key").unwrap();

    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err)).unwrap();

    config.key_log = Arc::new(rustls::KeyLogFile::new());
    config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Starting to listen on {}",addr);
    loop {
        let (stream, _peer_addr) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();

        let fut = async move {
            let mut stream = acceptor.accept(stream).await.unwrap();
            //let mut output = sink();
            /*stream.write_all(
                &b"HTTP/1.1 200 ok\r\n\
                Connection: close\r\n\
                Content-length: 20\r\n\
                \r\n\
                Signin Successfull !"[..],
            ).await.unwrap();*/
            stream.write_all(response.as_bytes()).await.unwrap();
                
            let mut buf = Vec::new();   
            let _size = match stream.read_to_end(&mut buf).await{
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to read data from {}\n 
                        FATAL ERROR `{}`",_peer_addr,e);
                    panic!("{}",e);
                }
            };
            
            //println!("size = `{}`\ncontent = `{}`",size,String::from_utf8_lossy(&buf));

            stream.shutdown().await.unwrap();
            //copy(&mut stream, &mut output).await.unwrap();
            //println!("successfully connected to: {}", peer_addr);
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

fn load_certs_from_str(content: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    certs( &mut BufReader::new( content.as_bytes() ) ).collect()
}

fn load_key_from_str(content: &str) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(content.as_bytes()))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        )).unwrap())
}

pub fn generate_random_data_base64url(length:u8) -> String {
    const CHARSET: &[u8;64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef\
                                ghijklmnopqrstuvwxyz0123456789+/";
    let mut rng = rand::thread_rng();
    let rand_bytes: Vec<u8> = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..64);
            CHARSET[idx]
        })
        .collect();
    base64url_encode_no_padding(&rand_bytes)
}

pub fn base64url_encode_no_padding(buffer:&[u8]) -> String {
    for ref mut c in buffer {
        if **c == b'+' {*c = &b'-'};
        if **c == b'/' {*c = &b'_'};
    }
    
    match std::str::from_utf8(buffer) {
        Ok(v) => v,
        Err(e) => panic!("Could not convert bytes to string : `{}`",e),
    }.replace("=","")
}

pub fn strip_string(instr:&str) -> String {
    //instr.replace("\n","")
    instr.to_string()
}
