fn main() {
    //link library 
    println!("cargo:rustc-link-search=rarezip/gzip", );
    println!("cargo:rustc-link-lib=rarezip", );
}