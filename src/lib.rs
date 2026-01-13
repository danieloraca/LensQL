pub mod app;
pub mod db;
pub mod errors;
pub mod ui;

use app::{action::Action, event::Event};
use crossterm::event::{self, Event as CEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub async fn run() -> Result<(), errors::AppError> {
    // --- channels ---
    let (cmd_tx, cmd_rx) = mpsc::channel::<app::command::Command>(256);
    let (evt_tx, mut evt_rx) = mpsc::channel::<app::event::Event>(256);

    // --- spawn DB worker (stub for now) ---
    tokio::spawn(db::worker::run(cmd_rx, evt_tx));

    // --- terminal init ---
    let mut term = ui::terminal::TerminalGuard::init()?;

    // --- app state ---
    let mut state = app::state::AppState::new();

    // initial load (e.g. connections from storage later)
    // for now seed dummy connections:
    state.connections.items = vec![app::state::ConnectionItem::new(
        "localhost",
        "127.0.0.1",
        9966,
        "root",
        "secret",
        "9_testa",
    )];

    // tick for UI refresh (spinners etc later)
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // --- render ---
        term.draw(|f| ui::render(f, &state))?;

        // --- input (non-blocking-ish) ---
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        // Prefer processing any pending worker events first
        while let Ok(ev) = evt_rx.try_recv() {
            let cmds = app::reducer::reduce_event(&mut state, ev);
            for c in cmds {
                let _ = cmd_tx.try_send(c);
            }
        }

        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if let Some(action) = app::keymap::map_key(&state, key) {
                    // global quit
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
            // future: dispatch tick action/event if needed
        }
    }

    // terminal guard drops & restores automatically
    Ok(())
}
