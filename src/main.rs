#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[macro_use]
extern crate wei_log;

#[tokio::main(flavor = "multi_thread", worker_threads = 100)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        if args[1] == "install" {
            use std::fs::OpenOptions;
            use std::io::{self, BufRead, Write};

            let bashrc_path = "/root/.bashrc";
            let export_path = "export PATH=$HOME/.wei/bin/data:$PATH";

            let file = OpenOptions::new().read(true).open(bashrc_path)?;
            let reader = io::BufReader::new(file);
            let mut found = false;

            for line in reader.lines() {
                let line = line?;
                if line.trim() == export_path {
                    found = true;
                    break;
                }
            }

            if !found {
                let mut file = OpenOptions::new().append(true).open(bashrc_path)?;
                writeln!(file, "{}", export_path)?;
            }
            // 获取当前WEI的执行目录
            let exe_path = std::env::current_exe()?;
            // 获取exe路径
            let exe_path = exe_path.parent().unwrap();

            let data = r#"
[Unit]
Description=wei

[Service]
Restart=always
RestartSec=30
TimeoutStartSec=0

User=root
ExecStartPre=-/usr/bin/killall wei
ExecStart=$PATH/wei
ExecStop=/usr/bin/killall wei

[Install]
WantedBy=multi-user.target
"#.replace("$PATH", exe_path.to_str().unwrap());
            std::fs::write("/etc/systemd/system/wei.service", data)?;
            wei_run::command("systemctl", vec!["daemon-reload"])?;
            wei_run::command("systemctl", vec!["enable", "wei"])?;
            wei_run::command("systemctl", vec!["restart", "wei"])?;

            println!("Install success!");

            std::process::exit(0);
        }
    }

    #[cfg(target_os = "windows")]
    match wei::init() {
        Ok(_) => {
            info!("init success");
        }
        Err(err) => {
            info!("init error: {}", err);
            println!("init error: {}", err);
            #[cfg(target_os = "windows")] {
                use tauri_winrt_notification::{Duration, Sound, Toast};
                Toast::new(Toast::POWERSHELL_APP_ID)
                .title("Wei")
                .text1(&err.to_string())
                .sound(Some(Sound::SMS))
                .duration(Duration::Short).show()?;
            }
        }
    };

    wei_windows::init();
    wei_env::bin_init("wei");
    let instance = wei_single::SingleInstance::new("wei")?;
    if !instance.is_single() { 
        #[cfg(target_os = "windows")] {
            use tauri_winrt_notification::{Duration, Sound, Toast};
            Toast::new(Toast::POWERSHELL_APP_ID)
            .title("Wei")
            .text1("已经存在相同的客户端软件，请检查托盘图标。")
            .sound(Some(Sound::SMS))
            .duration(Duration::Short).show()?;
        }

        std::process::exit(1);
    };

    info!("wei start");
    wei_env::start();

    info!("set_current_dir ./data");
    // 获取exe路径
    let exe_path = std::env::current_exe()?;
    // 设置exe路径为当前路径
    if let Some(parent_path) = exe_path.parent() {
        if let Err(e) = std::env::set_current_dir(parent_path) {
            info!("设置当前目录失败: {}, 路径: {:?}", e, parent_path);
            return Err(e.into());
        }
        info!("成功设置当前目录为: {:?}", parent_path);
    } else {
        info!("无法获取exe文件的父目录: {:?}", exe_path);
        return Err("无法获取exe文件的父目录".into());
    }
    match std::env::set_current_dir("./data") {
        Ok(_) => {
            info!("成功设置当前目录为 ./data");
        }
        Err(err) => {
            info!("设置当前目录为 ./data 失败: {}", err);
            return Err(err.into());
        }
    };

    info!("run wei-daemon");
    // 如果是windows系统则运行wei-daemon.ps1，其它系统则运行wei-daemon
    #[cfg(not(target_os = "windows"))]
    wei_run::run_async("wei-daemon", vec![])?;

    #[cfg(target_os = "windows")] {
        std::process::Command::new("powershell")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-File").arg("wei-daemon-close.ps1")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).output()?;

        std::process::Command::new("powershell")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-File").arg("wei-daemon.ps1")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).spawn()?;
    }

    // #[cfg(target_os = "windows")]
    // wei_run::run("wei-ui", vec![])?;

    // #[cfg(not(target_os = "windows"))]
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;
    }

    // #[cfg(target_os = "windows")]
    Ok(())
}
