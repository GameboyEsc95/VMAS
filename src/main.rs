use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt};
use std::{fs::{OpenOptions, create_dir_all}, io::Write, thread, time::Duration};
use chrono::Local;

fn main() {
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_usage = sys.used_memory() as f32 * 100.0 / sys.total_memory() as f32;
        let disk_usage = sys
            .disks()
            .iter()
            .map(|disk| disk.total_space() - disk.available_space())
            .sum::<u64>() as f32
            * 100.0
            / sys
                .disks()
                .iter()
                .map(|disk| disk.total_space())
                .sum::<u64>() as f32;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Top 5 procesos por uso de CPU
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let top5 = processes.iter().take(5).map(|p| {
            format!(
                "\n{} (PID: {}) - {:.2}% | {:.2} MB", 
                p.name(), p.pid(), p.cpu_usage(), p.memory() as f64 / 1024.0 / 1024.0 // Convertir memoria en bytes a mega-bytes
            )
        }).collect::<Vec<String>>().join(" | ");

        // Limpiar pantalla
        print!("\x1B[2J\x1B[1;1H");
        println!(
            "⏰ {}\nCPU: {:.2}% | Mem: {:.2}% | Disk: {:.2}%\nTop 5 procesos: {}\n",
            timestamp, cpu_usage, memory_usage, disk_usage, top5
        );

        // Guardar CSV histórico
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let filename = format!("logs/metrics_{}.csv", date_str);
        create_dir_all("logs").unwrap();

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filename)
            .unwrap();

        if file.metadata().unwrap().len() == 0 {
            writeln!(file, "timestamp,cpu,memory,disk,top5_processes").unwrap(); // encabezado
        }
            // Imprmir en pantalla el uso de recursos y procesos
        writeln!(file, "{},{:.2},{:.2},{:.2},\"{}\"", timestamp, cpu_usage, memory_usage, disk_usage, top5).unwrap();

        thread::sleep(Duration::from_secs(5)); // Cada 5 Segundos
    }
}
