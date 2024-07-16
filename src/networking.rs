use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::net::{SocketAddr,ToSocketAddrs,Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tokio::io::{copy, sink, AsyncWriteExt,AsyncReadExt};
use tokio::net::{TcpListener,TcpStream};
use tokio_rustls::{rustls, TlsAcceptor,TlsConnector};

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
                        println!("path not found url needs atleat one \"/\" after domain");
                        return Err(format!("{:?} is invalid url , url need at learst 1 \"/\" after domain",url))?
                    }
                }
            }
            None => {
                println!("// not found in url , invalid formatting");
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
        Some(v) => Some(v).unwrap(),
        None => {
            eprintln!("No port could be found");
            return 5444;
        }
    }
}

pub fn format_as_url(text:String) -> String {
    text.replace(":","%3A").replace("/","%2F")
}

pub async fn send_https(method:&str, select_url:&str, headers:Vec<(&str,&str)>, body:&str, cafile:Option<PathBuf>) -> String {
    let url = match Url::parse_from_str(select_url) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to parse URL , `{}`",e);
            return "".to_string();
        },
    };
    //default Port for http = 80 ; default port for https = 443
    let addr = (url.domain.as_str(),443)
        .to_socket_addrs().unwrap()
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound)).unwrap();


    let mut content = method.to_string() +" "+ &url.path+" HTTP/1.1\r\n"+"Host: "+&url.domain+"\r\n";
    for h in headers {
        content = content + h.0 + ": " + h.1 + "\r\n";
    }
    content = content + "\r\n" + body;

    let mut root_cert_store = rustls::RootCertStore::empty();
    if let Some(cafile) = cafile {
        let mut pem = BufReader::new(File::open(cafile).unwrap());
        for cert in rustls_pemfile::certs(&mut pem) {
            root_cert_store.add(cert.unwrap()).unwrap();
        }
    } else {
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let stream = TcpStream::connect(&addr).await.unwrap();
    
    println!("Attempting connection to domain: {}  on address: {}",url.domain, addr);
    println!("Sending content: {{\n{}\n}}\n\n",content);

    let domain = rustls::pki_types::ServerName::try_from(url.domain)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid DNS-name")).unwrap()
        .to_owned();

    let mut stream = match connector.connect(domain, stream).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to connect to {}, because `{}`",addr, e);
            return "".to_string();
        }
    };
  
    stream.write_all(content.as_bytes()).await.unwrap(); 
    let mut buffer = [0;8192];
    let byte_count = stream.read(&mut buffer).await.unwrap();
    //println!("Stream {:?}",String::from_utf8_lossy(&buffer[0..byte_count]));
    
    String::from_utf8_lossy(&buffer[0..byte_count]).to_string()

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
    println!("Starting to listen on {}",addr);
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
            let _size = stream.read_to_end(&mut buf).await.unwrap();
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
