

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ServerOptions;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use tomodachi_shared::{ClientMessage, CreatureEvent, CreatureState, DaemonResponse, PIPE_NAME};

use crate::DaemonState;

pub async fn run_pipe_server(
    daemon_state: Arc<Mutex<DaemonState>>,
    state_tx: watch::Sender<CreatureState>,
) -> anyhow::Result<()> {
    info!(pipe = PIPE_NAME, "starting IPC server");

    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(PIPE_NAME)?;

    loop {
        
        server.connect().await?;
        info!("client connected");

        let connected = server;
        server = ServerOptions::new().create(PIPE_NAME)?;

        let ds = Arc::clone(&daemon_state);
        let stx = state_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(connected, ds, stx).await {
                debug!("client disconnected: {}", e);
            }
        });
    }
}

async fn handle_client(
    pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    daemon_state: Arc<Mutex<DaemonState>>,
    state_tx: watch::Sender<CreatureState>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = tokio::io::split(pipe);
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        debug!(msg = %line, "received message");

        let response = match serde_json::from_str::<ClientMessage>(&line) {
            Ok(msg) => process_message(msg, &daemon_state, &state_tx),
            Err(e) => {
                warn!(error = %e, "invalid message");
                DaemonResponse::Error {
                    message: format!("invalid message: {}", e),
                }
            }
        };

        let mut response_json = serde_json::to_string(&response)?;
        response_json.push('\n');
        writer.write_all(response_json.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}

fn process_message(
    msg: ClientMessage,
    daemon_state: &Arc<Mutex<DaemonState>>,
    state_tx: &watch::Sender<CreatureState>,
) -> DaemonResponse {
    match msg {
        ClientMessage::Notify {
            exit_code,
            cwd,
            pending_cmd,
            shell,
        } => {
            let mut ds = daemon_state.lock().unwrap();

            if let Some(ref cmd) = pending_cmd {
                let event = CreatureEvent::CommandPending {
                    command: cmd.clone(),
                };
                ds.creature.process_event(&event);
            }

            if let Some(code) = exit_code {
                let event = CreatureEvent::CommandFinished {
                    exit_code: code,
                    command: pending_cmd.clone(),
                    cwd: cwd.clone(),
                };
                ds.creature.process_event(&event);

                if let Err(e) = ds.db.log_command(
                    pending_cmd.as_deref(),
                    Some(code),
                    cwd.as_deref(),
                    shell.as_deref(),
                ) {
                    error!("failed to log command: {}", e);
                }
            }

            let new_state = ds.creature.state().clone();
            if let Err(e) = ds.db.save_creature(&new_state) {
                error!("failed to save state: {}", e);
            }

            let _ = state_tx.send(new_state);

            DaemonResponse::Ok
        }

        ClientMessage::Veto { command, args } => {
            let ds = daemon_state.lock().unwrap();
            let (allowed, reason) = ds.creature.evaluate_veto(&command, &args);
            DaemonResponse::VetoResult { allowed, reason }
        }

        ClientMessage::Status => {
            let ds = daemon_state.lock().unwrap();
            DaemonResponse::State {
                creature: ds.creature.state().clone(),
            }
        }

        ClientMessage::Ping => {
            let ds = daemon_state.lock().unwrap();
            DaemonResponse::State {
                creature: ds.creature.state().clone(),
            }
        }

        ClientMessage::Roast => {
            let ds = daemon_state.lock().unwrap();
            let text = match ds.db.generate_roast() {
                Ok(t) => t,
                Err(e) => format!("Failed to generate roast: {}", e),
            };
            DaemonResponse::RoastText { text }
        }
    }
}
