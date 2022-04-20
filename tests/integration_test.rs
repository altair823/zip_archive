mod common;
extern crate zip_archive;

use std::{path::Path, sync::mpsc::Receiver};

use function_name::named;

use crate::common::{cleanup, get_archiver, setup, Dir};
use zip_archive::Format;

fn assert_receiver<T: AsRef<Path>>(dest: T, receiver: Receiver<String>, format: Format) {
    let mut messages = Vec::new();
    for message in receiver {
        messages.push(message);
    }

    let extension = get_extension(format.clone());

    let mut expected_messages = vec![
        "Total archive directory count: 3".to_string(),
        format!(
            "{} archiving complete: {}/dir1{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            extension
        ),
        format!(
            "{} archiving complete: {}/dir2{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            extension
        ),
        format!(
            "{} archiving complete: {}/dir3{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            extension
        ),
        "Archiving Complete!".to_string(),
    ];

    messages.sort();
    expected_messages.sort();

    assert_eq!(messages, expected_messages);
}

fn assert_format<T: AsRef<Path>>(dest: T, format: Format) {
    let extension = get_extension(format);
    assert!(dest.as_ref().join(format!("dir1{}", extension)).is_file());
    assert!(dest.as_ref().join(format!("dir2{}", extension)).is_file());
    assert!(dest.as_ref().join(format!("dir3{}", extension)).is_file());
}

fn get_extension(format: Format) -> String {
    match format {
        Format::C7z => ".7z".to_string(),
        Format::Cxz => ".tar.xz".to_string(),
    }
}

#[named]
fn comp_test(format: Format) {
    let Dir {
        origin,
        dest,
        tx,
        tr,
    } = setup(&format!("{}_{}", function_name!(), format.to_string()));

    {
        let archiver = get_archiver(&origin, &dest, tx, format.clone());
        archiver.archive().unwrap();
    }

    assert_receiver(&dest, tr, format.clone());
    assert_format(&dest, format.clone());

    cleanup(&format!("{}_{}", function_name!(), format.to_string()));
}

#[test]
fn comp_test_all() {
    comp_test(Format::C7z);
    comp_test(Format::Cxz);
}
