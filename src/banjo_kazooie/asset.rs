use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;

pub fn from_indx_and_bytes(i :usize, in_bytes: &[u8]) -> Box<dyn Asset>{
    return Box::new(Binary::from_bytes(in_bytes));
}

pub enum AssetType{
    Binary,
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

pub trait Asset {
    fn to_bytes(&self)->Vec<u8>;
    fn get_type(&self)->AssetType;
    fn write(&self, path: &Path);
}