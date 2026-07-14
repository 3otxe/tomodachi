

mod creature;
mod ipc;
mod renderer;
mod sprite;
mod state;
mod tray;
mod watcher;

use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};
use tomodachi_shared::{CreatureEvent, CreatureState};
use tracing::{error, info};

pub struct DaemonState {
    pub creature: creature::CreatureEngine,
    pub db: state::StateStore,
}

fn main() {
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("tomodachi daemon starting up");

    let db = match state::StateStore::open() {
        Ok(db) => db,
        Err(e) => {
            error!("failed to open state database: {}", e);
            eprintln!("Error: failed to open state database: {}", e);
            std::process::exit(1);
        }
    };

    let initial_state = db.load_creature().unwrap_or_default();
    info!(
        mood = %initial_state.mood,
        xp = initial_state.xp,
        hp = initial_state.hp,
        level = initial_state.level,
        "loaded creature state"
    );

    let engine = creature::CreatureEngine::new(initial_state.clone());

    let daemon_state = Arc::new(Mutex::new(DaemonState {
        creature: engine,
        db,
    }));

    let (event_tx, event_rx) = mpsc::unbounded_channel::<CreatureEvent>();

    let (state_tx, state_rx) = watch::channel(initial_state);

    let daemon_state_clone = Arc::clone(&daemon_state);
    let event_tx_clone = event_tx.clone();
    let state_tx_clone = state_tx.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async move {
            
            let ds = Arc::clone(&daemon_state_clone);
            let ipc_state_tx = state_tx_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = ipc::run_pipe_server(ds, ipc_state_tx).await {
                    error!("IPC server error: {}", e);
                }
            });

            let etx = event_tx_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = watcher::run_watcher(etx).await {
                    error!("filesystem watcher error: {}", e);
                }
            });

            let ds = Arc::clone(&daemon_state_clone);
            let stx = state_tx_clone;
            tokio::spawn(async move {
                process_events(event_rx, ds, stx).await;
            });

            let etx = event_tx_clone;
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    let _ = etx.send(CreatureEvent::MoodDecayTick);
                }
            });

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        });
    });

    info!("starting popup renderer on main thread");
    renderer::run_renderer(state_rx, event_tx);
}

async fn process_events(
    mut event_rx: mpsc::UnboundedReceiver<CreatureEvent>,
    daemon_state: Arc<Mutex<DaemonState>>,
    state_tx: watch::Sender<CreatureState>,
) {
    while let Some(event) = event_rx.recv().await {
        let new_state = {
            let mut ds = daemon_state.lock().unwrap();
            ds.creature.process_event(&event);
            let state = ds.creature.state().clone();

            if let Err(e) = ds.db.save_creature(&state) {
                error!("failed to save creature state: {}", e);
            }

            if let CreatureEvent::CommandFinished {
                exit_code,
                ref command,
                ref cwd,
            } = event
            {
                if let Err(e) = ds.db.log_command(
                    command.as_deref(),
                    Some(exit_code),
                    cwd.as_deref(),
                    None,
                ) {
                    error!("failed to log command: {}", e);
                }
            }

            state
        };

        let _ = state_tx.send(new_state);
    }
}
