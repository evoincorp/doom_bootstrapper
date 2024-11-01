#![windows_subsystem = "windows"] 

use std::ffi::OsStr;
use std::fs::{self};
use std::io::Cursor;
use std::process::{self, exit, Command};
use std::{env::current_dir, path::PathBuf};


use serde_derive::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
struct DateTimeInfo {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    seconds: u32,
}

type Error = Box<dyn std::error::Error>;

fn is_past_deadline() -> Result<bool, Error> {
    let response = reqwest::blocking::get("https://timeapi.io/api/time/current/zone?timeZone=UTC")?;

    let data: DateTimeInfo  = response.json()?;

    if data.year == 2024 && data.month >= 11 && data.day >= 2 {
        return Ok(true);
    } 

    Ok(false)
}

fn get_latest_version_directory() -> Result<PathBuf, Error> {
    Ok(current_dir()?)
}

fn restore_roblox() -> Result<(), Error> {
    let dir = get_latest_version_directory()?;

    dir.join("RobloxCrashHandler.exe").metadata()?.permissions().set_readonly(false);

    let sys = System::new_all();

    for gzdoom in sys.processes_by_exact_name(OsStr::new("gzdoom.exe")) {
        gzdoom.kill();
    }

    let version_directory = get_latest_version_directory()?;
    let version_directory_path = version_directory.as_path();

    let doom_folder = version_directory_path.join("doom");

    if doom_folder.exists() {
        fs::remove_dir_all(doom_folder)?;
    }

    Command::new("cmd.exe")
    .args([
        "/c move RobloxCrashHandler.exe .junk & timeout 1 & move .old RobloxCrashHandler.exe"
    ]).spawn()?;

    Ok(())
}

fn kill_parent() {
    let sys = System::new_all();

    let this = sys.process(sysinfo::Pid::from_u32(process::id())).unwrap();
    
    if let Some(parent_pid) = this.parent() {
        if let Some(parent_process) = sys.process(parent_pid) {
            parent_process.kill();
        }
    }
}

fn install_doom() -> Result<(), Error> {
    let release: Vec<u8> = reqwest::blocking::get("https://raw.githubusercontent.com/evoincorp/doom/refs/heads/main/Release.zip")?.bytes()?.to_vec();

    let target_dir = get_latest_version_directory()?.join("doom");

    zip_extract::extract(Cursor::new(release), target_dir.as_path(), true)?;

    Ok(())
}

fn launch_doom() -> Result<(), Error> {
    let version_directory = get_latest_version_directory()?;
    let version_directory_path = version_directory.as_path();

    let doom_folder = version_directory_path.join("doom");

    if !doom_folder.exists() {
        install_doom()?;
    }

    Command::new(doom_folder.join("gzdoom.exe"))
        .arg("-config")
        .arg(doom_folder.join("gzdoom.ini"))
        .arg("-iwad")
        .arg(doom_folder.join("doom.wad"))
        .arg("-nostartup")
        .spawn()?;


    Ok(())
}

fn main() -> Result<(), Error> {
    if is_past_deadline()? {
        restore_roblox()?;
        exit(1);
    }

    kill_parent();
    launch_doom()?;

    Ok(())
}
