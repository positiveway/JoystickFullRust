use color_eyre::eyre::{bail, OptionExt, Result};
use homedir::get_my_home;
use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

pub fn get_home_dir() -> Result<PathBuf> {
    Ok(get_my_home()?
        .ok_or_eyre("'Home' env var is not set")?
        .as_path()
        .to_path_buf())
}

pub fn last_path_component(path: &Path) -> Result<&str> {
    Ok(path
        .components()
        .last()
        .ok_or_eyre("Cannot get the last component of the path")?
        .as_os_str()
        .to_str()
        .ok_or_eyre("Cannot convert to str")?)
}

pub fn get_project_dir(project_name: &str) -> Result<PathBuf> {
    let mut cur_dir = current_dir()?;
    while last_path_component(cur_dir.as_path())? != project_name {
        cur_dir = cur_dir
            .parent()
            .ok_or_eyre("Cannot get parent directory")?
            .to_path_buf();
    }
    Ok(cur_dir)
}

pub fn read_toml<T, P, S>(folder: P, filename: S) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path>,
        S: AsRef<str>,
{
    const EXTENSION: &str = ".toml";
    let mut filename = filename.as_ref().to_string();
    if !filename.ends_with(EXTENSION) {
        filename += EXTENSION
    }

    let filepath = folder.as_ref().join(filename);
    let file_content = read_to_string(filepath)?;
    let decoded_obj = toml::from_str(file_content.as_str())?;
    Ok(decoded_obj)
}
