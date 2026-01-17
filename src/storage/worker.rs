use tokio::sync::mpsc;

use crate::{
    app::{
        command::StorageCommand,
        event::{Event, StorageEvent},
    },
    storage::{file_repo::FileConnectionRepo, repo::ConnectionRepo},
};

pub async fn run(
    mut rx: mpsc::Receiver<StorageCommand>,
    tx: mpsc::Sender<Event>,
    repo: FileConnectionRepo,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            StorageCommand::SaveConnections { connections } => {
                let res = repo.save_connections(&connections);
                match res {
                    Ok(_) => {
                        let _ = tx
                            .send(Event::Storage(StorageEvent::ConnectionsSaved))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Event::Storage(StorageEvent::Error {
                                message: e.to_string(),
                            }))
                            .await;
                    }
                }
            }
        }
    }
}
