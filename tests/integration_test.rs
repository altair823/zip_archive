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

    let mut expected_messages = vec![
        "Total archive directory count: 3".to_string(),
        format!(
            "{} archiving complete: {}/dir1{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            format.extension()
        ),
        format!(
            "{} archiving complete: {}/dir2{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            format.extension()
        ),
        format!(
            "{} archiving complete: {}/dir3{}",
            format.to_string(),
            &dest.as_ref().to_str().unwrap(),
            format.extension()
        ),
        "Archiving Complete!".to_string(),
    ];

    messages.sort();
    expected_messages.sort();

    assert_eq!(messages, expected_messages);
}

fn assert_format<T: AsRef<Path>>(dest: T, format: Format) {
    assert!(dest
        .as_ref()
        .join(format!("dir1{}", format.extension()))
        .is_file());
    assert!(dest
        .as_ref()
        .join(format!("dir2{}", format.extension()))
        .is_file());
    assert!(dest
        .as_ref()
        .join(format!("dir3{}", format.extension()))
        .is_file());
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
    comp_test(Format::_7z);
    comp_test(Format::xz);
    comp_test(Format::zip);
}
