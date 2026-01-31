use std::{
    env, fs,
    path::{Path, PathBuf},
    process, thread,
};

use serde_json;

use crate::{
    logging::{Log, Logger},
    update::sysinfo::end_electron_main,
};

use super::{
    callbacks::UpdateUi,
    ops::{check_permission, copy_file, flush_config_file, mark_update_myself_now},
    state::{RunningConfig, RunningState, UpdateConfigJson},
};

fn update(
    ui: &impl UpdateUi,
    exe_path_buf: PathBuf,
    skip_check: bool,
    mut running_config: RunningConfig,
    runnning_config_path: &Path,
) {
    let mut running_config_file = {
        if runnning_config_path.exists() {
            match fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(runnning_config_path)
            {
                Ok(file) => file,
                Err(e) => {
                    Log::error("打开运行状态文件失败");
                    Log::error(e.to_string().as_str());
                    ui.on_quit();
                    return;
                }
            }
        } else {
            match fs::File::create(runnning_config_path) {
                Ok(file) => file,
                Err(e) => {
                    Log::error("创建运行状态文件失败");
                    Log::error(e.to_string().as_str());
                    ui.on_quit();
                    return;
                }
            }
        }
    };
    running_config.status = RunningState::Updating;
    flush_config_file(&mut running_config_file, &running_config);
    let exe_path = exe_path_buf.as_path();
    Log::info("exe_path路径: ");
    Log::info(exe_path.to_string_lossy().as_ref());
    let path = match exe_path.parent() {
        Some(path) => path,
        None => {
            Log::error("无法获取根目录");
            ui.on_quit();
            return;
        }
    };
    Log::info("根目录: ");
    Log::info(path.to_string_lossy().as_ref());
    let update_temp_path = match env::var("update_temp_path") {
        Ok(path) if Path::new(&path).is_absolute() => Path::new(&path).to_owned(),
        _ => Path::new(&path).join("update_temp"),
    };
    Log::info("更新temp目录: ");
    Log::info(update_temp_path.to_string_lossy().as_ref());

    let update_config_file_name = match env::var("update_config_file_name") {
        Ok(name) => name,
        _ => "update-config.json".to_string(),
    };
    Log::info("配置update_config_file_name: ");
    Log::info(update_config_file_name.as_str());
    Log::info("读取更新配置：");
    Log::info("读取更新配置路径：");
    let update_config_path = update_temp_path.join(&update_config_file_name);
    Log::info(update_config_path.to_string_lossy().as_ref());
    running_config.update_temp_path = update_temp_path.to_string_lossy().to_string();
    flush_config_file(&mut running_config_file, &running_config);
    let config: UpdateConfigJson =
        match serde_json::from_slice(&fs::read(&update_config_path).unwrap_or_default()) {
            Ok(config) => config,
            _ => {
                Log::error("读取更新配置失败：");
                running_config.status = RunningState::Nothing;
                flush_config_file(&mut running_config_file, &running_config);
                ui.on_quit();
                return;
            }
        };
    Log::info("读取更新配置为：");
    Log::info(format!("{config:#?}").as_str());
    Log::info("开始更新");
    Log::info("处理未关闭的electron进程");
    end_electron_main(exe_path);
    if !skip_check {
        if !check_permission(&config, path, update_temp_path.as_path(), &mut running_config) {
            running_config.status = RunningState::Nothing;
            flush_config_file(&mut running_config_file, &running_config);
            Log::error("检测权限不通过，更新结束");
            ui.on_quit();
            return;
        };
        running_config.status = RunningState::Updating;
        flush_config_file(&mut running_config_file, &running_config);
    }

    Log::info("迁移文件");
    if !copy_file(
        &config,
        &path,
        &update_temp_path.as_path(),
        &mut running_config_file,
        &mut running_config,
        ui,
    ) {
        callback(&mut running_config_file, &mut running_config);
    } else {
        running_config.status = RunningState::Finish;
        flush_config_file(&mut running_config_file, &running_config);
        Log::info("迁移文件结束，更新完成");
        Log::info("清理更新文件");
        let update_myself_now = mark_update_myself_now();
        Log::info(&format!("set UPDATE_MYSELF_NOW {update_myself_now}"));
        std::thread::sleep(std::time::Duration::from_millis(500));
        match update_temp_path.file_name().and_then(|name| name.to_str()) {
            Some("update_temp") => {
                if let Err(e) = fs::remove_dir_all(update_temp_path) {
                    Log::error("清理更新文件出错：");
                    Log::error(e.to_string().as_str());
                }
            }
            _ => {
                Log::error("清理更新文件已跳过：update_temp_path 目录名不匹配 update_temp");
            }
        };
        Log::info("清理更新文件完成");
        Log::info("重启程序");
        let mut child = match process::Command::new(exe_path)
            .env("updateCallback", "success")
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                Log::error("重启程序失败");
                Log::error(e.to_string().as_str());
                ui.on_quit();
                return;
            }
        };
        thread::spawn(move || {
            let _ = child.wait();
        });
        Log::info("退出更新程序");
        ui.on_quit();
    }
}

