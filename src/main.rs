#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

mod config;
use config::{Config, Mode};

fn main() -> Result<()> {
    let config_path = get_config_path()?;

    let config = if config_path.exists() {
        read_config(&config_path)?
    } else {
        let default_config = Config {
            mode: Mode::General,
        };
        write_config(&config_path, &default_config)?;
        println!("Конфигурационный файл создан. Пожалуйста, настройте его и перезапустите приложение.");
        return Ok(());
    };

    run_fix(&config)?;

    Ok(())
}

fn run_fix(config: &Config) -> Result<()> {
    let base_path = std::env::current_exe()?.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")); // Исправлено
    let bin_path = base_path.join("winws.exe");
    let quic_path = base_path.join("quic_initial_www_google_com.bin");
    let tls_path = base_path.join("tls_clienthello_www_google_com.bin");
    let list_path = base_path.join("list-discord.txt");


    let required_files = vec![
        ("winws.exe", &bin_path),
        ("quic_initial_www_google_com.bin", &quic_path),
        ("tls_clienthello_www_google_com.bin", &tls_path),
        ("list-discord.txt", &list_path),
    ];

    for (name, path) in required_files {
        if !path.exists() {
            anyhow::bail!("Не найден файл {}", name);
        }
    }

    let mut cmd = Command::new(&bin_path);
    cmd.current_dir(&base_path).creation_flags(0x08000000).stdout(Stdio::inherit()).stderr(Stdio::inherit()); // current_dir also takes a reference

    match config.mode {
        Mode::General => {
            cmd.args(&[
                "--wf-tcp=443",
                "--wf-udp=443,50000-65535",
                "--filter-udp=443",
                "--hostlist",
                list_path.to_str().unwrap_or(""),
                "--dpi-desync=fake",
                "--dpi-desync-udplen-increment=10",
                "--dpi-desync-repeats=6",
                "--dpi-desync-udplen-pattern=0xDEADBEEF",
                "--dpi-desync-fake-quic",
                quic_path.to_str().unwrap_or(""),
                "--new",
                "--filter-udp=50000-65535",
                "--dpi-desync=fake",
                "--dpi-desync-any-protocol",
                "--dpi-desync-cutoff=d3",
                "--dpi-desync-repeats=6",
                "--dpi-desync-fake-quic",
                quic_path.to_str().unwrap_or(""),
                "--new",
                "--filter-tcp=443",
                "--hostlist",
                list_path.to_str().unwrap_or(""),
                "--dpi-desync=fake,split",
                "--dpi-desync-autottl=2",
                "--dpi-desync-repeats=6",
                "--dpi-desync-fooling=badseq",
                "--dpi-desync-fake-tls",
                tls_path.to_str().unwrap_or(""),
            ]);
        },
        Mode::Beeline => {
            cmd.args(&[
                "--wf-udp=50000-65535",
                "--filter-udp=50000-65535",
                "--hostlist",
                list_path.to_str().unwrap_or(""),
                "--dpi-desync=fake",
                "--dpi-desync-any-protocol",
                "--dpi-desync-cutoff=d3",
                "--dpi-desync-repeats=6",
                "--dpi-desync-fake-quic",
                quic_path.to_str().unwrap_or(""),
                "--new",
                "--wf-l3=ipv4",
                "--wf-tcp=443",
                "--dpi-desync=syndata",
                "--dpi-desync-split2",
                "--dpi-desync-disorder2",
                "--dpi-desync-fake-syndata",
                tls_path.to_str().unwrap_or(""),
                "--wssize",
                "1:6",
                "--dpi-desync-fake-syndata=",
                tls_path.to_str().unwrap_or(""),
            ]);
        },
        Mode::Mgts => {
            cmd.args(&[
                "--wf-tcp=443",
                "--wf-udp=443,50000-65535",
                "--filter-udp=443",
                "--hostlist",
                list_path.to_str().unwrap_or(""),
                "--dpi-desync=fake",
                "--dpi-desync-udplen-increment=10",
                "--dpi-desync-repeats=6",
                "--dpi-desync-udplen-pattern=0xDEADBEEF",
                "--dpi-desync-fake-quic",
                quic_path.to_str().unwrap_or(""),
                "--new",
                "--filter-udp=50000-65535",
                "--dpi-desync=fake,tamper",
                "--dpi-desync-any-protocol",
                "--dpi-desync-cutoff=d3",
                "--dpi-desync-repeats=6",
                "--dpi-desync-fake-quic",
                quic_path.to_str().unwrap_or(""),
                "--new",
                "--filter-tcp=443",
                "--hostlist",
                list_path.to_str().unwrap_or(""),
                "--dpi-desync=fake",
                "--dpi-desync-autottl=2",
                "--dpi-desync-repeats=6",
                "--dpi-desync-fooling=md5sig",
                "--dpi-desync-fake-tls",
                tls_path.to_str().unwrap_or(""),
            ]);
        }
    }

    cmd.spawn().context("Не удалось запустить процесс")?;
    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let config_path = exe_path.with_extension("json");
    Ok(config_path)
}

fn read_config(path: &PathBuf) -> Result<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}


fn write_config(path: &PathBuf, config: &Config) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}