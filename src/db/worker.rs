use crate::app::{
    command::{Command, DbCommand},
    event::{DbEvent, Event},
};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use tokio::sync::mpsc;

pub async fn run(mut cmd_rx: mpsc::Receiver<Command>, evt_tx: mpsc::Sender<Event>) {
    let mut pool: Option<MySqlPool> = None;

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            Command::Db(db) => match db {
                DbCommand::Connect {
                    name,
                    host,
                    port,
                    user,
                    password,
                    db,
                } => {
                    // Build a DSN: mysql://user:pass@host:port/db
                    // NOTE: For production, you’ll want proper URL encoding for password.
                    let url = format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, db);

                    match MySqlPoolOptions::new()
                        .max_connections(5)
                        .connect(&url)
                        .await
                    {
                        Ok(p) => {
                            pool = Some(p);
                            let display = format!("{} ({}/{})", name, host, db);
                            let _ = evt_tx.send(Event::Db(DbEvent::Connected { display })).await;
                        }
                        Err(e) => {
                            let _ = evt_tx
                                .send(Event::Db(DbEvent::Error {
                                    message: e.to_string(),
                                }))
                                .await;
                        }
                    }
                }

                DbCommand::Disconnect => {
                    pool = None; // dropping pool closes connections
                    let _ = evt_tx.send(Event::Db(DbEvent::Disconnected)).await;
                }

                DbCommand::LoadTables => {
                    let Some(p) = pool.as_ref() else {
                        let _ = evt_tx
                            .send(Event::Db(DbEvent::Error {
                                message: "Not connected".to_string(),
                            }))
                            .await;
                        continue;
                    };

                    // MySQL: fetch table names from current DB
                    let res: Result<Vec<String>, sqlx::Error> = sqlx::query_scalar(
                        r#"
                        SELECT table_name
                        FROM information_schema.tables
                        WHERE table_schema = DATABASE()
                        ORDER BY table_name
                        "#,
                    )
                    .fetch_all(p)
                    .await;

                    match res {
                        Ok(tables) => {
                            let _ = evt_tx
                                .send(Event::Db(DbEvent::TablesLoaded { tables }))
                                .await;
                        }
                        Err(e) => {
                            let _ = evt_tx
                                .send(Event::Db(DbEvent::Error {
                                    message: e.to_string(),
                                }))
                                .await;
                        }
                    }
                }
                DbCommand::LoadColumns { table } => {
                    let Some(p) = pool.as_ref() else {
                        let _ = evt_tx
                            .send(Event::Db(DbEvent::Error {
                                message: "Not connected".to_string(),
                            }))
                            .await;
                        continue;
                    };

                    let res = sqlx::query(
                        r#"
                        SELECT
                            column_name,
                            data_type,
                            is_nullable,
                            column_key
                        FROM information_schema.columns
                        WHERE table_schema = DATABASE()
                          AND table_name = ?
                        ORDER BY ordinal_position
                        "#,
                    )
                    .bind(&table)
                    .fetch_all(p)
                    .await;

                    match res {
                        Ok(rows) => {
                            use sqlx::Row;

                            let cols = rows
                                .into_iter()
                                .map(|r| crate::app::state::ColumnInfo {
                                    name: r.try_get::<String, _>("column_name").unwrap_or_default(),
                                    data_type: r
                                        .try_get::<String, _>("data_type")
                                        .unwrap_or_default(),
                                    is_nullable: r
                                        .try_get::<String, _>("is_nullable")
                                        .map(|v| v == "YES")
                                        .unwrap_or(false),
                                    column_key: r
                                        .try_get::<Option<String>, _>("column_key")
                                        .unwrap_or(None),
                                })
                                .collect::<Vec<_>>();

                            let _ = evt_tx
                                .send(Event::Db(DbEvent::ColumnsLoaded {
                                    table,
                                    columns: cols,
                                }))
                                .await;
                        }
                        Err(e) => {
                            let _ = evt_tx
                                .send(Event::Db(DbEvent::Error {
                                    message: e.to_string(),
                                }))
                                .await;
                        }
                    }
                }
            },
        }
    }
}
