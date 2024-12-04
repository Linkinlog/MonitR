use serde_json::json;
use sysinfo::System;

pub fn print() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let response = json!({
        "system": get_system_info(&sys),
        "disks": get_disks_info(),
        "networks": get_networks_info(),
        "components": get_components_info(),
    });

    println!("{}", response.to_string());
}

fn get_system_info(sys: &System) -> serde_json::Value {
    let load_avg = System::load_average();
    let uptime = format_uptime(System::uptime());

    json!({
        "boot_time": System::boot_time(),
        "uptime": uptime,
        "total_memory": sys.total_memory(),
        "used_memory": sys.used_memory(),
        "total_swap": sys.total_swap(),
        "used_swap": sys.used_swap(),
        "name": System::name().unwrap_or_else(|| "Unknown".to_string()),
        "kernel_version": System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        "os_version": System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        "host_name": System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        "cpus": sys.cpus().len(),
        "load_avg": {
            "one": load_avg.one,
            "five": load_avg.five,
            "fifteen": load_avg.fifteen,
        },
    })
}

fn get_disks_info() -> serde_json::Value {
    let disks = sysinfo::Disks::new_with_refreshed_list();

    disks
        .iter()
        .map(|disk| {
            json!({
                "name": disk.name().to_str().unwrap_or("Unknown disk"),
                "file_system": disk.file_system().to_str().unwrap_or("Unknown file system"),
                "mount_point": disk.mount_point().to_string_lossy(),
                "total_space": disk.total_space(),
                "available_space": disk.available_space(),
            })
        })
        .collect()
}

fn get_networks_info() -> serde_json::Value {
    let networks = sysinfo::Networks::new_with_refreshed_list();

    networks
        .iter()
        .map(|(name, data)| {
            json!({
                "interface_name": name,
                "total_received": data.total_received(),
                "total_transmitted": data.total_transmitted(),
            })
        })
        .collect()
}

fn get_components_info() -> serde_json::Value {
    let components = sysinfo::Components::new_with_refreshed_list();

    components
        .iter()
        .map(|component| {
            json!({
                "temperature": component.temperature(),
                "max": component.max(),
                "critical": component.critical().unwrap_or(0.0),
                "label": component.label(),
            })
        })
        .collect()
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let leftover_seconds = seconds % 60;

    format!("{days} days {hours} hours {minutes} minutes ({leftover_seconds} seconds in total)")
}
