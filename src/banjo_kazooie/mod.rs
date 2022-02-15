use std::convert::TryInto;
use std::fs::{self, DirBuilder};
use std::io::{Write, Read};
use std::path::Path;
use yaml_rust::{YamlLoader,Yaml};

use rarezip::bk;

pub mod asset;

#[derive(Clone, Copy)]
struct AssetMeta{
    pub offset : usize,
    pub c_flag : bool,
    pub t_flag : u16,
}

impl AssetMeta {
    pub fn from_bytes(in_bytes: &[u8])->AssetMeta{
        let offset = u32::from_be_bytes([in_bytes[0], in_bytes[1], in_bytes[2], in_bytes[3]]);
        let c_flag = u16::from_be_bytes([in_bytes[4], in_bytes[5]]);
        let t_flag = u16::from_be_bytes([in_bytes[6], in_bytes[7]]);
        return AssetMeta{offset: offset as usize, c_flag: c_flag != 0, t_flag: t_flag}
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        let mut out : Vec<u8> = (self.offset as u32).to_be_bytes().to_vec();
        out.push(0x00);
        out.push(self.c_flag as u8);
        out.append(&mut self.t_flag.to_be_bytes().to_vec());
        return out;
    }
}

// struct containing metadata and maybe a dyn asset::Asset
struct AssetEntry{
    pub uid  : usize,
    pub seg : usize,
    pub meta : AssetMeta,
    pub data : Option<Box<dyn asset::Asset>>
}

impl AssetEntry{
    pub fn new(uid:usize)->AssetEntry{
        AssetEntry{uid: uid, seg: 0, meta: AssetMeta{offset:0, c_flag:false, t_flag:4}, data: None}
    }

    pub fn from_yaml(yaml:&Yaml)->AssetEntry{
        assert!(yaml["uid"].as_i64().is_some(),"could not read uid as interger");
        let uid = yaml["uid"].as_i64().unwrap() as usize;
        let c_type : bool = yaml["compressed"].as_bool().unwrap();
        let t_type : u16 = yaml["flags"].as_i64().unwrap() as u16;
        let meta = AssetMeta{offset: 0, c_flag: c_type , t_flag: t_type };
        AssetEntry{meta: meta, ..AssetEntry::new(uid)}
    }
}

pub struct AssetFolder{
    assets : Vec<AssetEntry>
}

impl AssetFolder{
    pub fn new() -> AssetFolder{
        return AssetFolder{assets: Vec::new()}
    }

    pub fn from_bytes(in_bytes: &[u8]) -> AssetFolder{
        let asset_slot_cnt : usize = u32::from_be_bytes(in_bytes[..4].try_into().unwrap()) as usize;
        let (table_bytes, data_bytes) = in_bytes[8..].split_at(8*asset_slot_cnt);

        let meta_info : Vec<AssetMeta> = table_bytes.chunks_exact(8).map(|chunk| {AssetMeta::from_bytes(chunk)}).collect();
        let mut segment : usize = 0; //segment number + 1
        let mut prev_t : u16 = 0x3; //used for segment_detection
        let asset_list : Vec<AssetEntry> = meta_info.windows(2).enumerate().map(|(i, window)|{
            let this = &window[0];
            let next = &window[1];

            if this.t_flag == 4{ //empty entry
                return AssetEntry{uid : i, seg : 0, meta : this.clone(), data : None};
            }
            else if (this.t_flag != 2)
                    && (prev_t & 2) != (this.t_flag & 2)
            {
                segment += 1;
                prev_t = this.t_flag;
            }

            //decompress
            let comp_bin = &data_bytes[this.offset.. next.offset];
            let decomp_bin = match this.c_flag {
                true  => bk::unzip(comp_bin),
                false => comp_bin.to_vec(),
            };
            let this_asset = asset::from_seg_indx_and_bytes(segment, i, &decomp_bin);
            let out = AssetEntry{uid : i, seg :segment, meta : this.clone(), data : Some(this_asset)};
            return out
        }).collect();


        return AssetFolder{assets: asset_list};
    }

