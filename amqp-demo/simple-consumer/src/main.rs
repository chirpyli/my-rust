use log::{info, error};
use env_logger::Env;
use std::time::Duration;
use std::string::String;
use lapin::{Connection, ConnectionProperties, Consumer, 
    options::*, types::FieldTable,
    BasicProperties};
use futures_lite::stream::StreamExt;


#[tokio::main]
async fn main() -> Result<(), lapin::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("This is an demo of using rabbitmq amqp client in rust");

    // connect to rabbitmq and send a message
    let amqp_url = "amqp://guest:guest@localhost:5672/%2F";

    // Producer和Consumer客户端通过TCP连接到rabbitmq服务器
    let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;

    // 创建消息通道
    let channel_a = conn.create_channel().await?;

    // 声明一个队列
    let queue = channel_a
        .queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default())
        .await?;

    info!("Declared queue {:?}", queue);

    // 接收消息
    let mut consumer = channel_a.basic_consume(
        "hello",
        "my_consumer",
        lapin::options::BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let msg = String::from_utf8(delivery.data).unwrap();
                info!("Received message: {:?}", msg);
            }
            Err(e) => {
                error!("Error receiving message: {:?}", e);
            }
        }
    }
    
    channel_a.close(200, "Bye").await?;
    conn.close(200, "Bye").await?;

    Ok(())
}
