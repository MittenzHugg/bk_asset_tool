use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub fn from_indx_and_bytes(i :usize, in_bytes: &[u8]) -> Box<dyn Asset>{
    return match in_bytes {
        [0x01, 0x01, 0x02, 0x05, 0x00, ..] => Box::new(QuizQuestion::from_bytes(in_bytes)),
        [0x01, 0x03, 0x00, 0x05, 0x00, ..] => Box::new(GruntyQuestion::from_bytes(in_bytes)),
        [0x01, 0x03, 0x00,..] => {
            Box::new(Dialog::from_bytes(in_bytes))},
        _ => Box::new(Binary::from_bytes(in_bytes)),
    }
}

pub enum AssetType{
    Binary,
    Dialog,
    GruntyQuestion,
    QuizQuestion,
}

pub struct Binary{
    bytes: Vec<u8>,
}

impl Binary{
    pub fn from_bytes(in_bytes: &[u8])->Binary{
        Binary{bytes: in_bytes.to_vec()}
    }

    pub fn read(path: &Path) -> Binary{
        Binary{bytes: fs::read(path).unwrap()}
    }
}

impl Asset for Binary{
    fn to_bytes(&self)->Vec<u8>{
        return self.bytes.clone();
    }

    fn get_type(&self)->AssetType{
        return AssetType::Binary;
    }

    fn write(&self, path: &Path){
        let mut bin_file = File::create(path).unwrap();
        bin_file.write_all(&self.bytes).unwrap();
    }
}

#[derive(Clone)]
struct BKString{
    cmd: u8,
    string: Vec<u8>,
}

impl BKString{
    pub fn from_yaml(yaml: &Yaml) -> BKString{
        let cmd = yaml["cmd"].as_i64().unwrap() as u8;
        let string = string_to_vecu8(&yaml["string"].as_str().unwrap());            
        
        BKString{cmd : cmd, string: string}
    }
}

pub struct Dialog{
    bottom: Vec<BKString>,
    top: Vec<BKString>,
}

impl Dialog{
    pub fn from_bytes(in_bytes: &[u8])->Dialog{
        let mut offset : usize = 3;
            
        let mut bottom = Vec::new();
        let bottom_size : u8 = in_bytes[offset];
        offset += 1;
        let mut i = 0;
        for i in 0..bottom_size{
            let cmd : u8 = in_bytes[offset];
            let str_size : u8 = in_bytes[offset + 1];
            let i_string = BKString{cmd : cmd, string : in_bytes[offset + 2 .. offset + 2 + str_size as usize].to_vec()};
            bottom.push(i_string);
            offset += 2 + str_size as usize;
        }

        let mut top = Vec::new();
        let top_size : u8 = in_bytes[offset];
        offset += 1;
        let mut i = 0;
        for i in 0..top_size{
            let cmd : u8 = in_bytes[offset];
            let str_size : u8 = in_bytes[offset + 1];
            let i_string = BKString{cmd : cmd, string : in_bytes[offset + 2 .. offset + 2 + str_size as usize].to_vec()};
            top.push(i_string);
            offset += 2 + str_size as usize;
        }

        return Dialog{ bottom: bottom, top: top,};
    }

    pub fn read(path: &Path) -> Dialog{
        let doc = &YamlLoader::load_from_str(&fs::read_to_string(path).expect("could not open yaml")).unwrap()[0];
        let doc_type = doc["type"].as_str().unwrap();
        assert_eq!(doc_type, "Dialog");
        let bottom_obj = doc["bottom"].as_vec().unwrap();
        let bottom : Vec<BKString> = bottom_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        let top_obj = doc["top"].as_vec().unwrap();
        let top : Vec<BKString> = top_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        Dialog{bottom: bottom, top: top}
    }
}

impl Asset for Dialog{
    fn to_bytes(&self)->Vec<u8>{
        let mut out :Vec<u8> = vec![0x01, 0x03, 0x00];
        out.push(self.bottom.len() as u8);
        for text in self.bottom.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        out.push(self.top.len() as u8);
        for text in self.top.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        return out;
    }

    fn get_type(&self)->AssetType{
        return AssetType::Dialog;
    }

    fn write(&self, path: &Path){
        let mut bin_file = File::create(path).unwrap();
        
        writeln!(bin_file, "type: Dialog").unwrap();
        writeln!(bin_file, "bottom:").unwrap();
        for text in self.bottom.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
        writeln!(bin_file, "top:").unwrap();
        for text in self.top.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
    }
}

pub struct QuizQuestion{
    question: Vec<BKString>,
    options: [BKString; 3],
}

impl QuizQuestion{
    pub fn from_bytes(in_bytes: &[u8])->QuizQuestion{
        let mut texts = Vec::new();
        let mut str_cnt = in_bytes[5];
        let mut offset : usize = 6;
        for _i in 0..str_cnt{
            let cmd : u8 = in_bytes[offset];
            let str_size : u8 = in_bytes[offset + 1];
            let i_string = BKString{cmd : cmd, string : in_bytes[offset + 2 .. offset + 2 + str_size as usize].to_vec()};
            texts.push(i_string);
            offset += 2 + str_size as usize;
        }
        let (q_text, o_text) = texts.split_at(texts.len() - 3); 

        let options : [BKString; 3] = [o_text[0].clone(), o_text[1].clone(), o_text[2].clone()];
        return QuizQuestion{ question: q_text.to_vec(), options: options};
    }

