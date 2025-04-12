import os
import pandas as pd
import matplotlib.pyplot as plt
from fpdf import FPDF

output_dir = "reports"
os.makedirs(output_dir, exist_ok=True)  # Crear directorio si no existe

def generar_estadisticas(nombre_csv):
    df = pd.read_csv(nombre_csv)

    resumen = {
        'CPU Promedio': df['cpu'].mean(),
        'CPU Máximo': df['cpu'].max(),
        'Memoria Promedio': df['memory'].mean(),
        'Memoria Máximo': df['memory'].max(),
        'Disco Promedio': df['disk'].mean(),
        'Disco Máximo': df['disk'].max(),
    }

    return resumen

def generar_grafica(df, columna_x, columna_y, output_file):
    # Verifica los nombres de las columnas
    print(df.columns)  # Imprime las columnas para verificar

    # Limpia los nombres de las columnas (en caso de espacios extra)
    df.columns = df.columns.str.strip()

    # Crear gráfica
    plt.figure(figsize=(10, 6))
    plt.plot(df[columna_x], df[columna_y], marker='o', linestyle='-')
    plt.title(f'Reporte: {columna_y} vs {columna_x}')
    plt.xlabel(columna_x)
    plt.ylabel(columna_y)
    plt.grid(True)

    # Guardar gráfica como imagen
    plt.savefig(output_file)
    plt.close()

def generar_pdf(nombre_csv):
    df = pd.read_csv(nombre_csv)
    resumen = generar_estadisticas(nombre_csv)

    # Generar gráfica y guardarla como imagen
    grafica_file = os.path.join(output_dir, os.path.basename(nombre_csv).replace(".csv", "_grafica.png"))
    generar_grafica(df, 'timestamp', 'cpu', grafica_file)  

    # Crear PDF
    pdf = FPDF()
    pdf.add_page()
    pdf.set_font("Arial", size=12)

    # Título del reporte
    pdf.cell(200, 10, txt=f"Reporte de métricas: {os.path.basename(nombre_csv)}", ln=True, align='C')
    pdf.ln(10)

    # Estadísticas generales
    pdf.cell(200, 10, txt="Estadísticas Generales:", ln=True)
    for key, value in resumen.items():
        pdf.cell(200, 8, txt=f"{key}: {value:.2f}", ln=True)

    # Insertar la gráfica
    pdf.ln(10)  # Espacio antes de la gráfica
    pdf.image(grafica_file, x=10, y=pdf.get_y(), w=180)

    # Guardar el PDF
    output_pdf = os.path.join(output_dir, os.path.basename(nombre_csv).replace(".csv", ".pdf"))
    pdf.output(output_pdf)

    print(f"✅ Reporte generado como: {output_pdf}")

if __name__ == "__main__":
    directorio_logs = "logs"  # Directorio donde se encuentran los archivos CSV
    # Obtener todos los archivos CSV del directorio
    archivos_csv = [f for f in os.listdir(directorio_logs) if f.endswith('.csv')]

    # Generar un reporte para cada archivo CSV
    for archivo in archivos_csv:
        archivo_csv_path = os.path.join(directorio_logs, archivo)
        generar_pdf(archivo_csv_path)
