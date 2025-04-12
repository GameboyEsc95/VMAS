use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt};
use std::{
    fs::{OpenOptions, create_dir_all},
    io::Write,
    thread,
    time::Duration,
};
use chrono::Local;
use plotters::prelude::*;
use windows::Win32::System::Console::GetConsoleWindow;

// Estructura para almacenar las muestras de CPU
struct CPUSample {
    timestamp: String,
    cpu_usage: f64,
}

fn is_console_attached() -> bool {  // Función para saber si está la consola 
    unsafe { GetConsoleWindow().0 != 0 }
}

pub fn generate_cpu_graph(samples: &[CPUSample]) {
    if !is_console_attached() {
        eprintln!("⚠️  Gráfico no generado hasta abrir una consola.");
        return;
    }

    let root_area = BitMapBackend::new("cpu_usage_graph.png", (640, 480)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Uso de CPU", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..samples.len() as u32, 0.0..100.0)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Intervalos (x5s)")
        .y_desc("CPU (%)")
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            samples
                .iter()
                .enumerate()
                .map(|(i, s)| (i as u32, s.cpu_usage)),
            &RED,
        ))
        .unwrap();

    println!("✅ Gráfico actualizado: cpu_usage_graph.png");
}

fn main() {
    let mut sys = System::new_all();
    let mut cpu_samples: Vec<CPUSample> = Vec::new();
    let mut iteration_count = 0;

    loop {
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_usage = sys.used_memory() as f32 * 100.0 / sys.total_memory() as f32;
        let disk_usage = sys
            .disks()
            .iter()
            .map(|d| d.total_space() - d.available_space())
            .sum::<u64>() as f32
            * 100.0
            / sys.disks().iter().map(|d| d.total_space()).sum::<u64>() as f32;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Top 5 procesos por uso de CPU
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let top5 = processes.iter().take(5).map(|p| {
            format!(
                "\n{} (PID: {}) - {:.2}% | {:.2} MB", 
                p.name(), p.pid(), p.cpu_usage(), p.memory() as f64 / 1024.0 / 1024.0
            )
        }).collect::<Vec<String>>().join(" | ");

        // Limpiar pantalla
        print!("\x1B[2J\x1B[1;1H");
        println!(
            "⏰ {}\nCPU: {:.2}% | Mem: {:.2}% | Disk: {:.2}%\nTop 5 procesos: {}\n",
            timestamp, cpu_usage, memory_usage, disk_usage, top5
        );

        // Cada 5 minutos (60 ciclos de 5s) guarda en CSV
        if iteration_count % 60 == 0 {
            let date_str = Local::now().format("%Y-%m-%d").to_string();
            let filename = format!("logs/metrics_{}.csv", date_str);
            create_dir_all("logs").unwrap();

            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&filename)
                .unwrap();

            if file.metadata().unwrap().len() == 0 {
                writeln!(file, "timestamp,cpu,memory,disk,top5_processes").unwrap();
            }

            writeln!(file, "{},{:.2},{:.2},{:.2},\"{}\"", timestamp, cpu_usage, memory_usage, disk_usage, top5).unwrap();
        }

        // Agregar muestra a la gráfica
        cpu_samples.push(CPUSample {
            timestamp: timestamp.clone(),
            cpu_usage: cpu_usage as f64,
        });

        if cpu_samples.len() > 10 {
            cpu_samples.remove(0);
        }

        if cpu_samples.len() == 10 {
            generate_cpu_graph(&cpu_samples);
        }

        iteration_count += 1;
        thread::sleep(Duration::from_secs(5));
    }
}
