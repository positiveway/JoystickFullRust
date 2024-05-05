use std::fs::{read_dir, remove_file, File, OpenOptions};
use std::io::prelude::*;

const IS_PAD: bool = false;

const BUF_SIZE: usize = 60;
const BASE_PATH: &str = "/home/user/Documents/bytes";

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

pub fn buf_to_string_raw(msg_counter: u32, buf: Vec<u8>) -> String {
    let mut res = format!("[{}] ", align_num(format!("{}", msg_counter), 3));
    for (ind, num) in buf.iter().enumerate() {
        let num = align_num(format!("{:08b}", num), 8);
        res.push_str(format!("{}|", num).as_str());
    }
    res.push('\n');
    res
}

pub fn buf_to_string(msg_counter: u32, buf: Vec<u8>) -> (String, String) {
    let res = buf_to_string_raw(msg_counter, buf.clone());

    if &buf[12..=15] == [0, 0, 0, 0] {
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

pub fn clean_dir_fn(dir_path: String) -> color_eyre::Result<()> {
    for entry in read_dir(dir_path)? {
        remove_file(entry?.path())?;
    }
    Ok(())
}

pub fn create_file(subject: &str, endings: bool) -> color_eyre::Result<File> {
    let dir_path = format!("{}/{}", BASE_PATH, subject);

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
        .open(format!("{}/{}.txt", dir_path, subject))?;
    file.write_all(get_header().as_bytes())?;

    Ok(file)
}

pub fn init_debug_files() -> color_eyre::Result<(File, File, File)> {
    let subject = if IS_PAD { "pad" } else { "stick" };

    let mut subject_file = create_file(subject, false)?;
    let mut subject_endings_file = create_file(subject, true)?;

    let mut cmp_file = create_file("cmp", false)?;
    cmp_file.write_all("\n".as_bytes())?;
    cmp_file.write_all(get_separator().as_bytes())?;

    Ok((subject_file, subject_endings_file, cmp_file))
}
