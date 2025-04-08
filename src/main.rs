use chrono::{DateTime, Utc, Duration};
use csv::{ReaderBuilder, Writer};
use plotters::prelude::*;
use rusqlite::{params, Connection};
use serde::{Serialize, Deserialize};
use std::{error::Error, fs::{File, OpenOptions}};
use std::path::Path;

use windows_service::service_control_handler::{register_service_ctrl_handler, ServiceControlHandlerResult};
use windows_service::service::{ServiceControl, ServiceStatus, ServiceType};


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Data {
    timestamp: DateTime<Utc>,
    metric: String,
    value: f64,
}

impl Data {
    fn to_vec(&self) -> Vec<String> {
        vec![
            self.timestamp.to_rfc3339(),
            self.metric.clone(),
            self.value.to_string(),
        ]
    }
}

// Función para cargar y filtrar datos históricos
fn load_and_filter_data(file_path: &str) -> Result<Vec<Data>, Box<dyn Error>> {
    let mut data = Vec::new();
    let current_time = Utc::now();
    let two_days_ago = current_time - Duration::days(2);

    // Abrir el archivo CSV
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    // Leer los datos
    for result in rdr.records() {
        let record = result?;
        let timestamp = DateTime::parse_from_rfc3339(&record[0])?.with_timezone(&Utc);
        if timestamp >= two_days_ago {
            data.push(Data {
                timestamp,
                metric: record[1].to_string(),
                value: record[2].parse()?,
            });
        }
    }
    Ok(data)
}

// Función para guardar nuevos datos en CSV
fn save_data_to_csv(file_path: &str, new_data: &Data) -> Result<(), Box<dyn Error>> {
    let mut data = load_and_filter_data(file_path)?;
    data.push(new_data.clone());

    // Guardar los datos en el archivo CSV
    let file = OpenOptions::new().create(true).write(true).truncate(true).open(file_path)?;
    let mut wtr = Writer::from_writer(file);

    // Escribir los registros
    for entry in data {
        wtr.write_record(entry.to_vec())?;
    }

    Ok(())
}

// Función para guardar los datos en SQLite
fn save_data_to_sqlite(conn: &Connection, new_data: &Data) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metrics (timestamp TEXT, metric TEXT, value REAL)",
        [],
    )?;
    conn.execute(
        "INSERT INTO metrics (timestamp, metric, value) VALUES (?1, ?2, ?3)",
        params![new_data.timestamp.to_rfc3339(), new_data.metric, new_data.value],
    )?;
    Ok(())
}

// Función para generar un gráfico de uso
fn generate_graph(data: &[Data]) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("usage_graph.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Usage Over Time", ("sans-serif", 30))
        .build_cartesian_2d(
            (0..data.len()).into_segmented(),
            (0.0..=100.0),
        )?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().enumerate().map(|(i, d)| (i, d.value)),
        &RED,
    ))?;

    Ok(())
}

// Función para ejecutar el servicio en Windows o Linux
#[cfg(windows)]
fn create_windows_service() -> Result<(), Box<dyn Error>> {
    use windows_service::service::{ServiceAccess, ServiceControl, ServiceState, ServiceStatus, ServiceType, ServiceControlHandlerResult, ServiceStart};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    
    // Aquí agregas la implementación para registrar el servicio
    Ok(())
}

#[cfg(not(windows))]
fn create_linux_service() -> Result<(), Box<dyn Error>> {
    use std::process::Command;

    // Crear archivo de servicio systemd
    let service_content = r#"
    [Unit]
    Description=Rust Metrics Collection Service

    [Service]
    ExecStart=/path/to/your/rust/binary
    Restart=always
    User=your_user
    Group=your_group

    [Install]
    WantedBy=multi-user.target
    "#;

    let path = "/etc/systemd/system/rust-metrics.service";
    std::fs::write(path, service_content)?;

    // Habilitar el servicio en systemd
    Command::new("systemctl").arg("daemon-reload").output()?;
    Command::new("systemctl").arg("enable").arg("rust-metrics").output()?;
    Command::new("systemctl").arg("start").arg("rust-metrics").output()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "historical_data.csv";
    let conn = Connection::open("metrics.db")?;

    // Crear un nuevo dato (por ejemplo, uso de CPU)
    let new_data = Data {
        timestamp: Utc::now(),
        metric: "CPU_Usage".to_string(),
        value: 75.0,
    };

    // Guardar los nuevos datos
    save_data_to_csv(file_path, &new_data)?;
    save_data_to_sqlite(&conn, &new_data)?;

    // Generar gráfica
    let data = load_and_filter_data(file_path)?;
    generate_graph(&data)?;

    println!("Datos guardados y gráfico generado.");

    // Ejecutar servicio al inicio
    #[cfg(windows)]
    create_windows_service()?;
    
    #[cfg(not(windows))]
    create_linux_service()?;

    Ok(())
}
