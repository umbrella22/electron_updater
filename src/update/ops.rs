use std::{
    env, fs,
    io::{Seek, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use serde_json;

use crate::logging::{Log, Logger};

use super::{
    callbacks::UpdateUi,
    state::{RunningConfig, RunningState, UpdateConfigJson},
};

static NEED_UPDATE_MYSELF: AtomicBool = AtomicBool::new(false);
static UPDATE_MYSELF_NOW: AtomicBool = AtomicBool::new(false);

fn set_need_update_myself(value: bool) {
    NEED_UPDATE_MYSELF.store(value, Ordering::SeqCst);
}

fn need_update_myself() -> bool {
    NEED_UPDATE_MYSELF.load(Ordering::SeqCst)
}

fn update_myself_now() -> bool {
    UPDATE_MYSELF_NOW.load(Ordering::SeqCst)
}

pub(crate) fn mark_update_myself_now() -> bool {
    UPDATE_MYSELF_NOW.store(true, Ordering::SeqCst);
    UPDATE_MYSELF_NOW.load(Ordering::SeqCst)
}

pub(crate) fn check_permission<P: AsRef<Path>>(
    config: &UpdateConfigJson,
    path: P,
    update_temp_path: P,
    running_config: &mut RunningConfig,
) -> bool {
    let update_temp_path_old_p =
        Path::new(update_temp_path.as_ref()).join(".update_temp_path_old_version");
    if update_temp_path_old_p.exists() {
        if let Err(e) = fs::remove_dir_all(&update_temp_path_old_p) {
            Log::error("清除更新缓存文件夹读取：");
            Log::error(e.to_string().as_str());
            return false;
        }
    }
    if let Err(e) = fs::create_dir_all(&update_temp_path_old_p) {
        Log::error("创建更新缓存文件夹失败");
        Log::error(e.to_string().as_str());
        return false;
    }
    let current_exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            Log::error("获取当前可执行文件路径失败");
            Log::error(e.to_string().as_str());
            return false;
        }
    };
    running_config.file_path = std::collections::HashMap::new();
    let mut move_target = Vec::new();
    for (index, item) in config.added.iter().chain(config.changed.iter()).enumerate() {
        let file_path = path.as_ref().join(&item.filePath);
        let from_path = Path::new(update_temp_path.as_ref()).join(&item.hash);
        let to_path = update_temp_path_old_p.join(index.to_string());
        let parent_create_failed = {
            let parent = match file_path.parent() {
                Some(parent) => parent,
                None => {
                    Log::error("创建目标父文件夹失败");
                    Log::error("目标文件缺少父目录");
                    return false;
                }
            };
            if let Err(e) = fs::create_dir_all(parent) {
                Log::error("创建目标父文件夹失败");
                Log::error(parent.to_string_lossy().as_ref());
                Log::error(e.to_string().as_str());
                true
            } else {
                false
            }
        };
        let check = {
            if !from_path.is_file() {
                Log::error("缺少迁移的目标文件:");
                Log::error(from_path.to_string_lossy().as_ref());
                false
            } else if parent_create_failed {
                false
            } else if file_path.exists() {
                if current_exe_path == file_path {
                    set_need_update_myself(true);
                    true
                } else {
                    match fs::rename(&file_path, &to_path) {
                        Ok(_) => {
                            running_config
                                .file_path
                                .insert(index, file_path.to_string_lossy().to_string());
                            move_target.push((file_path, to_path));
                            true
                        }
                        Err(e) => {
                            Log::error("rename文件失败");
                            Log::error(file_path.to_string_lossy().as_ref());
                            Log::error(e.to_string().as_str());
                            false
                        }
                    }
                }
            } else {
                true
            }
        };
        if !check {
            for (from, to) in move_target.iter() {
                if let Err(e) = fs::rename(to, from) {
                    Log::error("回退文件失败");
                    Log::error(e.to_string().as_str());
                    return false;
                }
            }
            running_config.file_path.clear();
            return false;
        }
    }

    true
}

pub(crate) fn copy_file<P: AsRef<Path>>(
    config: &UpdateConfigJson,
    path: P,
    update_temp_path: P,
    running_config_file: &mut fs::File,
    running_config: &mut RunningConfig,
    ui: &impl UpdateUi,
) -> bool {
    let mut hand_file_num = 0.0;
    let total_file = (config.added.len() + config.changed.len()) as f64;
    Log::info("总共需要迁移得文件为");
    Log::info(total_file.to_string().as_str());
    let current_exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            Log::error("获取当前可执行文件路径失败");
            Log::error(e.to_string().as_str());
            return false;
        }
    };
    for item in config.added.iter().chain(config.changed.iter()) {
        hand_file_num += 1.0;
        Log::info(format!(" 当前迁移第{}个文件", hand_file_num as u32).as_str());
        let file_path = path.as_ref().join(&item.filePath);
        Log::info("迁移的目标文件:");
        Log::info(file_path.to_string_lossy().as_ref());

        let from_path = Path::new(update_temp_path.as_ref()).join(&item.hash);
        Log::info("迁移的源文件:");
        Log::info(from_path.to_string_lossy().as_ref());
        ui.on_progress(hand_file_num / total_file);

        thread::sleep(Duration::from_millis(10));
        if need_update_myself() && file_path == current_exe_path {
            std::thread::spawn(move || loop {
                let update_myself_now = update_myself_now();
                if update_myself_now {
                    let parent = match from_path.parent() {
                        Some(parent) => parent,
                        None => {
                            Log::error("无法获取旧文件父目录");
                            break;
                        }
                    };
                    let r = fs::rename(&file_path, parent.join("updater_old"));
                    Log::info(format!("delete {r:#?}").as_str());
                    let r = fs::rename(from_path, file_path);
                    Log::info(format!("rename {r:#?}").as_str());
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            });
            continue;
        }

        if fs::copy(from_path, &file_path).is_err() {
            Log::error("复制源文件到对应路径错误");
            Log::error(file_path.to_string_lossy().as_ref());
            running_config.status = RunningState::Failed;
            flush_config_file(running_config_file, running_config);
            return false;
        } else {
            running_config
                .moved_path
                .push(file_path.to_string_lossy().to_string());
            flush_config_file(running_config_file, running_config);
        }
    }
    true
}

pub(crate) fn flush_config_file(
    running_config_file: &mut fs::File,
    running_config: &RunningConfig,
) {
    if let Err(e) = running_config_file.set_len(0) {
        Log::error("清空配置文件失败");
        Log::error(e.to_string().as_str());
        return;
    }
    if let Err(e) = running_config_file.rewind() {
        Log::error("重置配置文件指针失败");
        Log::error(e.to_string().as_str());
        return;
    }
    let config_json = match serde_json::to_string(running_config) {
        Ok(json) => json,
        Err(e) => {
            Log::error("序列化运行配置失败");
            Log::error(e.to_string().as_str());
            return;
        }
    };
    if let Err(e) = running_config_file.write_all(config_json.as_bytes()) {
        Log::error("写入配置文件失败");
        Log::error(e.to_string().as_str());
        return;
    }
    if let Err(e) = running_config_file.sync_all() {
        Log::error("同步配置文件失败");
        Log::error(e.to_string().as_str());
    }
}
