use jsonic::json_item::JsonItem;

pub struct GetJson {}
impl GetJson {
    pub fn i128(json:&JsonItem,field:&str) -> i128 {
        if json[field].exists() {
            match json[field].as_i128() {
                Some(v) => v,
                None => {
                    eprintln!("Jsonitem [\"{}\"] is not a integer",field);
                    panic!("not a number !!!");
                },
            }
        }
        else {
            eprintln!("JsonItem [\"{}\"] does not exist",field);
            panic!("Could not find field");
        }
    }

    pub fn f64(json:&JsonItem,field:&str) -> f64 {
        if json[field].exists() {
            match json[field].as_f64() {
                Some(v) => v,
                None => {
                    eprintln!("Jsonitem [\"{}\"] is not a floating point number",field);
                    panic!("not a number !!!");
                },
            }
        }
        else {
            eprintln!("JsonItem [\"{}\"] does not exist",field);
            panic!("Could not find field");
        }
    }

    pub fn string(json:&JsonItem,field:&str) -> String {
        if json[field].exists() {
            match json[field].as_str() {
                Some(v) => v.to_string(),
                None => {
                    eprintln!("Jsonitem [\"{}\"] is not a string",field);
                    panic!("not a number !!!");
                },
            }
        }
        else {
            eprintln!("JsonItem [\"{}\"] does not exist",field);
            panic!("Could not find field");
        }
    }
    pub fn bool(json:&JsonItem,field:&str) -> bool {
        if json[field].exists() {
            match json[field].as_bool() {
                Some(v) => v,
                None => {
                    eprintln!("Jsonitem [\"{}\"] is not a boolean",field);
                    panic!("not a number !!!");
                },
            }
        }
        else {
            eprintln!("JsonItem [\"{}\"] does not exist",field);
            panic!("Could not find field");
        }
    }
}


pub struct Document<'a> { 
    pub title: String,
    pub body: Vec<BodyPart>,
    pub document_style:String,
    pub named_styles:String, 
    pub revision_id:String,
    pub suggestions_view_mode: String,
    pub id: &'a str,
    pub last_index:u64,
    pub json: JsonItem,
}

