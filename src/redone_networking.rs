use std::io::{Error,ErrorKind,BufReader,Read,Write};
use std::net::{TcpStream,TcpListener,ToSocketAddrs,SocketAddr};
use std::sync::Arc;

use rustls::RootCertStore;

use std::env;
use std::error::Error as StdError;
use std::fs::File;

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

pub fn test_client(method:&str,raw_url:&str,headers:Vec<(&str,&str)>,body:&str) -> String {
    let url = match Url::parse_from_str(raw_url) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse URL , `{}`",e);
            return "".to_string();
        },
};

    let addr = (url.domain.as_str(),443)
        .to_socket_addrs().unwrap()
        .next()
        .ok_or_else(|| Error::from(ErrorKind::NotFound)).unwrap();


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
    let mut sock = TcpStream::connect(addr).unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
   
    
    let mut content = method.to_string() +" "+ &url.path+" HTTP/1.1\r\n"+
                      "Host: "+&url.domain+"\r\n"+"Connection: close\r\n";
    for h in headers {
        content = content + h.0 + ": " + h.1 + "\r\n";
    }
    content = content + "\r\n" + body;
    
    println!("Content = \n{}\n",content);

    tls.write_all(content.as_bytes()).unwrap();
    
    let ciphersuite = tls
        .conn
        .negotiated_cipher_suite()
        .unwrap();
    writeln!(
        &mut std::io::stderr(),
        "Current ciphersuite: {:?}",
        ciphersuite.suite()
    )
    .unwrap();
    let mut plaintext = Vec::new();
    match tls.read_to_end(&mut plaintext) {
        Ok(_) => {},
        Err(e) => eprintln!("There was and Error reading the stream\n\nError: `{}`\n",e)
    };

    String::from_utf8_lossy(&mut plaintext).to_string()
}
/*
pub async fn send_https(method:&str, select_url:&str, headers:Vec<(&str,&str)>, body:&str, cafile:Option<PathBuf>) -> String {
    let url = match Url::parse_from_str(select_url) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse URL , `{}`",e);
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
    

    let stream = match TcpStream::connect(&addr).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to connect to {}\n 
                check your internet connection",url.domain);
            panic!("{}",e);
        }
    };
    

    
    //println!("Attempting connection to domain: {}  on address: {}",url.domain, addr);
    //println!("Sending content: {{\n{}\n}}\n\n",content);

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

    let mut fin_str = String::new();
    loop {
        let mut buffer = [0;8192];

        let byte_count = match stream.read(&mut buffer).await {
            Ok(v) => v,
            Err(e) => { eprintln!("\nConnection Interuppted while reading ! `ERR:\n{}\n`",e);
                0
            }
        };

        println!("SUCCESSFULLY READ STREAM !");
        //println!("\n\nStream {:?}\n\n",String::from_utf8_lossy(&buffer[0..byte_count]));
        if byte_count == 0 {break;};
       
        let stream_str = String::from_utf8_lossy(&buffer[0..byte_count]);

        fin_str += &stream_str;
        if stream_str.contains("0\r\n\r\n") {break;}; 
    }
    //println!("fin_str = \n{}",fin_str);
    fin_str
}
*/

pub fn test_server(addr:SocketAddr,response:&str) -> String {
    let cert_file = "certs/root.crt";
    let private_key_file = "certs/root.key";

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file).unwrap()))
        .collect::<Result<Vec<_>, _>>().unwrap();
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(private_key_file).unwrap())).unwrap()
            .unwrap();
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key).unwrap();
    config.key_log = Arc::new(rustls::KeyLogFile::new());
    config.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

   
    let listener = TcpListener::bind(addr).unwrap();
    let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
    println!("Starting to listen on {}...",addr);
    
    let mut tls = rustls::Stream::new(&mut conn, &listener)
    /*loop {
        let (mut stream, peer_addr) = listener.accept().unwrap(); 
        println!("Accepted Connection from {}",peer_addr);

        let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();

        conn.complete_io(&mut stream).unwrap();

        conn.writer().write_all(response.as_bytes()).unwrap();
        loop {

            let mut buf:Vec<u8> = Vec::new();
            match conn.reader().read_to_end(&mut buf) {
                Ok(_) => {},
                Err(e) => 
                 if e.kind() == std::io::ErrorKind::WouldBlock {
                    //eprintln!("connection Would Block !");
                    continue;
                } else {
                    eprintln!("reading from connection Error [`{}`]",e);
                }
            }

            let s = String::from_utf8_lossy(&buf).to_string();
            println!("Received message from client: {}",s);
            break;
        }*/
        
        /*let mut conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();

        conn.complete_io(&mut stream).unwrap();

        conn.process_new_packets();
        let mut buf:Vec<u8> = Vec::new();
        println!("attempting read");
        match conn.reader().read_to_end(&mut buf) {
            Ok(_) => {},
            Err(e) => 
            if e.kind() == std::io::ErrorKind::WouldBlock {
                eprintln!("connection Would Block !");
            } else {
                eprintln!("reading from connection Error [`{}`]",e);
            }
        };

        conn.writer().write_all(response.as_bytes()).unwrap();
        conn.complete_io(&mut stream).unwrap();
        conn.complete_io(&mut stream).unwrap();
        

        let s = String::from_utf8_lossy(&buf).to_string();
        println!("Received message from client: {}",s);
        //return fut.await*/
    
    
}
