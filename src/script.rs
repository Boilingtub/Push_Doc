use crate::auth::ClientSecrets;
use crate::docs_update::{DocUpdate,UpdateRequest};
use crate::docs_types::Document;
use crate::googledocs;
use std::fs;

fn read_file_to_string(path:&str) -> String {
    //println!("path:\n{}",path);
    match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could no read ({})\nERR:`{}`",path,e);
            std::process::exit(1);
        }   
    }
    //println!("file content:\n{}",fc);
    
} 

pub struct Script {
    pub jobs:Vec<Job>
}
impl Script {
    pub async fn compile_file(path:&str) -> Script {
        let script_str = read_file_to_string(path);
        Script::compile_from_str(&script_str).await
    }

    pub async fn interpret_file(path:&str) {
        let script_str = read_file_to_string(path);
        Script::interpret_from_str(&script_str).await
    }

    pub fn try_as_file_reference(input:&str) -> String {
        match input.find("$[") {
            Some(a) => match input[a..].find("]$") {
                Some(b) => {
                    //println!("found : {}",&input[a+2..][..b]);
                    read_file_to_string(&input[a+2..][..b-2])
                }
                None => {
                    eprintln!("File path end delimiter `]$` not found\ncheck: {}",&input);
                    input[a+3..].to_owned()
                }
            }
            None => input.to_owned()
        }
    }

   
    pub fn get_job_from_str<'a>(input:&'a str,start_index:&'a mut usize) -> &'a str {
        match input[*start_index..].find("{\n") {
            Some(a) => { 
                match input[*start_index..].find("}\n") {
                    Some(b) => { 
                        let fin_str = &input[*start_index..][a..][..b];
                        *start_index += b+1;
                        fin_str
                    },
                    None => { 
                        eprintln!("Open bracket found on line: {}\n 
                                    but no closing bracket found !",a);
                        ""
                    }
                }
            }
            None => &""    
        }
    }

    pub async fn compile_from_str(input:&str) -> Script {    
        let mut jobs_vec:Vec<Job> = vec![];
        let mut job_index = 0;
        loop {
            let job = Script::get_job_from_str(input,&mut job_index);
            if job.len() > 1 {
                jobs_vec.push(Job::from_str(job).await)
            } else {break;}
        }
        Script { jobs: jobs_vec }
    }

    pub async fn interpret_from_str(input:&str) {
        let mut job_index = 0;
        loop {
            let job = Script::get_job_from_str(input,&mut job_index);
            if job.len() > 1 {
                let job = Job::from_str(job).await;
                println!("UPDATES = \n{}\n",job.updates.to_string());
                googledocs::update_document(&job.id,&job.client_secret,&job.updates.to_string()).await

            } else {break;};
        }
    }


}


pub struct Job {
    pub id:String,
    pub client_secret: ClientSecrets,
    pub updates:DocUpdate,
}
impl Job {
    pub async fn from_str(job_str:&str) -> Job {
        let id = Script::try_as_file_reference(
            Self::get_line_by_key(job_str,"id="));
        let client_secret = match ClientSecrets::new(&Script::try_as_file_reference(
            Self::get_line_by_key(job_str,"client_secrets="))) {
            Ok(v) => v,
            Err(e) => {println!("client_secrets, is not valid JSON !\n 
                                Check if you missed the $[***]$ or if the
                                json is maybe incorrectly formatted !\nERR`{}`",e);
                        std::process::exit(1);
            }
        };

        let original_document = googledocs::get_document_by_id(&id,&client_secret).await;

        let updates = Self::parse_updates(job_str,&original_document);
        //println!("Returning JOB");
        Job {id,client_secret,updates}
    }

    fn get_line_by_key<'a>(job_str:&'a str, key_str:&'a str) -> &'a str {
        match job_str.find(key_str) {
            Some(a) => match job_str[a+key_str.len()..].find("\n") {
                Some(b) => {
                    &job_str[a+key_str.len()..][..b]
                },
                None => {
                    println!("There Must be a new line after {}*** , on line {}",key_str,a);
                    ""
                }
            },
            None =>{
                println!("Job must have a {}*** line !",key_str);
                ""
            }
        }
    }

    fn parse_updates(job_str:&str, original_document:&Document) -> DocUpdate {
        let mut update_vec:Vec<UpdateRequest> = Vec::new();
        //Add future task_types here
        update_vec.append(&mut Self::get_all_delete_content_range(job_str,original_document.last_index));
        update_vec.append(&mut Self::get_all_insert_text(job_str,original_document.last_index));
        update_vec.append(&mut Self::get_all_replace_all_text(job_str));
        DocUpdate::new(update_vec)
    }

    fn get_all_delete_content_range(job_str:&str,doc_last_index:u64) -> Vec<UpdateRequest> {
        let mut deleterange_vec:Vec<UpdateRequest> =  Vec::new();
        const MATCH_STR:&str = "deleteContentRange=";
        let match_str_len = MATCH_STR.len();
        for i in job_str.match_indices(MATCH_STR) {
            let content = match job_str[i.0+match_str_len..].find("\n") {
                Some(v) => &job_str[i.0+MATCH_STR.len()..][..v],
                None => {
                    println!("There `Must be a new line after {}*** ,"
                        ,&job_str[i.0..(i.0+MATCH_STR.len())]);
                    ""                    
                }
            };
            deleterange_vec.push(new_delete_content_range(content,doc_last_index));
        }
        deleterange_vec 
    }

    fn get_all_insert_text(job_str:&str,doc_last_index:u64) -> Vec<UpdateRequest> {
        let mut insert_vec:Vec<UpdateRequest> =  Vec::new();
        const MATCH_STR:&str = "insertText=";

        let mut index_vec:Vec<usize> = Vec::new() ;
        for i in job_str.match_indices(MATCH_STR) {
            index_vec.push(i.0)
        };

        for i in index_vec.iter().rev() {
            let content = match job_str[*i..].find("\n") {
                Some(v) => &job_str[i+MATCH_STR.len()..][..v],
                None => {
                    println!("There Must be a new line after {}*** ,"
                        ,&job_str[*i..(i+MATCH_STR.len())]);
                    ""                    
                }
            };

            insert_vec.push(new_insert_text(content,doc_last_index));
        }

        insert_vec
    }

    fn get_all_replace_all_text(job_str:&str) -> Vec<UpdateRequest> {
        let mut replaceall_vec:Vec<UpdateRequest> =  Vec::new();
        const MATCH_STR:&str = "replaceAllText=";
        let match_str_len = MATCH_STR.len();
        for i in job_str.match_indices(MATCH_STR) {
            let content = match job_str[i.0+match_str_len..].find("\n") {
                Some(v) => &job_str[i.0+MATCH_STR.len()..][..v],
                None => {
                    println!("There `Must be a new line after {}*** ,"
                        ,&job_str[i.0..(i.0+MATCH_STR.len())]);
                    ""                    
                }
            };
            replaceall_vec.push(new_replace_all_text(content));
        }
        replaceall_vec 
    }
}



pub fn two_simple_parameter(content:&str) -> (&str,&str) {
    let (parm1,parm2) = match content.find("(") {
        Some(a) => match content[a..].find(",") {
            Some(b) => match content[a..][b..].find(")") {
                Some(c) => (&content[a+1..][..b-1] , &content[a+1..][b..][..c-1]),
                None => {
                    eprintln!("\nneed closing \")\" to end \nin line : {}",content);
                    std::process::exit(1);
                },
            },
            None => {
                eprintln!("\nneed  \",\" to seperate values\n in line: {}",content);
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("\nneed opening \"(\", to start\nin line: {}",content);
            std::process::exit(1);
        }
    };
    (parm1,parm2) 
}

pub fn try_index_parameter_as_u64(index:&str,doc_last_index:u64) -> u64 {
    if index == "END" {
        doc_last_index-1
    }
    else if index == "START" {
        1
    }
    else {
    match index.parse::<u64>() {
            Ok(v) => if v > doc_last_index {doc_last_index}
                     else if v < 1 {1}
                     else {v},
            Err(e) => {
                eprintln!("Cannot parse index in \n{} \n{}",index,e);
                std::process::exit(1);
            }
        }
    }
}

pub fn new_delete_content_range(content:&str,doc_last_index:u64) -> UpdateRequest {
    let (start,end) = two_simple_parameter(content);
    let start_index = try_index_parameter_as_u64(start,doc_last_index);
    let end_index = try_index_parameter_as_u64(end,doc_last_index);
    UpdateRequest::new_delete_content_range(start_index,end_index,"")
}

pub fn new_insert_text(content:&str,doc_last_index:u64) -> UpdateRequest {
    let (to_insert,index) = two_simple_parameter(content);
    let index_as_num = try_index_parameter_as_u64(index,doc_last_index);
    let fin_insert_text = Script::try_as_file_reference(to_insert);
    UpdateRequest::new_insert_text_request(&fin_insert_text,index_as_num,"")
}

pub fn new_replace_all_text(content:&str) -> UpdateRequest {
    let (to_replace,replacer) = two_simple_parameter(content); 
    let fin_replace_text = Script::try_as_file_reference(to_replace);
    UpdateRequest::new_replace_all_text_request(&fin_replace_text,replacer,true)
}
