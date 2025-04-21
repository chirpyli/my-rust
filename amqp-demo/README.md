## Rust使用RabbitMQ示例
使用RabbitMQ需要用实现了AMQP协议的客户端，[lapin](https://docs.rs/lapin/latest/lapin/index.html)是Rust中一个实现了AMQP协议的客户端，这里使用lapin来连接RabbitMQ。

### 简单场景
对简单的场景，比如不需要路由，不需要交换机（采用默认exchange），只需要一个队列，可以参考如下示例：
- [simple-publisher](./simple-publisher/)    发布者
- [simple-consumer](./simple-consumer/)     消费者

### 路由场景

#### fanout模式
该类型路由规则非常简单，会把所有发送到该exchange的消息路由到所有与它绑定的Queue中，相当于广播功能。

- [fanout-publisher](./fanout-publisher/)    发布者， 可以发布消息到exchange，所有绑定到该exchange的队列都会收到消息
- [fanout-consumer](./fanout-consumer/)     消费者，绑定到exchange的队列，会收到消息

如果是多个消费者订阅同一个队列时，则队列会平均分配消息，即消息会以轮询的方式发送给各消费者，实现天然的负载均衡。在 AMQP 0-9-1 中，消息的负载均衡是发生在消费者（consumer）之间的，而不是队列（queue）之间。


#### direct模式

direct模式是RabbitMQ默认的exchange类型，该类型路由规则非常简单，消息会被路由到binding key与routing key完全匹配的Queue中。

- [direct-publisher](./direct-publisher/)    发布者， 可以发布消息到exchange，只有binding key与routing key完全匹配的队列会收到消息
- [direct-consumer](./direct-consumer/)     消费者，只有binding key与routing key完全匹配的队列会收到消息




参考文档：
[一篇文章讲透彻了AMQP协议](https://developer.aliyun.com/article/847370)
[AMQP协议](https://www.rabbitmq.com/resources/specs/amqp0-9-1.pdf)]
