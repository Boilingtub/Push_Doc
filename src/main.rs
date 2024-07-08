//use jsonic;
use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use openssl::ssl::{SslConnector, SslMethod};

struct Url {
    protocol: String,
    domain: String,
    path: String,
}

impl Url {
    fn parse_from_str(url:&str) -> Result<Url, Box<dyn std::error::Error>> {
        let protocol;
        let domain;
        let path;

        match url.find("://") {
            Some(index1) => {
                protocol = url[..index1].to_string();
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
            protocol: protocol,
            domain: domain,
            path: path
        })

    } 
}

enum HttpMethod {
    GET,
    POST,
}

struct HttpRequest<'a> {
    method: HttpMethod,
    path: String,
    http_version: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: String, 
    
}

impl <'a>HttpRequest<'_> {
    fn new(method: HttpMethod, path: &'a str, http_version: &'a str ,headers: Vec<(&'a str,&'a str)>, body:&'a str ) ->  HttpRequest<'a> {
        HttpRequest {
            method: method,
            path: path.to_string(),
            http_version: http_version,
            headers: headers,
            body: body.to_string(),
        }   
    }
    fn to_string(&mut self) -> String {
        let method:String = match &self.method {
            HttpMethod::GET  => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
        };

        let headers:String = Self::headers_as_string(self);
        
        method + " " + &self.path + " " + "HTTP/"+ self.http_version + "\r\n" + &headers + "\r\n" + &self.body
    }

    fn headers_as_string(&self) -> String {
        let mut headers_string: String = "".to_string();
        for h in &self.headers {
            headers_string.push_str(&h.0);
            headers_string.push_str(": ");
            headers_string.push_str(&h.1);
            headers_string.push_str("\r\n");
        };
        headers_string
    }
}

fn main() -> std::io::Result<()> {
    println!("Starting Push_Doc");
    //let select_url = "https://docs.googleapis.com/v1/documents/1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo";
    //let select_url = "https://httpbin.org/get";
    let select_url = "https://docs.googleapis.com/v1/documents/";

    let url = match Url::parse_from_str(select_url) {
        Ok(v) => v,
        Err(e) => {
            println!("{}",e);
            return Ok(());
        },
    };

    let host: SocketAddr = match gethostbyname(&url.domain) {
        Ok(v) => v,
        Err(v) => {
            println!("{}",v);
            return Ok(());
        },
    };

    println!("Domain: {} , ip: {:?}",&url.domain, &host.ip());

    let addr = SocketAddr::new(host.ip(), 80);
    
    let mut stream =  match TcpStream::connect(addr) {
        Ok(v) => {
            println!("connected to server !\n");
            v
        },
        Err(v) => {
            println!("Could not connect to server: {:?}\\n",v);
            return Ok(());
        }
    };

    
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let mut stream = connector.connect(&url.domain, stream).unwrap();
    

    let body = "{}";
    let body_len = body.len().to_string();
    let headers : Vec<(&str,&str)> = vec![
        ("host", "docs.googleapis.com"),
        ("body" , "Empty"),
        ("Content-Length", &body_len),
    ];

    let mut request = HttpRequest::new(HttpMethod::POST, &url.path, "1.0" , headers, body);
    
    println!("Sending Request: \n\n{}",request.to_string());

    stream.write_all(request.to_string().as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut res = vec![];

    stream.read_to_end(&mut res).unwrap();
    println!("{}",String::from_utf8_lossy(&res));
    
    Ok(())
} 


fn gethostbyname(hostname: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    if let Ok(mut sock_addrs) = (hostname, 0).to_socket_addrs() {
        if let Some(sock_addr) = sock_addrs.next() {
            return Ok(sock_addr);
        }
    }
    Err(format!("no ip address for {:?}",hostname))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_test() {
        let result = Url::parse_from_str("https://docs.googleapis.com/documents/v1").unwrap();
        assert_eq!(result.protocol, "https");
        assert_eq!(result.domain  , "docs.googleapis.com");
        assert_eq!(result.path, "/documents/v1");
    }

    #[test]
    fn http_request_create_test() {
        let headers : Vec<(&str,&str)> = vec![
            ("Content-Length","1916"),
            ("Date", "2024 07 07"),
            ("Content-Type", "application/json"),
        ];
        let mut request = HttpRequest::new(HttpMethod::GET, "v1/documents/", "1.1" , headers, "{\nBodyText\n}");
        assert_eq!(request.to_string(),"GET v1/documents/ HTTP/1.0\r\nContent-Length: 1916\r\nDate: 2024 07 07\r\nContent-Type: application/json\r\n\r\n{\nBodyText\n}");
    }
}