fn callback(running_config_file: &mut fs::File, running_config: &mut RunningConfig) {
    let update_temp_path = Path::new(&running_config.update_temp_path);
    running_config.status = RunningState::Failed;
    flush_config_file(running_config_file, running_config);

    let update_temp_path_old_p = Path::new(update_temp_path).join(".update_temp_path_old_version");
    running_config.moved_path.iter().for_each(|i| {
        let p = Path::new(i);
        if let Err(e) = fs::remove_file(p) {
            Log::error("删除回退文件失败");
            Log::error(e.to_string().as_str());
        }
    });
    running_config.moved_path.clear();
    flush_config_file(running_config_file, running_config);

    running_config.file_path.iter().for_each(|(index, from)| {
        let to_path = update_temp_path_old_p.join(index.to_string());
        let from_path = PathBuf::from(from);
        if let Err(e) = fs::rename(to_path, from_path) {
            Log::error("回滚文件失败");
            Log::error(e.to_string().as_str());
        }
    });
    running_config.status = RunningState::Nothing;
    running_config.file_path.clear();
    flush_config_file(running_config_file, running_config);
}

pub fn run_task(ui: impl UpdateUi) {
    Log::setup_logging();
    Log::info("程序开始");
    Log::info("获取electron程序的执行目录,判断任务状态");
    let running_config_path = Path::new(".running_status");

    match env::var("exe_path") {
        Ok(path) if Path::new(&path).is_absolute() => {
            Log::info("执行更新程序");
            let exe_path_buf = Path::new(&path).to_owned();
            let config = RunningConfig {
                status: RunningState::UpdateButNotCheck,
                file_path: std::collections::HashMap::new(),
                exe_path: exe_path_buf.to_string_lossy().to_string(),
                update_temp_path: String::new(),
                moved_path: Vec::new(),
            };
            update(&ui, exe_path_buf, false, config, running_config_path);
        }
        _ => {
            Log::error("获取exe_path变量错误; 程序将退出");
            if !running_config_path.exists() {
                Log::info("程序无执行任务");
                ui.on_quit();
            } else {
                match serde_json::from_slice::<RunningConfig>(
                    &fs::read(running_config_path).unwrap_or_default(),
                ) {
                    Ok(mut config) if config.status == RunningState::Failed => {
                        let mut running_config_file = {
                            if running_config_path.exists() {
                                match fs::OpenOptions::new()
                                    .read(true)
                                    .write(true)
                                    .open(running_config_path)
                                {
                                    Ok(file) => file,
                                    Err(e) => {
                                        Log::error("打开运行状态文件失败");
                                        Log::error(e.to_string().as_str());
                                        ui.on_quit();
                                        return;
                                    }
                                }
                            } else {
                                match fs::File::create(running_config_path) {
                                    Ok(file) => file,
                                    Err(e) => {
                                        Log::error("创建运行状态文件失败");
                                        Log::error(e.to_string().as_str());
                                        ui.on_quit();
                                        return;
                                    }
                                }
                            }
                        };
                        callback(&mut running_config_file, &mut config);
                        ui.on_quit();
                    }
                    Ok(config) if config.status == RunningState::Updating => {
                        let exe_path_buf = PathBuf::from(&config.exe_path);
                        update(&ui, exe_path_buf, true, config, running_config_path);
                    }
                    Ok(config) if config.status == RunningState::UpdateButNotCheck => {
                        let exe_path_buf = PathBuf::from(&config.exe_path);
                        update(&ui, exe_path_buf, false, config, running_config_path);
                    }
                    Ok(_) => {
                        Log::info("程序无执行任务");
                        ui.on_quit();
                    }
                    _ => {
                        Log::error("读取运行配置失败：");
                        ui.on_quit();
                    }
                };
            }
        }
    };
}
