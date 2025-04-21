use log::*;
use env_logger::Env;
use lapin::{Connection, ConnectionProperties, 
    options::*, types::FieldTable,
    BasicProperties, ExchangeKind};
use futures_lite::stream::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<(), lapin::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("This is an demo of using rabbitmq amqp client in rust");    

        // 获取命令行参数, 第一个参数是队列ID，第二个参数是消费者ID
    let mut args = Vec::new();
    for arg in env::args().skip(1) {
        args.push(arg);
    }

    let mut route_key = String::from("");
    if args[0] == 1.to_string() {
        route_key = String::from("beijing");
    } else if args[0] == 2.to_string() {
        route_key = String::from("hangzhou");
    } else {
        panic!("Please input a valid queue ID");
    }

    // connect to rabbitmq and send a message
    let amqp_url = "amqp://guest:guest@localhost:5672/%2F";

    // Producer和Consumer客户端通过TCP连接到rabbitmq服务器
    let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;

    // 创建消息通道
    let channel = conn.create_channel().await?;

    let mut queueoptions = QueueDeclareOptions::default();
    queueoptions.durable = true;
    queueoptions.auto_delete = true;

    let queue_name = String::from("worker_") + args[0].as_str();

    info!("Queue name: {}", queue_name);

    // 声明一个队列
    channel.queue_declare(&queue_name, queueoptions, FieldTable::default()).await?;

    // 创建一个direct类型的交换机
    channel.exchange_declare("exchange-direct", ExchangeKind::Direct, ExchangeDeclareOptions::default(), FieldTable::default()).await?;

    // 绑定队列到交换机，并指定路由键
    channel.queue_bind(&queue_name, "exchange-direct", &route_key, QueueBindOptions::default(), FieldTable::default()).await?;
    
    let properties = BasicProperties::default();

    let consumer_tag = String::from("consumer_") + args[1].as_str(); // 消费者标签

    // 消费消息
    let mut consumer = channel.basic_consume(&queue_name, &consumer_tag, BasicConsumeOptions::default(), FieldTable::default()).await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;

        let body = delivery.data;
        let delivery_tag = delivery.delivery_tag;

        info!("consumer {:?} received message: {:?} from queue {:?}", args[1], String::from_utf8(body).unwrap(), &queue_name);

        channel.basic_ack(delivery_tag, BasicAckOptions::default()).await?;
    }

    channel.close(200, "Bye").await?;

    Ok(())
}