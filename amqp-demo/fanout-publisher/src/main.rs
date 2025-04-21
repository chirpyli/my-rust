use log::*;
use env_logger::Env;
use lapin::{Connection, ConnectionProperties, 
    options::*, types::FieldTable,
    BasicProperties, ExchangeKind};

#[tokio::main]
async fn main() -> Result<(), lapin::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("This is an demo of using rabbitmq amqp client in rust");    
    
    // connect to rabbitmq and send a message
    let amqp_url = "amqp://guest:guest@localhost:5672/%2F";

    // Producer和Consumer客户端通过TCP连接到rabbitmq服务器
    let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;

    // 创建消息通道
    let channel = conn.create_channel().await?;

    let mut queueoptions = QueueDeclareOptions::default();
    queueoptions.durable = true;
    queueoptions.auto_delete = true;

    // 声明一个队列
    channel.queue_declare("worker_1", queueoptions, FieldTable::default()).await?;
    channel.queue_declare("worker_2", queueoptions, FieldTable::default()).await?;

    // 创建一个fanout类型的交换机
    channel.exchange_declare("myexchange", ExchangeKind::Fanout, ExchangeDeclareOptions::default(), FieldTable::default()).await?;
    
    // 绑定队列到交换机
    channel.queue_bind("worker_1", "myexchange", "", QueueBindOptions::default(), FieldTable::default()).await?;
    channel.queue_bind("worker_2", "myexchange", "", QueueBindOptions::default(), FieldTable::default()).await?;
    
    let payload = b"Hello, world!"; // 发送消息

    let mut publish_options = BasicPublishOptions::default();
    publish_options.mandatory = false;

    let properties = BasicProperties::default();

    let _confirm = channel.basic_publish("myexchange", "", publish_options, payload, properties).await?;

    info!("publish message: {:?}", String::from_utf8_lossy(payload));

    channel.close(200, "Bye").await?;

    Ok(())
}