use crate::file_ops::get_home_dir;
use color_eyre::eyre::Result;
use log::debug;
use std::fs::{create_dir_all, read_dir, remove_file, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

// const BUF_SIZE: usize = 60;
const BUF_SIZE: usize = 64;
const BUF_OFFSET: usize = 4;

#[cfg(target_os = "linux")]
const DUMP_DIR: &str = "Documents/bytes";
#[cfg(target_os = "windows")]
const DUMP_DIR: &str = "Documents\\bytes";

pub fn align_num(val: String, padding: usize) -> String {
    let mut res = String::from("");
    if val.len() >= padding {
        val
    } else {
        for _ in 0..(padding - val.len()) {
            res.push(' ')
        }
        res.push_str(val.as_str());
        res
    }
}

pub fn buf_to_string_raw(msg_counter: u32, buf: &[u8]) -> String {
    let mut res = format!("[{}] ", align_num(format!("{}", msg_counter), 3));
    for (ind, num) in buf.iter().enumerate() {
        let num = align_num(format!("{:08b}", num), 8);
        res.push_str(format!("{}|", num).as_str());
    }
    res.push('\n');
    res
}

fn is_ending(buf: &[u8]) -> bool {
    let pad_data_start = BUF_OFFSET + 12;
    let pad_data_end = pad_data_start + 4;
    buf[pad_data_start..pad_data_end] == [0, 0, 0, 0]
}

pub fn buf_to_string(msg_counter: u32, buf: &[u8]) -> (String, String) {
    let res = buf_to_string_raw(msg_counter, buf);

    if is_ending(buf) {
        (format!("{}{}", res, get_header()), res)
    } else {
        (res, String::from(""))
    }
}

pub fn get_separator() -> String {
    let mut res = String::from("");
    for _ in 0..(BUF_SIZE * 9 + 5) {
        res.push('-');
    }
    res.push('\n');
    res
}

pub fn get_header() -> String {
    let mut content = get_separator();
    content.push_str("[   ] ");
    for i in 0..BUF_SIZE {
        content.push_str(format!("{}|", align_num(format!("{}", i), 8)).as_str());
    }
    content.push('\n');
    content.push_str(get_separator().as_str());
    content
}

pub fn clean_dir_fn(dir_path: PathBuf) -> Result<()> {
    for entry in read_dir(dir_path)? {
        remove_file(entry?.path())?;
    }
    Ok(())
}

pub fn create_file(subject: &str, endings: bool) -> Result<File> {
    let dir_path = get_home_dir()?.join(DUMP_DIR).join(subject);
    debug!("{:#?}", dir_path.as_os_str());
    create_dir_all(dir_path.clone())?;

    if !endings {
        clean_dir_fn(dir_path.clone())?;
    }

    let subject = if endings {
        format!("{}_endings", subject)
    } else {
        subject.to_string()
    };

    let mut file = OpenOptions::new()
        // .append(true)
        // .create(true)
        .write(true)
        .create_new(true)
        .open(dir_path.join(format!("{}.txt", subject)))?;
    file.write_all(get_header().as_bytes())?;

    Ok(file)
}

pub fn init_debug_files(is_left_pad_bytes_dump: bool) -> Result<(File, File, File)> {
    let subject = if is_left_pad_bytes_dump {
        "pad"
    } else {
        "stick"
    };

    let mut subject_file = create_file(subject, false)?;
    let mut subject_endings_file = create_file(subject, true)?;

    let mut cmp_file = create_file("cmp", false)?;
    cmp_file.write_all("\n".as_bytes())?;
    cmp_file.write_all(get_separator().as_bytes())?;

    Ok((subject_file, subject_endings_file, cmp_file))
}
