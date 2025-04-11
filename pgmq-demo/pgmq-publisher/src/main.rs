use pgmq::PGMQueueExt;
use log::{info, error};
use env_logger::Env;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use chrono::{Local, DateTime};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct MyMessage {
    id: i32,
    content: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();
    info!("rust publisher exapmle of pgmq");

    let dburl = "postgres://postgres:postgres@localhost:5432/postgres";
    let pool = PgPool::connect(dburl).await.expect("Failed to connect to the database");
    let queue = PGMQueueExt::new_with_pool(pool).await;

    let qname = "myqueue";
    queue.create(&qname).await.expect("Failed to create queue");

    // 发布消息
    for i in 1..1000 {
        let now: DateTime<Local> = Local::now();
        let timestr = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let msg = MyMessage {
            id: i,
            content: timestr,
        };

        let send = queue.send(&qname, &msg).await;
        match send {
            Ok(_) => info!("Message sent {} successfully", i),
            Err(e) => error!("Failed to send message: {}", e),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    info!("1000 messages sent successfully");
}