    pub fn to_bytes(&mut self) -> Vec<u8>{
        if self.assets.last().unwrap().data.is_some(){
            self.assets.push(AssetEntry::new(self.assets.len())); //used to make table length correct
        }

        //get compressed version if compressed
        let comp_bins: Vec<Vec<u8>> = self.assets.iter().map(|a|{
            return match &a.data {
                None => Vec::new(),
                Some(ass) => {
                    match &a.meta.c_flag{
                        true => bk::zip(&ass.to_bytes()),
                        false => ass.to_bytes(),
                    }
                },
            }
        })
        .collect();

        //update asset offsets
        let data_offsets: Vec<usize> = comp_bins.iter().map(|v| v.len()).collect();
        self.assets.iter_mut().zip(data_offsets.iter()).fold(0, |o, (a, s)|{
            a.meta.offset = o;
            return o + *s;
        });

        //convert everything to bytes
        let mut out : Vec<u8> = ((self.assets.len()) as u32).to_be_bytes().to_vec();
        out.append(&mut vec![0xff, 0xff, 0xff, 0xff]);

        let mut meta_bytes : Vec<u8> = self.assets.iter()
            .map(|a|{return a.meta.to_bytes()})
            .flatten()
            .collect();

        let mut data_bytes: Vec<u8> = comp_bins.into_iter().flatten().collect();

        out.append(&mut meta_bytes);
        out.append(&mut data_bytes);
        self.assets.pop();
        return out;
    }

    pub fn write(&self, out_dir_path: &Path){
        let asset_yaml_path = out_dir_path.join("assets.yaml");

        //write assets.yaml
        let mut asset_yaml = fs::File::create(&asset_yaml_path).expect("could not write file");
        

        //assets.to_file
        writeln!(asset_yaml, "tbl_len: 0x{:X}", self.assets.len() + 1).unwrap();
        writeln!(asset_yaml, "files:").unwrap();
        for elem in self.assets.iter()
            .filter(|a| match a.data {None => false, _ => true})
        {
            
            let data = match &elem.data {
                Some(x) => x,
                None => panic!("None data element reached"),
            };
            let mut tmp_str: String;
            let data_type_str = match data.get_type(){
                asset::AssetType::Animation => "Animation",
                asset::AssetType::Binary => "Binary",
                asset::AssetType::DemoInput => "DemoInput",
                asset::AssetType::Dialog => "Dialog",
                asset::AssetType::GruntyQuestion => "GruntyQuestion",
                asset::AssetType::Midi => "Midi",
                asset::AssetType::Model => "Model",
                asset::AssetType::LevelSetup => "LevelSetup",
                asset::AssetType::QuizQuestion => "QuizQuestion",
                asset::AssetType::Sprite(fmt) => {let f = format!("{:?}",fmt).to_uppercase(); tmp_str = String::from("Sprite_") + &f; &tmp_str},
                _ => "Binary",
            };
            let mut tmp_str2: String;
            let file_ext = match data.get_type(){
                asset::AssetType::Binary => ".bin",
                asset::AssetType::Dialog => ".dialog",
                asset::AssetType::GruntyQuestion => ".grunty_q",
                asset::AssetType::QuizQuestion => ".quiz_q",
                asset::AssetType::DemoInput => ".demo",
                asset::AssetType::Midi => ".midi.bin",
                asset::AssetType::Model => ".model.bin",
                asset::AssetType::LevelSetup => ".lvl_setup.bin",
                asset::AssetType::Animation => ".anim.bin",
                asset::AssetType::Sprite(fmt) => {tmp_str2 = format!(".sprite.{:?}.bin",fmt).to_lowercase(); &tmp_str2.as_str()},
                _ => ".bin"
            };
            let containing_folder = match data.get_type(){
                asset::AssetType::Binary => "bin",
                asset::AssetType::Dialog => "dialog",
                asset::AssetType::GruntyQuestion => "grunty_q",
                asset::AssetType::QuizQuestion => "quiz_q",
                asset::AssetType::DemoInput => "demo",
                asset::AssetType::Midi => "midi",
                asset::AssetType::Model => "model",
                asset::AssetType::LevelSetup => "lvl_setup",
                asset::AssetType::Animation => "anim",
                asset::AssetType::Sprite(fmt) => "sprite",
                _ => "bin"
            };

            let elem_folder = out_dir_path.join(containing_folder);
            DirBuilder::new().recursive(true).create(&elem_folder).unwrap();
            assert!(fs::metadata(&elem_folder).unwrap().is_dir());
            
            let elem_path = elem_folder.join(format!("{:04X}{}", elem.uid, file_ext));
            let relative_path = elem_path.strip_prefix(out_dir_path).unwrap().to_str().unwrap();
            writeln!(asset_yaml, "  - {{uid: 0x{:04X}, type: {:6}, compressed: {:5}, flags: 0x{:04X}, relative_path: {:?}}}", elem.uid, data_type_str, elem.meta.c_flag, elem.meta.t_flag, relative_path).unwrap();
        
            data.write(&elem_path);
        }


    }

