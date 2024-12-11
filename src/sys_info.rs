use core::str;
use std::process::Command;

use rusqlite::{params, Connection};
use serde_json::json;
use sysinfo::System;

pub fn log_system_info(conn: &Connection) -> i64 {
    let mut sys = System::new_all();
    sys.refresh_all();

    let test_host = "1.1.1.1";

    let response = json!({
        "system": system_info(&sys),
        "disks": disks_info(),
        "networks": networks_info(),
        "components": components_info(),
        "ping": ping_info(test_host),
    });

    let id = insert_log_info(&conn).expect("Error inserting log info");
    insert_system_info(&conn, id, &response).expect("Error inserting system info");
    insert_disks_info(&conn, id, &response).expect("Error inserting disk info");
    insert_networks_info(&conn, id, &response).expect("Error inserting network info");
    insert_components_info(&conn, id, &response).expect("Error inserting component info");
    insert_ping_time(
        &conn,
        id,
        test_host,
        response["ping"]["time"].as_f64().unwrap_or(0.0),
    )
    .expect("Error inserting ping time");

    id
}

pub fn print_system_info(conn: &Connection) {
    let mut sys = System::new_all();
    sys.refresh_all();

    let test_host = "1.1.1.1";

    let response = json!({
        "system": system_info(&sys),
        "disks": disks_info(),
        "networks": networks_info(),
        "components": components_info(),
        "ping": ping_info(test_host),
    });

    println!("{}", response);

    match query_entries(conn) {
        Ok(entries) => || {
            for entry in entries.as_array().unwrap() {
                println!("{}", entry);
            }
        }(),
        Err(err) => eprintln!("Error querying entries: {}", err),
    };
}

pub fn create_tables(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ping_times (
            id INTEGER PRIMARY KEY,
            log_id INTEGER,
            host TEXT,
            time REAL
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS log_info (
            id INTEGER PRIMARY KEY,
            timestamp INTEGER
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_info (
            id INTEGER PRIMARY KEY,
            log_id INTEGER,
            boot_time INTEGER,
            uptime TEXT,
            total_memory INTEGER,
            used_memory INTEGER,
            total_swap INTEGER,
            used_swap INTEGER,
            name TEXT,
            kernel_version TEXT,
            os_version TEXT,
            host_name TEXT,
            cpus INTEGER,
            load_avg_one REAL,
            load_avg_five REAL,
            load_avg_fifteen REAL
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS disk_info (
            id INTEGER PRIMARY KEY,
            log_id INTEGER,
            name TEXT,
            file_system TEXT,
            mount_point TEXT,
            total_space INTEGER,
            available_space INTEGER
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS network_info (
            id INTEGER PRIMARY KEY,
            log_id INTEGER,
            interface_name TEXT,
            total_received INTEGER,
            total_transmitted INTEGER
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS component_info (
            id INTEGER PRIMARY KEY,
            log_id INTEGER,
            temperature REAL,
            max REAL,
            critical REAL,
            label TEXT
        )",
        [],
    )
    .unwrap();
}
pub fn ping_info(target: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("ping").arg("-c").arg("1").arg(target).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = str::from_utf8(&output.stdout).unwrap_or("");
                if let Some(ms) = parse_ping_time(stdout) {
                    return Ok(json!({ "time": ms, "host": target }));
                } else {
                    return Err("Failed to parse ping time".to_string());
                }
            } else {
                let stderr = str::from_utf8(&output.stderr).unwrap_or("");
                return Err(format!("Ping failed: {}", stderr));
            }
        }
        Err(e) => {
            return Err(format!("Failed to run ping: {}", e));
        }
    }
}

pub fn insert_ping_time(
    conn: &Connection,
    id: i64,
    host: &str,
    time: f64,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "INSERT INTO ping_times (log_id, host, time) VALUES (?1, ?2, ?3)",
        params![id, host, time],
    )
}

fn parse_ping_time(output: &str) -> Option<f64> {
    output.lines().find_map(|line| {
        if line.contains("time=") {
            let time_part = line.split("time=").nth(1)?;
            let ms_str = time_part.split_whitespace().next()?;
            return ms_str.parse::<f64>().ok();
        }
        None
    })
}

