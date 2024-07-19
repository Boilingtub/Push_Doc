pub struct DocUpdate { //replaceText
    requests: Vec<UpdateRequest>,
    write_control: WriteControl,
}
impl DocUpdate {
    pub fn new(requests: Vec<UpdateRequest>, revision_id:&str) 
               -> DocUpdate {
        DocUpdate {
            requests,
            write_control: WriteControl::new(revision_id),
        }
    }

    pub fn add_request(&mut self,req:UpdateRequest) {
        self.requests.push(req);
    }

    pub fn to_string(&self) -> String {
        "{\n".to_owned() +
            "  \"requests\": [\n" +
                &self.requests_to_string() +
            "  ],\n" +
            "  \"writeControl\": {\n" + 
                "    \"requiredRevisionId\": " + &self.write_control.required_revision_id + ",\n" +
                "    \"targetRevisionId\": " + &self.write_control.target_revision_id + "\n" +
            "  }\n" +
        "}"
    }

    fn requests_to_string(&self) -> String {
        let mut finstr = String::new();
        for r in self.requests.iter() {
            let s = match r {
                UpdateRequest::ReplaceAllText(v) =>{
                    "    \"replaceAllText\": {\n".to_owned() +
                        "      \"replaceText\": \"" + &v.replace_text + "\",\n" +
                        "      \"containsText\": {\n" +
                            "        \"text\": \"" + &v.contains_text.text + "\",\n" +
                            "        \"matchCase\": " + &v.contains_text.match_case.to_string() + "\n" +
                        "      }\n" +
                    "    },\n" 
                },
                UpdateRequest::InsertText(v) => {
                    "    \"insertText\": {\n".to_owned() +
                        "      \"text\": \"" + &v.text + "\",\n" +
                        "      \"location\": {\n" +
                            "        \"segmentId\": \"" + &v.location.segment_id + "\",\n" +
                            "        \"index\": " + &v.location.index.to_string() + "\n" +
                        "      },\n" +
                        "      \"endOfSegmentLocation\": {\n" +
                            "        \"segmentId\": \"" + &v.end_of_segment_location.segment_id + "\"\n" +
                        "      }\n" +
                    "    },\n"
                },
            };
            finstr += &s
        }
        finstr[..finstr.len()-2].to_string() + "\n"
    }
}


pub enum UpdateRequest {
    ReplaceAllText(ReplaceAllText),
    InsertText(InsertText)
}
impl UpdateRequest {
    pub fn new_replace_all_text_request(to_replace:&str, replace_text:&str,
                                     case_sensitive:bool) -> UpdateRequest {
        UpdateRequest::ReplaceAllText( 
            ReplaceAllText {
                replace_text: replace_text.to_string(),
                contains_text: ContainsText {
                    text: to_replace.to_string(),
                    match_case: case_sensitive,
                }
            }
        )
    }
    
    pub fn new_insert_text_request(text:&str, location:u32, 
                               segment_id_start:&str, segment_id_end:&str)
                               -> UpdateRequest {
        UpdateRequest::InsertText(
            InsertText {
                text: text.to_string(),
                location: Location {
                    segment_id: segment_id_start.to_string(),
                    index: location,
                },
                end_of_segment_location: EndOfSegmentLocation {
                    segment_id: segment_id_end.to_string()
                }
            }
        )
    }
}

//-----
pub struct ReplaceAllText {
    replace_text: String,
    contains_text: ContainsText,
}
pub struct ContainsText {
    text: String,
    match_case: bool,
}
//-----

//-----
pub struct InsertText {
    text: String,
    location: Location,
    end_of_segment_location: EndOfSegmentLocation,
}
pub struct Location {
    segment_id: String,
    index: u32,
}
pub struct EndOfSegmentLocation {
    segment_id: String
}
//-----

//-----
pub struct WriteControl {
    required_revision_id: String,
    target_revision_id: String,
}
impl WriteControl {
    pub fn new(revision_id:&str) -> WriteControl {
        WriteControl {
            required_revision_id: revision_id.to_string(),
            target_revision_id: revision_id.to_string()
        }
    }
}
//-----
