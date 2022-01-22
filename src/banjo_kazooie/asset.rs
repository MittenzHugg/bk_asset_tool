use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub fn from_indx_and_bytes(segment :usize, in_bytes: &[u8]) -> Box<dyn Asset>{
    return match segment{
        //0 => animations
        //1 => models and sprites
        //2 => map setup
        //3 => sprites
        4 => match in_bytes { //Dialog, GruntyQuestions, QuizQuestions, DemoButtonFiles
                [0x01, 0x01, 0x02, 0x05, 0x00, ..] => Box::new(QuizQuestion::from_bytes(in_bytes)),
                [0x01, 0x03, 0x00, 0x05, 0x00, ..] => Box::new(GruntyQuestion::from_bytes(in_bytes)),
                [0x01, 0x03, 0x00,..] => Box::new(Dialog::from_bytes(in_bytes)),
                _ => Box::new(DemoButtonFile::from_bytes(in_bytes)),
            },
        //5 => level_models
        //6 => midi seq
        _ => Box::new(Binary::from_bytes(in_bytes)),
    }
}

pub enum AssetType{
    Binary,
    Dialog,
    GruntyQuestion,
    QuizQuestion,
    DemoInput,
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
        let mut out :Vec<u8> = vec![0x01, 0x01, 0x02, 0x05, 0x00];
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

struct ContInput{
    x: i8,
    y: i8,
    buttons: u16,
    frames: u8,
}

impl ContInput{
    fn to_bytes(&self)->Vec<u8>{
        let b = self.buttons.to_be_bytes();
        return vec![self.x as u8, self.y as u8, b[0], b[1], self.frames, 0x00];
    }

    fn from_yaml(yaml: &Yaml)->ContInput{
        let x = yaml["x"].as_i64().unwrap() as i8;
        let y = yaml["y"].as_i64().unwrap() as i8;
        let buttons = yaml["buttons"].as_i64().unwrap() as u16;
        let frames = yaml["frames"].as_i64().unwrap() as u8;
        return ContInput{x: x, y: y, buttons: buttons, frames: frames}
    }
}

/// ToDo:
///     - read
pub struct DemoButtonFile{
    inputs: Vec<ContInput>,
    frame1_flag: u8,
    bytes: Vec<u8>, //remove once conversion confirmed correct
}

impl DemoButtonFile{
    pub fn from_bytes(in_bytes: &[u8])->DemoButtonFile{
        if in_bytes.len() < 4 { return DemoButtonFile{inputs: Vec::new(), frame1_flag: 0, bytes: in_bytes.to_vec()}}
        let expect_len : usize =  u32::from_be_bytes(in_bytes[..4].try_into().unwrap()) as usize;
        let f1f = in_bytes[9];
        let inputs : Vec<ContInput> = in_bytes[4..].chunks_exact(6)
            .map(|a|{
                ContInput{
                    x : a[0] as i8, 
                    y : a[1] as i8,
                    buttons : u16::from_be_bytes([a[2], a[3]]),
                    frames : a[4],
                }
            })
            .collect();
        assert_eq!(expect_len, inputs.len()*6);
        DemoButtonFile{inputs: inputs, frame1_flag: f1f, bytes: in_bytes.to_vec()}
    }

    pub fn read(path: &Path) -> DemoButtonFile{
        // return DemoButtonFile{inputs: Vec::new(), frame1_flag: 0, bytes: Vec::new()};
        
        let doc = &YamlLoader::load_from_str(&fs::read_to_string(path).expect("could not open yaml")).unwrap()[0];
        let doc_type = doc["type"].as_str().unwrap();
        let f1f = doc["flag"].as_i64().unwrap() as u8;
        assert_eq!(doc_type, "DemoInput");
        
        let inputs_yaml = doc["inputs"].as_vec().unwrap();
        let mut inputs : Vec<ContInput> = inputs_yaml.iter().map(|y|{
            ContInput::from_yaml(y)
        })
        .collect();
        return DemoButtonFile{inputs:inputs, frame1_flag: f1f, bytes: Vec::new()}
    }
}

impl Asset for DemoButtonFile{
    fn to_bytes(&self)->Vec<u8>{
        if self.inputs.is_empty() { return Vec::new(); }

        let mut output : Vec<u8> = (6*self.inputs.len() as u32).to_be_bytes().to_vec();
        let mut input_bytes : Vec<u8> = self.inputs.iter().map(|i|{
            i.to_bytes()
        })
        .flatten()
        .collect();
        input_bytes[5] = self.frame1_flag;
        output.append(&mut input_bytes);
        return output;
    }

    fn get_type(&self)->AssetType{
        return AssetType::DemoInput;
    }

    fn write(&self, path: &Path){
        //bin output //REMOVE once matching read is created
        // let mut bin_file = File::create(path).unwrap();
        // bin_file.write_all(&self.bytes).unwrap();

        //.demo_output
        let mut demo_path = path.parent().unwrap().join(path.file_stem().unwrap());
        demo_path.set_extension("demo");
        let mut demo_file = File::create(demo_path).unwrap();
        writeln!(demo_file, "type: DemoInput").unwrap();
        writeln!(demo_file, "flag: 0x{:02X}", self.frame1_flag).unwrap();
        if(self.inputs.len() == 0){
            writeln!(demo_file, "inputs: []").unwrap();
            return;
        }
        writeln!(demo_file, "inputs:").unwrap();
        for input in self.inputs.iter(){
            writeln!(demo_file, "  - {{x: {:3}, y: {:3}, buttons: 0x{:04X}, frames: {}}}", input.x, input.y, input.buttons, input.frames).unwrap();
        }
    }
}