fn system_info(sys: &System) -> serde_json::Value {
    let load_avg = System::load_average();
    let uptime = System::uptime();

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

fn disks_info() -> serde_json::Value {
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

fn networks_info() -> serde_json::Value {
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

fn components_info() -> serde_json::Value {
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

fn insert_log_info(conn: &Connection) -> Result<i64, rusqlite::Error> {
    match conn.execute(
        "INSERT INTO log_info (timestamp) VALUES (?1)",
        params![chrono::Local::now().timestamp()],
    ) {
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(err) => Err(err),
    }
}

fn insert_system_info(
    conn: &Connection,
    id: i64,
    response: &serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let system_info = &response["system"];
    match conn.execute(
        "INSERT INTO system_info (log_id, boot_time, uptime, total_memory, used_memory, total_swap, used_swap, name, kernel_version, os_version, host_name, cpus, load_avg_one, load_avg_five, load_avg_fifteen) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            id,
            system_info["boot_time"].as_u64().unwrap_or(0),
            system_info["uptime"].as_str().unwrap_or(""),
            system_info["total_memory"].as_u64().unwrap_or(0),
            system_info["used_memory"].as_u64().unwrap_or(0),
            system_info["total_swap"].as_u64().unwrap_or(0),
            system_info["used_swap"].as_u64().unwrap_or(0),
            system_info["name"].as_str().unwrap_or(""),
            system_info["kernel_version"].as_str().unwrap_or(""),
            system_info["os_version"].as_str().unwrap_or(""),
            system_info["host_name"].as_str().unwrap_or(""),
            system_info["cpus"].as_u64().unwrap_or(0),
            system_info["load_avg"]["one"].as_f64().unwrap_or(0.0),
            system_info["load_avg"]["five"].as_f64().unwrap_or(0.0),
            system_info["load_avg"]["fifteen"].as_f64().unwrap_or(0.0),
        ],
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn insert_disks_info(
    conn: &Connection,
    id: i64,
    response: &serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let disks_info = &response["disks"];
    for disk in disks_info.as_array().unwrap() {
        match conn.execute(
            "INSERT INTO disk_info (log_id, name, file_system, mount_point, total_space, available_space) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                disk["name"].as_str().unwrap_or(""),
                disk["file_system"].as_str().unwrap_or(""),
                disk["mount_point"].as_str().unwrap_or(""),
                disk["total_space"].as_u64().unwrap_or(0),
                disk["available_space"].as_u64().unwrap_or(0),
            ],
        ) {
            Ok(_) => (),
            Err(err) => eprintln!("Error inserting disk info: {}", err),
        };
    }

    Ok(())
}

fn insert_networks_info(
    conn: &Connection,
    id: i64,
    response: &serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let networks_info = &response["networks"];
    for network in networks_info.as_array().unwrap() {
        match conn.execute(
            "INSERT INTO network_info (log_id, interface_name, total_received, total_transmitted) VALUES (?1, ?2, ?3, ?4)",
            params![
                id,
                network["interface_name"].as_str().unwrap_or(""),
                network["total_received"].as_u64().unwrap_or(0),
                network["total_transmitted"].as_u64().unwrap_or(0),
            ],
        ) {
            Ok(_) => (),
            Err(err) => eprintln!("Error inserting network info: {}", err),
        };
    }

    Ok(())
}

fn insert_components_info(
    conn: &Connection,
    id: i64,
    response: &serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let components_info = &response["components"];
    for component in components_info.as_array().unwrap() {
        match conn.execute(
            "INSERT INTO component_info (log_id, temperature, max, critical, label) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                component["temperature"].as_f64().unwrap_or(0.0),
                component["max"].as_f64().unwrap_or(0.0),
                component["critical"].as_f64().unwrap_or(0.0),
                component["label"].as_str().unwrap_or(""),
            ],
        ) {
            Ok(_) => (),
            Err(err) => eprintln!("Error inserting component info: {}", err),
        };
    }

    Ok(())
}

fn query_entries(conn: &Connection) -> Result<serde_json::Value, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM log_info")?;
    let systems = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            Ok(json!({
                "id": id,
                "system": query_system_for_entry(conn, id)?,
                "disks": query_disks_for_entry(conn, id)?,
                "networks": query_networks_for_entry(conn, id)?,
                "components": query_components_for_entry(conn, id)?,
                "ping": query_ping_for_entry(conn, id)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(systems.into())
}

fn query_system_for_entry(
    conn: &Connection,
    log_id: i64,
) -> Result<serde_json::Value, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM system_info WHERE log_id = ?")?;
    let system = stmt
        .query_map([log_id], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "log_id": row.get::<_, i64>(1)?,
                "boot_time": row.get::<_, i64>(2)?,
                "uptime": row.get::<_, String>(3)?,
                "total_memory": row.get::<_, i64>(4)?,
                "used_memory": row.get::<_, i64>(5)?,
                "total_swap": row.get::<_, i64>(6)?,
                "used_swap": row.get::<_, i64>(7)?,
                "name": row.get::<_, String>(8)?,
                "kernel_version": row.get::<_, String>(9)?,
                "os_version": row.get::<_, String>(10)?,
                "host_name": row.get::<_, String>(11)?,
                "cpus": row.get::<_, i64>(12)?,
                "load_avg": {
                    "one": row.get::<_, f64>(13)?,
                    "five": row.get::<_, f64>(14)?,
                    "fifteen": row.get::<_, f64>(15)?,
                }
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(system.into())
}

fn query_disks_for_entry(
    conn: &Connection,
    log_id: i64,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM disk_info WHERE log_id = ?")?;
    let disks = stmt
        .query_map([log_id], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "log_id": row.get::<_, i64>(1)?,
                "name": row.get::<_, String>(2)?,
                "file_system": row.get::<_, String>(3)?,
                "mount_point": row.get::<_, String>(4)?,
                "total_space": row.get::<_, i64>(5)?,
                "available_space": row.get::<_, i64>(6)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(disks)
}

fn query_networks_for_entry(
    conn: &Connection,
    log_id: i64,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM network_info WHERE log_id = ?")?;
    let networks = stmt
        .query_map([log_id], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "log_id": row.get::<_, i64>(1)?,
                "interface_name": row.get::<_, String>(2)?,
                "total_received": row.get::<_, i64>(3)?,
                "total_transmitted": row.get::<_, i64>(4)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(networks)
}

fn query_components_for_entry(
    conn: &Connection,
    log_id: i64,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM component_info WHERE log_id = ?")?;
    let components = stmt
        .query_map([log_id], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "log_id": row.get::<_, i64>(1)?,
                "temperature": row.get::<_, f64>(2)?,
                "max": row.get::<_, f64>(3)?,
                "critical": row.get::<_, Option<f64>>(4)?,
                "label": row.get::<_, String>(5)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(components)
}

fn query_ping_for_entry(
    conn: &Connection,
    log_id: i64,
) -> Result<Option<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM ping_times WHERE log_id = ?")?;
    let mut rows = stmt.query([log_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(json!({
            "id": row.get::<_, i64>(0)?,
            "log_id": row.get::<_, i64>(1)?,
            "host": row.get::<_, String>(2)?,
            "time": row.get::<_, f64>(3)?,
        })))
    } else {
        Ok(None)
    }
}
