# Zip archive

`zip_archive` is a library that select 7z executable that are depending on the operating system,
and run it with multithread.

# Requirements

## Windows 10

1. Install [7-Zip](https://www.7-zip.org/).
2. Find 7z.exe file in installed 7z folder and add it to path.
Or place 7z.exe in project root folder.

## macOS

1. Download [7-Zip console version executable](https://www.7-zip.org/download.html) for macOS.
2. Place 7zz executable to home directory.

# Examples

- Compress root directory with multithread.
```rust
use std::path::PathBuf;
use zip_archive::Archiver;

let origin = PathBuf::from("./origin");
let dest = PathBuf::from("./dest");
let thread_count = 4;

let mut archiver = Archiver::new();
archiver.push(origin);
archiver.set_destination(dest);
archiver.set_thread_count(thread_count);

match archiver.archive(){
    Ok(_) => (),
    Err(e) => println!("Cannot archive the directory! {}", e),
};
```

- Compress each directory using the container's iterator.
```rust
use std::path::PathBuf;
use zip_archive::Archiver;

let origin = PathBuf::from("./origin");
let dest = PathBuf::from("./dest");

let mut archiver = Archiver::new();
archiver.push_from_iter(vec!["./origin/dir1", "./origin/dir2", "./origin/dir3"].into_iter());
archiver.set_destination(dest);
match archiver.archive(){
    Ok(_) => (),
    Err(e) => println!("Cannot archive the directory! {}", e),
};
```

- Compress subdirectories with a depth of 1.
```rust
use std::path::PathBuf;
use zip_archive::{Archiver, get_dir_list};

let origin = PathBuf::from("./origin");  // Change to the wanted directory.
let dest = PathBuf::from("./dest");

let mut archiver = Archiver::new();
archiver.push_from_iter(get_dir_list(origin).unwrap().iter());
archiver.set_destination(dest);
match archiver.archive(){
    Ok(_) => (),
    Err(e) => println!("Cannot archive the directory! {}", e),
}; 
```