    pub fn read(path: &Path) -> QuizQuestion{
        let doc = &YamlLoader::load_from_str(&fs::read_to_string(path).expect("could not open yaml")).unwrap()[0];
        let doc_type = doc["type"].as_str().unwrap();
        assert_eq!(doc_type, "QuizQuestion");
        let q_obj = doc["question"].as_vec().unwrap();
        let q : Vec<BKString> = q_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        let a_obj = doc["options"].as_vec().unwrap();
        let a : Vec<BKString> = a_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        let options : [BKString; 3] = [a[0].clone(), a[1].clone(), a[2].clone()];

        QuizQuestion{question: q, options: options}
    }
}

impl Asset for QuizQuestion{
    fn to_bytes(&self)->Vec<u8>{
        let mut out :Vec<u8> = vec![0x01, 0x03, 0x00, 0x05, 0x00];
        out.push((self.question.len() + self.options.len()) as u8);
        for text in self.question.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        for text in self.options.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        return out;
    }
    
    fn get_type(&self)->AssetType{
        return AssetType::QuizQuestion
    }

    fn write(&self, path: &Path){
        let mut bin_file = File::create(path).unwrap();
        
        writeln!(bin_file, "type: QuizQuestion").unwrap();
        writeln!(bin_file, "question:").unwrap();
        for text in self.question.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
        writeln!(bin_file, "options:").unwrap();
        for text in self.options.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
    }
}

pub struct GruntyQuestion{
    question: Vec<BKString>,
    options: [BKString; 3],
}

impl GruntyQuestion{
    pub fn from_bytes(in_bytes: &[u8])->GruntyQuestion{
        let mut texts = Vec::new();
        let mut str_cnt = in_bytes[5];
        let mut offset : usize = 6;
        for _i in 0..str_cnt{
            let cmd : u8 = in_bytes[offset];
            let str_size : u8 = in_bytes[offset + 1];
            let i_string = BKString{cmd : cmd, string : in_bytes[offset + 2 .. offset + 2 + str_size as usize].to_vec()};
            texts.push(i_string);
            offset += 2 + str_size as usize;
        }
        let (q_text, o_text) = texts.split_at(texts.len() - 3); 

        let options : [BKString; 3] = [o_text[0].clone(), o_text[1].clone(), o_text[2].clone()];
        return GruntyQuestion{ question: q_text.to_vec(), options: options};
    }

    pub fn read(path: &Path) -> GruntyQuestion{
        let doc = &YamlLoader::load_from_str(&fs::read_to_string(path).expect("could not open yaml")).unwrap()[0];
        let doc_type = doc["type"].as_str().unwrap();
        assert_eq!(doc_type, "GruntyQuestion");
        let q_obj = doc["question"].as_vec().unwrap();
        let q : Vec<BKString> = q_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        let a_obj = doc["options"].as_vec().unwrap();
        let a : Vec<BKString> = a_obj.iter()
            .map(|y|{BKString::from_yaml(y)})
            .collect();

        let options : [BKString; 3] = [a[0].clone(), a[1].clone(), a[2].clone()];

        GruntyQuestion{question: q, options: options}
    }
}

impl Asset for GruntyQuestion{
    fn to_bytes(&self)->Vec<u8>{
        let mut out :Vec<u8> = vec![0x01, 0x03, 0x00, 0x05, 0x00];
        out.push((self.question.len() + self.options.len()) as u8);
        for text in self.question.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        for text in self.options.iter(){
            out.push(text.cmd);
            out.push(text.string.len() as u8);
            out.append(&mut text.string.clone());
        }
        return out;
    }
    
    fn get_type(&self)->AssetType{
        return AssetType::GruntyQuestion
    }

    fn write(&self, path: &Path){
        let mut bin_file = File::create(path).unwrap();
        
        writeln!(bin_file, "type: GruntyQuestion").unwrap();
        writeln!(bin_file, "question:").unwrap();
        for text in self.question.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
        writeln!(bin_file, "options:").unwrap();
        for text in self.options.iter(){
            writeln!(bin_file,"  - {{ cmd: 0x{:02X}, string: \"{}\"}}", text.cmd, vecu8_to_string(&text.string)).unwrap()
        }
    }
}

pub trait Asset {
    fn to_bytes(&self)->Vec<u8>;
    fn get_type(&self)->AssetType;
    fn write(&self, path: &Path);
}

fn string_to_vecu8(string: &str) -> Vec<u8>{
    let mut string = string.as_bytes().to_vec();
    let mut squig_indx : Vec<usize> = string.windows(2)
        .enumerate()
        .filter(|(_, win)|{match win {[0xC3, 0xBD]=> true, _=>false,} })
        .map(|(i, _)|{i})
        .collect();
    squig_indx.reverse();
    for i in squig_indx{
        string[i] = 0xFD;
        string.remove(i+1);
    }
    string.push(0);
    return string
}

fn vecu8_to_string(bytes: &Vec<u8>) -> String{
    let mut out : String = String::new();
    for b in &bytes[..bytes.len() - 1]{
        let ch = *b as char;
        if !ch.is_ascii() || *b < 0x20 {
            out += format!("\\x{:02X}", ch as u8).as_str();
        }
        else{
            out.push(ch);
        }
    }
    return out
}