impl <'a>Document<'_> {
    pub fn new(raw_json:String,id:&'a str) -> Document<'a> {
        let json = match jsonic::parse(raw_json.as_str()) {
            Ok(v) => v,
            Err(e) => {println!("error parsing document `{}`",e);
                      panic!("could not parse document json in Document::new(raw_json:&str)");
            },
        };
        let body = Document::get_body_from_json(&json);
        let last_index = Document::get_body_last_index(&body,body.len()); 
    
        Document {
            title: Document::get_title_from_raw_json(&raw_json),
            body,
            document_style: GetJson::string(&json,"documentStyle"),
            named_styles: GetJson::string(&json,"namedStyles"),
            revision_id: GetJson::string(&json,"revisionId"),
            suggestions_view_mode: GetJson::string(&json,"suggestionsViewMode"),
            id,
            last_index,
            json,
        }
    }

    fn get_title_from_raw_json(raw_json:&str) -> String {
        let key_title = "\"title\": ";
        match raw_json.find(key_title) {
            Some(v) => {
                let last_index = match raw_json[v..].find("\",\n") {
                    Some(v) => v+5,
                    None => {
                        0
                    },
                };
                if last_index > 0 {
                    &raw_json[v+key_title.len()..last_index]
                }
                else {"Unspecified Title"}
            },
            None => {
                "Unspecified Title"
            },
        }.to_string()
    }

    fn get_body_last_index(body:&Vec<BodyPart>, length:usize) -> u64{
        match body.get(length) {
            Some(v) => match v {
                BodyPart::Paragraph(p) => p.end_index,
                BodyPart::SectionBreak(s) => s.end_index,
                BodyPart::Unknown(u) => {
                    eprintln!("last BodyPart is of Type BodyPart::Unknown \n{}\n",u);

                    Document::get_body_last_index(&body, length -1 )
                }
            }
            None => {
                Document::get_body_last_index(&body, length -1)
            }        
        }
    }

    fn get_body_from_json(json:&JsonItem) -> Vec<BodyPart> {
        let parts_iter = match json["body"]["content"].elements() {
            Some(v) => v,
            None => {eprintln!("Document retrived does not contain a 
                               \"body\": {{ \"content\": \"\"}} key 
                                make sure document is correct");
                    return Vec::new();
            },
        };
        let mut parts_vec:Vec<BodyPart> = Vec::new();
        for part in parts_iter {
            parts_vec.push(BodyPart::from_json(part));
        };
        parts_vec
    }

    pub fn to_string(&self) -> String {
        "{".to_string() + "\n" + 
        "\"title\": " + &self.title + "\n" +
        "\"body\": {\n" +
        "\"content\": [\n" +
            &Document::body_as_str(self) +   
        "]\n" +
        "},\n" +
        "\"documentStyle\": " + &self.document_style + ",\n" +
        "\"namedStyles\": " + &self.named_styles + ",\n" +

        "\"revision_id\": \"" + &self.revision_id + "\",\n" +
        "\"suggestionsViewMode\": \"" + &self.suggestions_view_mode + "\",\n" +
        "\"documentId\": \"" + &self.id + "\"" +
        "\n}"
    }

    fn body_as_str(&self) -> String {
        let mut finstr = String::new();
        for b in self.body.iter().take(self.body.len()-1) {
            let s ="{\n".to_string() +
                    &b.to_string() + 
                    "\n},\n";
            finstr += &s
        }

        let last_s = match &self.body.last() {
            Some(b) => &b.to_string(),
            None => {
                eprintln!("Last index of BodyParts Vector is Empty !
                            Returning empty String as body part !");
                ""
            }
        };
        finstr += &("{\n".to_string() + last_s + "\n}\n");
        finstr
    }
}

pub enum BodyPart {
    SectionBreak(SectionBreak),
    Paragraph(Paragraph),
    Unknown(String),
}

impl BodyPart {
    pub fn from_json(json:&JsonItem) -> BodyPart {
        if json["sectionBreak"].exists() {
            BodyPart::SectionBreak(SectionBreak::from_json(json))
        }
        else if json["paragraph"].exists() {
            BodyPart::Paragraph(Paragraph::from_json(json))
        }
        else {
            BodyPart::Unknown(match json.as_str() {
                Some(v) => v.to_string(),
                None => panic!("`ERROR`: Evaluating unknown BodyPart,
                                that is EMPTY !\n Document is not a 
                                valid !, check for possible internet connection
                                problems or packet loss !\n"),
            })
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            BodyPart::SectionBreak(v) => v.to_string(),
            BodyPart::Paragraph(v) => v.to_string(),
            BodyPart::Unknown(v) => v.to_string(),
        }
    } 
}

pub struct SectionBreak {
    start_index:u64,
    end_index:u64,
    //section_break in a "sectionBreak": {...}
    section_break:String,
}
impl SectionBreak {
    pub fn from_json(json:&JsonItem) -> SectionBreak {
        let end_index =  i128_to_u64(GetJson::i128(json,"endIndex"));
        
        let start_index = if !json["startIndex"].exists() {
            end_index-1
        } else {
            i128_to_u64(GetJson::i128(json,"startIndex"))
        };

        SectionBreak {
            start_index,
            end_index,
            section_break: GetJson::string(json,"sectionBreak"),
        }
    }

    fn to_string(&self) -> String {
        let start_index_str = if self.start_index > 0 {
            "\"startIndex\": ".to_owned()+&self.start_index.to_string()+",\n"
        } else {"".to_string()};
        start_index_str + 
        "\"endIndex\": " + &self.end_index.to_string() + ",\n" +
        "\"sectionBreak\": " + &self.section_break
    }
}

pub struct Paragraph {
    start_index:u64,
    end_index:u64,
    //elements and paragraph_style is in a "paragraph": {...}
    elements:Vec<ParagraphElement>,
    paragraph_style:ParagraphStyle,
}
impl Paragraph {
    pub fn from_json(json:&JsonItem) -> Paragraph {
        
        let elements_iter = match json["paragraph"]["elements"].elements() {
            Some(v) => v,
            None => {eprintln!("Document retrived does not contain a 
                               \"paragraph\":{{\"elements\":[]}} key 
                                make sure document is correct [reinstall recommended]");
                    std::process::exit(1);
            },
        };

        let mut elements_vec:Vec<ParagraphElement> = Vec::new();
        for element in elements_iter {
            elements_vec.push(ParagraphElement::from_json(element));
        };

        Paragraph {
            start_index: i128_to_u64(GetJson::i128(json,"startIndex")),
            end_index:  i128_to_u64(GetJson::i128(json,"endIndex")),
            elements: elements_vec,
            paragraph_style: ParagraphStyle::from_json(&json["paragraph"]["paragraphStyle"]),
        }
    }

    fn to_string(&self) -> String {
        "\"startIndex\": ".to_owned() + &self.start_index.to_string() + ",\n" +
        "\"endIndex\": " + &self.end_index.to_string() + ",\n" +
        "\"paragraph\": {\n" + 
            "\"elements\": [\n" +
                &self.paragraph_elements_as_str() +
            "],\n" +
            "\"paragraphStyle\": {\n" + 
                &self.paragraph_style.to_string() +
            "}\n" +
        "}"
    }

    fn paragraph_elements_as_str(&self) -> String {
        let mut finstr = String::new();
        for e in self.elements.iter().take(self.elements.len()-1) {
            let s ="{\n".to_string() +
                    &e.to_string() + 
                    "\n},\n";
            finstr += &s
        }

        let last_s = match &self.elements.last() {
            Some(e) => &e.to_string(),
            None => {
                eprintln!("Last index of BodyParts Vector is Empty !
                            Returning empty String as body part !");
                ""
            }
        };
        finstr += &("{\n".to_string() + last_s + "\n}\n");
        finstr

    }
}

pub struct ParagraphElement {
    start_index:u64,
    end_index:u64,
    text_run:TextRun,
}
impl ParagraphElement {
     pub fn from_json(json:&JsonItem) -> ParagraphElement {
        ParagraphElement {
            start_index: i128_to_u64(GetJson::i128(json,"startIndex")),
            end_index:  i128_to_u64(GetJson::i128(json,"endIndex")),
            text_run: TextRun::from_json(&json["textRun"]), 
        }
    }

    fn to_string(&self) -> String {
        //"{\n".to_owned() +
            "\"startIndex\": ".to_owned() + &self.start_index.to_string() + ",\n" +
            "\"endIndex\": " + &self.end_index.to_string() + ",\n" +
            "\"textRun\": {\n" +
                &self.text_run.to_string() +
            "}"
       // "}\n"
    }
}

pub struct TextRun {
    content:String,
    text_style:String,
}
impl TextRun {
    pub fn from_json(json:&JsonItem) -> TextRun {
        TextRun {
            content: GetJson::string(&json,"content"),
            text_style: GetJson::string(&json,"textStyle"),
        }
    }

    fn to_string(&self) -> String {
        "\"content\": \"".to_owned() + &self.content + "\",\n" +
        "\"textStyle\": " + &self.text_style + "\n"
    }
}

pub struct ParagraphStyle {
    named_style_type:String,
    direction:String,
}
impl ParagraphStyle {
    pub fn from_json(json:&JsonItem) -> ParagraphStyle {
        ParagraphStyle {
            named_style_type: GetJson::string(json,"namedStyleType"),
            direction: GetJson::string(json,"direction"),
        } 
    }

    fn to_string(&self) -> String {
        "\"namedStyleType\": \"".to_owned() + &self.named_style_type + "\",\n" +
        "\"direction\": \"" + &self.direction + "\",\n"
    }
}

fn i128_to_u64(v:i128) -> u64 {
    if v > std::u32::MAX as i128 {
        eprintln!("`ERROR !`number bigger that u32 , cannot fit !!!!
                    truncating to u32::MAX !!!!!");
        u64::MAX
    } else {
        v as u64
    }
}
