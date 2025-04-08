use sysinfo::{System, SystemExt, ProcessExt};

fn main() {
    // Crea una nueva instancia de System
    let mut sys = System::new_all();
    sys.refresh_all();

    // Obtén los procesos en el sistema
    let processes = sys.processes();

    // Itera sobre los procesos para mostrar los 5 procesos con más uso de CPU
    let mut processes_vec: Vec<_> = processes.iter().collect();

    // Ordena los procesos por uso de CPU (de mayor a menor)
    processes_vec.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());

    // Muestra los 5 primeros procesos
    for (pid, p) in processes_vec.iter().take(5) {
        // Obtén el valor de memoria en kilobytes
        let memory_kb = p.memory();

        // Imprime el valor en kilobytes para depuración
        println!("Memoria (KB): {}", memory_kb);

        // Convierte la memoria a megabytes
        let memory_mb = memory_kb as f64 / 1024.0 / 1024.0;
        
        //let memory_bytes = p.memory();
        //let memory_kb = memory_bytes / 1024;
        //let memory_mb = memory_kb as f64 / 1024.0;


        // Imprime los detalles del proceso
        println!(
            "{} (PID: {}) - {:.2}% CPU | {:.2} MB RAM",
            p.name(),
            p.pid(),
            p.cpu_usage(),
            memory_mb
        );
    }
}
