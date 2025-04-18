use log::{info, error};
use env_logger::Env;
use std::time::Duration;
use lapin::{Connection, ConnectionProperties, 
    options::QueueDeclareOptions, options::BasicPublishOptions, types::FieldTable,
    BasicProperties, types::ReplyCode};


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
    let channel_b = conn.create_channel().await?;

    // 声明一个队列
    let queue = channel_a
        .queue_declare("hello", QueueDeclareOptions::default(), FieldTable::default())
        .await?;

    info!("Declared queue {:?}", queue);

    for i in 1..100 {
        let body = format!("hello world a {}", i);

        // 发送消息
        let _confirm = channel_a
            .basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                body.as_bytes(),
                BasicProperties::default(),
            )
            .await?;

        info!("channel_a Publish {}", body);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    for i in 1..100 {
        let body = format!("hello world b {}", i);

        // 发送消息
        let _confirm = channel_b
            .basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                body.as_bytes(),
                BasicProperties::default(),
            )
            .await?;

        info!("channel_b Publish {}", body);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    channel_a.close(200, "OK").await?;

    channel_b.close(200, "OK").await?;
    conn.close(200, "OK").await?;
    info!("Connection closed");

    Ok(())
}
