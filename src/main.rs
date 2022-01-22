mod banjo_kazooie;

use std::env;
use std::fs::{self, DirBuilder};
use std::io::Write;
use std::path::Path;

enum Direction {
    Extract,
    Construct,
}

fn main() {
    //get inputs
    let arg1 = env::args().nth(1).expect("No input arguments provided");
    let direction = match arg1.as_str() {
        "--extract" | "-e" => Direction::Extract,
        "--construct" | "-c" => Direction::Construct,
        _=> panic!("invalid direction \"{}\" provided\n try: --extract, -e, --construct, or -c", arg1),
    };
    let in_path = env::args().nth(2).expect("No in path provided");
    let out_path = env::args().nth(3).expect("No out path provided");
    
    match direction {
        Direction::Extract => {
            // open asset binary
            assert!(fs::metadata(&in_path).unwrap().is_file());
            let in_bytes : Vec<u8> = fs::read(in_path).expect("Could not read file");
            
            // parse binary
            let af = banjo_kazooie::AssetFolder::from_bytes(&in_bytes);

            //create output
            DirBuilder::new().recursive(true).create(&out_path).unwrap();
            assert!(fs::metadata(&out_path).unwrap().is_dir());
            af.write(Path::new(&out_path));

        }
        Direction::Construct => {
            assert!(fs::metadata(&in_path).unwrap().is_file());
            let mut af = banjo_kazooie::AssetFolder::new();
            af.read(Path::new(&in_path));

            let mut decomp_buffer = af.to_bytes();
            decomp_buffer.resize((decomp_buffer.len() + 15) & !15, 0);
            let mut out_bin = fs::File::create(&out_path).expect("Could create output bin");
            out_bin.write_all(&decomp_buffer).unwrap();

        }
    }
}
