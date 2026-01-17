pub mod app;
pub mod db;
pub mod errors;
pub mod storage;
pub mod ui;

use crate::app::action::Action;
use crate::storage::{file_repo::FileConnectionRepo, repo::ConnectionRepo};
use crossterm::event::{self, Event as CEvent};
use directories::ProjectDirs;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use ulid::Ulid;

pub async fn run() -> Result<(), errors::AppError> {
    // --- terminal init ---
    let mut term = ui::terminal::TerminalGuard::init()?;

    // --- app state ---
    let mut state = app::state::AppState::new();

    // --- config path ---
    let proj = ProjectDirs::from("dev", "lensql", "lensql")
        .ok_or_else(|| errors::AppError::Config("Cannot determine config dir".into()))?;

    let config_dir = proj.config_dir();
    std::fs::create_dir_all(config_dir)?;

    // --- repo + load connections (do this BEFORE spawning worker that takes repo) ---
    let path = config_dir.join("connections.json");
    let repo = FileConnectionRepo::new(path);
    let stored = repo.load_connections()?;

    // --- keyring secrets (passwords) ---
    let secrets = storage::secrets::ConnectionSecrets::default();

    state.connections.items = stored
        .into_iter()
        .map(|c| {
            let id = c.id.parse::<Ulid>().map_err(|e| {
                errors::AppError::Config(format!("Invalid connection id ULID: {}", e))
            })?;

            let password = secrets
                .get_password(&c.id)?
                .unwrap_or_default();

            Ok(app::state::ConnectionItem {
                id,
                name: c.name,
                host: c.host,
                port: c.port,
                user: c.user,
                password,
                db: c.database,
            })
        })
        .collect::<Result<Vec<_>, errors::AppError>>()?;

    // --- channels ---
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<app::command::Command>(256);
    let (db_tx, db_rx) = mpsc::channel::<app::command::DbCommand>(256);
    let (st_tx, st_rx) = mpsc::channel::<app::command::StorageCommand>(256);
    let (evt_tx, mut evt_rx) = mpsc::channel::<app::event::Event>(256);

    // --- spawn workers ---
    tokio::spawn(db::worker::run(db_rx, evt_tx.clone()));
    tokio::spawn(storage::worker::run(st_rx, evt_tx.clone(), repo));

    // tick for UI refresh (spinners etc later)
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // --- render ---
        term.draw(|f| ui::render(f, &state))?;

        // Prefer processing any pending worker events first
        while let Ok(ev) = evt_rx.try_recv() {
            let cmds = app::reducer::reduce_event(&mut state, ev);
            for c in cmds {
                let _ = cmd_tx.try_send(c);
            }
        }

        // Route commands to the right worker
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                app::command::Command::Db(c) => {
                    let _ = db_tx.try_send(c);
                }
                app::command::Command::Storage(c) => {
                    let _ = st_tx.try_send(c);
                }
            }
        }

        // --- input timeout ---
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if let Some(action) = app::keymap::map_key(&state, key) {
                    if matches!(action, Action::Quit) {
                        break;
                    }

                    let cmds = app::reducer::reduce_action(&mut state, action);
                    for c in cmds {
                        let _ = cmd_tx.try_send(c);
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}
