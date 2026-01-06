use std::env;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::signal::unix::{signal, SignalKind};
use log::{info, error, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializar el sistema de logs (Requerimiento: registrar eventos) 
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 2. Configurar el puerto desde variable de entorno o usar default (Requerimiento: puerto configurable) 
    // Tu amigo usaba os.getenv('SERVER_PORT', 8080) [cite: 43]
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // 3. Iniciar el Listener TCP
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("Servidor TCP activo y listo en puerto {}", port);
            l
        },
        Err(e) => {
            error!("Fallo critico al arrancar el servidor: {}", e);
            return Err(e.into());
        }
    };

    // 4. Configurar manejo de señales (SIGINT y SIGTERM) 
    let mut sig_int = signal(SignalKind::interrupt())?;
    let mut sig_term = signal(SignalKind::terminate())?;

    // Loop principal usando tokio::select! para manejar conexiones y señales simultáneamente
    loop {
        tokio::select! {
            // A. Aceptar nueva conexión
            res = listener.accept() => {
                match res {
                    Ok((mut socket, addr)) => {
                        info!("Nueva conexion entrante desde: {}", addr);
                        
                        // Procesar la conexión en una tarea ligera (green thread)
                        tokio::spawn(async move {
                            let msg = "Hola desde el servidor TCP de PUCMM (Rust Version)!\n";
                            if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                error!("Fallo el envio de mensaje al cliente {}: {}", addr, e);
                            }
                            // El socket se cierra automáticamente al salir del scope (Drop trait)
                            info!("Cliente {} desconectado", addr);
                        });
                    }
                    Err(e) => error!("No se pudo aceptar la conexion entrante: {}", e),
                }
            }

            // B. Manejo de señales de apagado [cite: 42, 96]
            _ = sig_int.recv() => {
                warn!("Interrupcion detectada (SIGINT). Cerrando servidor...");
                break;
            }
            _ = sig_term.recv() => {
                warn!("Terminacion solicitada (SIGTERM). Apagando servidor...");
                break;
            }
        }
    }

    info!("Servidor TCP finalizado exitosamente. Hasta pronto!");
    Ok(())
}