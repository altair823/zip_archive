# Zip archive

`zip_archive` is a library that select 7z executable that are depending on the operating system, and run it with multithread. 

## Examples

```
use std::path::PathBuf;
use zip_archive::archive_root_dir;

let origin = PathBuf::from("./origin");
let dest = PathBuf::from("./dest");
let thread_count = 4;

match archive_root_dir(origin, dest, thread_count){
    Ok(_) => (),
    Err(e) => println!("Cannot archive the directory! {}", e),
};
```

## Requirements

#### Windows 10

1. Install [7-Zip](https://www.7-zip.org/). 
2. Find 7z.exe file in installed program folder and add it to path. Or place it in project root folder. 

#### macOS

1. Download [7-Zip console version executable](https://www.7-zip.org/download.html) for macOS.
2. Place 7zz executable to home directory. 