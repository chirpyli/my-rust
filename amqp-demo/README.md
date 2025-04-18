## Rust使用RabbitMQ示例
使用RabbitMQ需要用实现了AMQP协议的客户端，[lapin](https://docs.rs/lapin/latest/lapin/index.html)是Rust中一个实现了AMQP协议的客户端，这里使用lapin来连接RabbitMQ。

### 简单场景
对简单的场景，比如不需要路由，不需要交换机，只需要一个队列，可以参考如下示例：
- [simple-publisher](./simple-publisher/)    发布者
- [simple-consumer](./simple-consumer/)     消费者