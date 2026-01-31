use std::{env, path::Path};
use sysinfo::{Pid, System};

use crate::mlog::{Log, Logtrait};

/// 结束electron的进程
///
/// # Examples
///
/// ```
/// let path =  Path::new("/usr/bin/electron");
/// let result = end_electron_main(path);
///
/// ```
pub fn end_electron_main<P: AsRef<Path>>(path: P) -> bool {
    Log::info("尝试结束进程2");
    let mut sys = System::new_all();
    match env::var("exe_pid") {
        Ok(pid) if pid.parse::<usize>().is_ok() => {
            Log::info(format!("pid进程: {pid:#?}").as_str());

            // []
            match pid.parse::<usize>() {
                Ok(pid) => {
                    if let Some(process) = sys.process(Pid::from(pid)) {
                        process.kill();
                    }
                }
                Err(e) => {
                    Log::error("exe_pid 解析失败");
                    Log::error(e.to_string().as_str());
                }
            }
        }
        _ => (),
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.processes().iter().for_each(|(_pid, process)| {
        if let Some(exe) = process.exe() {
            if exe == path.as_ref() {
                Log::info(format!("再次尝试结束进程 {exe:?} {process:#?}").as_str());
                process.kill();
            }
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(50));
    sys.refresh_all();
    sys.processes().iter().any(|(_pid, process)| {
        if let Some(exe) = process.exe() {
            let r = exe == path.as_ref();
            if r {
                Log::error(format!("存在未退出的electron进程: {process:#?}").as_str());
                panic!("存在未退出的electron进程{:?}", exe);
            }
            r
        } else {
            false
        }
    })
}
