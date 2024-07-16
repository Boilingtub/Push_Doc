use std::{fs, path::Path};

pub fn check_if_auth_dir() {
    if !Path::new("auth").exists() {
        println!("The `auth` directory does not exists\nThe directory will now
            be created\n
            Please place your client_secrets file in that directory");
        match fs::create_dir("auth") {
        Ok(v) => v,
        Err(e) => { println!("Error creating `auth` directory\nErr:{}",e);
                    std::process::exit(1);
            }
        };
    };
}

pub fn if_token_exists() -> bool {
    Path::new("auth/token").exists() 
}

pub fn choose_client_secrets() -> String {
    let paths = match fs::read_dir("auth") {
        Ok(v) => v,
        Err(e) =>{ println!("`ERROR`:Could not list files in direcory `auth`\n{{Err={}}}",e); 
                   panic!();
        }
    };

    let mut client_secret_files:Vec<String> = Vec::new();

    for path in paths {
        let p = match path {
            Ok(v) => v,
            Err(e) => { println!("Cannot read file (DISK ERROR) in directory `auth`\nErr:{}",e);
                        panic!();
            },
        };

        if p.path().to_string_lossy().contains("client_secret") {
            client_secret_files.push(p.path().to_string_lossy().to_string());
        }
    }

    if client_secret_files.len() == 0 {
        println!("No client_secrets file's found , please put your 
                  client_secrets file in the `auth` directory");
        return "".to_string();
    } 
    else if client_secret_files.len() == 1 {
        return client_secret_files[0].clone(); 
    }
    else {
        let mut i = 0;
        println!("Multiple client_secret_files detected,
                    type number of file you wish to select");
        for f in client_secret_files.iter() {
            println!("{}.{}",i,f);
            i += 1;
        };
        loop {
            let mut input_line = String::new();
            std::io::stdin().read_line(&mut input_line)
                .expect("Failed to read line");
            let x: usize = match input_line.trim().parse() {
                Ok(v) => v,
                Err(..) => { println!("Please enter a number");
                            continue;
                }
            };

            if x > client_secret_files.len() {
                println!("number `{}` to large",x);
            } else {
                return client_secret_files[x].clone();
            }
        }
    }

}
