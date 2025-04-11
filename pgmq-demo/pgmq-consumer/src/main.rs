use pgmq::{Message, PGMQueueExt, PgmqError};
use log::{info, error};
use env_logger::Env;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct MyMessage {
    id: i32,
    content: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("rust consumer exapmle of pgmq");

    let dburl = "postgres://postgres:postgres@localhost:5432/postgres";
    let pool = PgPool::connect(dburl).await.expect("Failed to connect to the database");
    let queue = PGMQueueExt::new_with_pool(pool).await;

    let qname = "myqueue";
    queue.create(&qname).await.expect("Failed to create queue");

    // 消费消息
    loop {
        let rev: Result<Option<Vec<Message<MyMessage>>>, PgmqError> = queue.read_batch_with_poll(&qname,3,5,Some(Duration::from_secs(1)), Some(Duration::from_millis(100))).await;
        match rev {
            Ok(messages) => {
                if let Some(msgs) = messages {
                    for msg in msgs {
                        info!("Received message: {:?}", msg);
                        let i = msg.msg_id;
                        queue.delete(&qname, i).await.expect("Failed to delete message");
                        info!("Deleted message with ID: {}", i);
                    }
                } else {
                    info!("No messages received, sleeping for a while...");
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            },
            Err(err) => {
                error!("Error reading messages: {}", err);
            }
        }
    }
}