    pub fn read(&mut self, yaml_path: &Path){
        assert_eq!(yaml_path.extension().unwrap(), "yaml");
        let containing_folder = yaml_path.parent().unwrap();
        let base_name = yaml_path.file_stem().unwrap();
        
        let doc = &YamlLoader::load_from_str(&fs::read_to_string(yaml_path).expect("could not open yaml")).unwrap()[0];

        let asset_meta : Vec<AssetEntry> = doc["files"].as_vec().unwrap()
            .iter()
            .map(|y|{ AssetEntry::from_yaml(y)})
            .collect();
        let expect_len = doc["tbl_len"].as_i64().unwrap() as usize;
        let max_id :usize = asset_meta.iter().fold(0, |max, a|{
            return if max > a.uid {max} else {a.uid}
        });

        let expect_len = if expect_len < max_id + 1 {max_id + 1} else {expect_len};

        if self.assets.len() < expect_len {
            let mut i = 0;
            self.assets.resize_with(expect_len, ||{ let j = i; i += 1; return AssetEntry::new(j)})
        }

        for a in asset_meta.into_iter(){
            let i = a.uid.clone();
            self.assets[i] = a;
        }

        for y in doc["files"].as_vec().unwrap().iter(){
            let uid :usize = y["uid"].as_i64().unwrap() as usize;
            let relative_path = y["relative_path"].as_str().unwrap();
            let data :Option<Box<dyn asset::Asset>> = match y["type"].as_str().unwrap(){
                "Binary"            => Some(Box::new(asset::Binary::read(&containing_folder.join(relative_path)))),
                "Dialog"            => Some(Box::new(asset::Dialog::read(&containing_folder.join(relative_path)))),
                "GruntyQuestion"    => Some(Box::new(asset::GruntyQuestion::read(&containing_folder.join(relative_path)))),
                "QuizQuestion"      => Some(Box::new(asset::QuizQuestion::read(&containing_folder.join(relative_path)))),
                "DemoInput"         => Some(Box::new(asset::DemoButtonFile::read(&containing_folder.join(relative_path)))),
                // "Midi"              => Some(Box::new(asset::MidiSeqFile::read(&containing_folder.join(relative_path)))),
                // "Model"             => Some(Box::new(asset::Model::read(&containing_folder.join(relative_path)))),
                // "LevelSetup"        => Some(Box::new(asset::LevelSetup::read(&containing_folder.join(relative_path)))),
                // "Animation"         => Some(Box::new(asset::Animation::read(&containing_folder.join(relative_path)))),
                // x if x.starts_with("Sprite_") => Some(Box::new(asset::Sprite::read(&containing_folder.join(relative_path)))),
                _ => Some(Box::new(asset::Binary::read(&containing_folder.join(relative_path)))),
            };
            self.assets[uid].data = data;
        }
    }
}
