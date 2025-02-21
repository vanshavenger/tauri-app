#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::Write;
use tauri::Manager;
use device_query::{DeviceQuery, DeviceState};
use sysinfo::{System, SystemExt, CpuExt};

struct RecordingState(Mutex<bool>);
struct EventLog(Mutex<Vec<(u64, String)>>);

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(RecordingState(Mutex::new(false)));
            app.manage(EventLog(Mutex::new(Vec::new())));
            
            let app_handle = app.handle().clone();
            
            std::thread::spawn(move || {
                let device_state = DeviceState::new();
                let mut last_mouse = device_state.get_mouse().coords;
                let mut last_keys = device_state.get_keys();

                loop {
                    let is_recording = *app_handle.state::<RecordingState>().0.lock().unwrap();
                    if is_recording {
                        let mouse = device_state.get_mouse();
                        let keys = device_state.get_keys();

                        if mouse.coords != last_mouse {
                            log_event(&app_handle, format!("Mouse moved from {:?} to {:?}", last_mouse, mouse.coords));
                            last_mouse = mouse.coords;
                        }

                        if !mouse.button_pressed.is_empty() {
                            log_event(&app_handle, format!("Mouse clicked: {:?}", mouse.button_pressed));
                        }

                        if keys != last_keys {
                            for key in keys.iter() {
                                if !last_keys.contains(key) {
                                    log_event(&app_handle, format!("Key pressed: {:?}", key));
                                }
                            }
                            last_keys = keys;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording, get_os_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn log_event(app_handle: &tauri::AppHandle, event: String) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    app_handle.state::<EventLog>().0.lock().unwrap().push((timestamp, event));
}

#[tauri::command]
fn start_recording(state: tauri::State<RecordingState>) {
    *state.0.lock().unwrap() = true;
}

#[tauri::command]
fn stop_recording(state: tauri::State<RecordingState>, event_log: tauri::State<EventLog>) -> Result<(), String> {
    *state.0.lock().unwrap() = false;

    let mut events = event_log.0.lock().unwrap();
    let mut file = File::create("event_log.csv").map_err(|e| e.to_string())?;
    writeln!(file, "Timestamp,Event").map_err(|e| e.to_string())?;

    for (timestamp, event) in events.iter() {
        writeln!(file, "{},{}", timestamp, event).map_err(|e| e.to_string())?;
    }

    events.clear();
    Ok(())
}

#[tauri::command]
fn get_os_info() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = sys.name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = sys.os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = sys.kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let cpu_brand = sys.cpus().get(0).map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "Unknown".to_string());
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    format!(
        "OS: {} {}\nKernel: {}\nCPU: {}\nCPU Usage: {:.2}%\nMemory: {}/{} MB\nSwap: {}/{} MB",
        os_name, os_version, kernel_version, cpu_brand, cpu_usage,
        used_memory / 1024 / 1024, total_memory / 1024 / 1024,
        used_swap / 1024 / 1024, total_swap / 1024 / 1024
    )
}