# bk_asset_tool

extracts and constructs banjo-kazooie asset bins

# Requisites

### cargo:
```sh 
curl https://sh.rustup.rs -sSf | sh 
```

### rarezip crate:
```sh 
git submodule update --init --recursive 
```

# Building:
```sh
cargo build --release
```

# Usage:
### extract:
```sh 
bk_asset_tool <-e|--extract> <path/to/input.bin> <path/to/output/dir>
```

### construct:
```sh
bk_asset_tool <-c|--construct> <path/to/input.yaml> <path/to/output.bin>
```
