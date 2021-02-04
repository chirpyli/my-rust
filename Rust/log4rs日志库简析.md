[log4rs](https://github.com/sfackler/log4rs)是rust实现的高度可配置日志库，该库配置的方式比较灵活，功能相对丰富，可以满足绝大部分的项目需要。


## 一、用法示例
首先先给个简单示例，通过这个示例就可以看出log4rs进行日志配置非常的灵活：
```rust
#[macro_use]
extern crate log;
extern crate log4rs;

fn main() {
    log4rs::init_file("log.yaml",Default::default()).unwrap();
    info!("Hello, world!");
}
```
日志配置文件如下：（文件格式不唯一，可以是yaml,json,toml等格式，下面是yaml格式的配置文件）
```yaml
# Scan this file for changes every 30 seconds
refresh_rate: 30 seconds

appenders:
  # An appender named "stdout" that writes to stdout
  stdout:
    kind: console

  # An appender named "requests" that writes to a file with a custom pattern encoder
  requests:
    kind: file
    path: "log/requests.log"
    encoder:
      pattern: "{d} - {m}{n}"

# Set the default logging level to "info" and attach the "stdout" appender to the root
root:
  level: info
  appenders:
    - stdout

loggers:
  # Raise the maximum log level for events sent to the "app::backend::db" logger to "info"
  app::backend::db:
    level: info

  # Route log events sent to the "app::requests" logger to the "requests" appender,
  # and *not* the normal appenders installed at the root
  app::requests:
    level: info
    appenders:
      - requests
    additive: false
```
运行结果：
```shell
sl@Li:~/Works/study/helloworld$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/helloworld`
2018-12-10T10:11:45.240346970+08:00 INFO helloworld - Hello, world!

```
这里因为appenders配置的是stdout对应的输出到控制台，所以日志信息输出在了控制台上，如果选择的是requests，则日志信息就输出在了对应路径文件上。

当然日志配置的方式除了配置文件，还可以在程序代码中进行配置，代码如下：
```rust
extern crate log;
extern crate log4rs;

use log::LogLevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

fn main() {
    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/requests.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LogLevelFilter::Info))
        .logger(Logger::builder()
            .appender("requests")
            .additive(false)
            .build("app::requests", LogLevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LogLevelFilter::Warn))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
    //配置完成，下面输出日志信息即可
    // use handle to change logger configuration at runtime
}
//可以看到用代码的方式比较麻烦，所以推荐使用配置文件的方式，程序简洁，修改方便，配置灵活，何乐不为啊。
```
log4rs有上面的两种配置形式，函数如下：

- init_config 	—— Initializes the global logger as a log4rs logger with the provided config.（通过代码配置）
- init_file —— Initializes the global logger as a log4rs logger configured via a file.（通过配置文件配置）

其实init_file函数实现的内部调用了init_config函数，也就是说是通过读配置文件通过序列化的方式得到config，再调用init_config。（源代码：https://docs.rs/log4rs/0.7.0/src/log4rs/priv_file.rs.html#23-51）

通过程序配置较好理解，通过配置文件的话，是有一定的格式及定义要求的，yaml,json,toml等要按一定的规则才能生成正确的配置文件，下面以yaml格式为例：
```yaml
# If set, log4rs will scan the file at the specified rate for changes and
# automatically reconfigure the logger. The input string is parsed by the
# humantime crate.
refresh_rate: 30 seconds#配置文件的刷新速率，可以理解为每隔30s重新读取一次配置文件

# The "appenders" map contains the set of appenders, indexed by their names.
appenders:

  foo:

    # All appenders must specify a "kind", which will be used to look up the
    # logic to construct the appender in the `Deserializers` passed to the
    # deserialization function.
    kind: console

    # Filters attached to an appender are specified inside the "filters"
    # array.
    filters:

      -
        # Like appenders, filters are identified by their "kind".
        kind: threshold

        # The remainder of the configuration is passed along to the
        # filter's builder, and will vary based on the kind of filter.
        level: error

    # The remainder of the configuration is passed along to the appender's
    # builder, and will vary based on the kind of appender.
    # Appenders will commonly be associated with an encoder.
    encoder:

      # Like appenders, encoders are identified by their "kind".
      #
      # Default: pattern
      kind: pattern

      # The remainder of the configuration is passed along to the
      # encoder's builder, and will vary based on the kind of encoder.
      pattern: "{d} [{t}] {m}{n}"

# The root logger is configured by the "root" map.
root:

  # The maximum log level for the root logger.
  #
  # Default: warn
  level: warn

  # The list of appenders attached to the root logger.
  #
  # Default: empty list
  appenders:
    - foo

# The "loggers" map contains the set of configured loggers, indexed by their
# names.
loggers:

  foo::bar::baz:

    # The maximum log level.
    #
    # Default: parent logger's level
    level: trace

    # The list of appenders attached to the logger.
    #
    # Default: empty list
    appenders:
      - foo

    # The additivity of the logger. If true, appenders attached to the logger's
    # parent will also be attached to this logger.
    #
    Default: true
    additive: false
```

通过上面的示例，基本用法就差不多了，下面继续深入学习一下log4rs。
## 二、log4rs
log4rs由4部分组成：appenders（输出到什么地方去）， encoders（按什么格式输出）， filters（那些可以输出，那些不能输出），loggers（日志实例）。
#### 【1】appenders
>An appender takes a log record and logs it somewhere, for example, to a file, the console, or the syslog.

appenders主要有以下三种：

 - console  —— The console appender.（输出到控制台）
 -  file 	——The file appender.（输出到文件）
 - rolling_file ——  A rolling file appender.（实现文件回滚）

分别实现了输出到控制台，输出到文件，实现文件回滚等功能。其中较为复杂的是rolling_file，这也是我们工程中常用到的，比如实现日志大小的控制（控制日志文件大小为10MB，到达10MB后，自动清空日志，重新开始记录），实现日志文件数量的控制(限制数量的同时限制单个文件大小)等等。下面给一个rolling_file的示例程序，说明一下它的用法。

```rust
#[macro_use]
extern crate log;
extern crate log4rs;

use std::default::Default;
use std::thread;
use std::time::Duration;

fn main() {
    log4rs::init_file("config/log4rs.yaml",Default::default()).unwrap();
    for i in 1..2{
        info!("booting up {}",i);
        error!("error test {}",i);
    }
	//无限循环，不断记录日志
    loop{
        thread::sleep(Duration::from_secs(1));
        info!("booting up ");
        error!("error test");
    }
}
```
重点是配置文件，通过不同的配置去实现不同功能：
```yaml
#实现了限制日志文件大小为1024Byte的功能。
appenders:
  stdout:
    kind: console
  requests:
    kind: file
    path: "requests.log"
    encoder:
      pattern: "{d} [{t}] {l} {M}:{m}{n}"
 ################################################## 
  roll:#定义rooling_file的appenders
    kind: rolling_file
    path: "roll.log"
    append: true
    encoder: 
      kind: pattern
    policy:
      kind: compound
      trigger: 
        kind: size
        limit: 1024 #限制大小为1024Byte
      roller:
        kind: delete#回滚方式为直接删除
  ##################################################
  
root:
  level: info
  appenders:
    - roll#使用roll appenders
loggers:
  app::backend::db:
    level: info
  app::requests:
    level: info
    appenders:
      - requests
    additive: false
```

另为附加限制日志文件大小的配置文件及限制日志文件数量的配置参考代码：
```yaml
#yaml文件
appenders:
  foo:#限制日志文件大小的配置
    kind: rolling_file
    path: {0}/foo.log
    policy:
      trigger:
        kind: size
        limit: 1024
      roller:
        kind: delete
  bar:#限制日志文件数量的配置
    kind: rolling_file
    path: {0}/foo.log
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 5 mb
      roller:
        kind: fixed_window
        pattern: '{0}/foo.log.{{}}'
        base: 1
        count: 5
```
对应的配置项的解说：
```yaml
/// # Configuration
///
/// ```yaml
/// kind: rolling_file
///
/// # The path of the log file. Required.
/// path: log/foo.log
///
/// # Specifies if the appender should append to or truncate the log file if it
/// # already exists. Defaults to `true`.
/// append: true
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
///
/// # The policy which handles rotation of the log file. Required.
/// policy:
///   # Identifies which policy is to be used. If no kind is specified, it will
///   # default to "compound".
///   kind: compound
///
///   # The remainder of the configuration is passed along to the policy's
///   # deserializer, and will vary based on the kind of policy.
///   trigger:
///     kind: size
///     limit: 10 mb
///
///   roller:
///     kind: delete
/// ```
```
为什么是这样配置呢，或者说，这些配置项是从哪里来的，就要分析源代码了，以rolling_file为例：
rolling_file中含有5个结构体，分别是：

- LogFile 	—— Information about the active log file.
- RollingFileAppender 	—— An appender which archives log files in a configurable strategy.
- RollingFileAppenderBuilder 	—— A builder for the RollingFileAppender.
- **RollingFileAppenderConfig 	—— Configuration for the rolling file appender.**
- RollingFileAppenderDeserializer 	—— A deserializer for the RollingFileAppender.
```rust
/// Configuration for the rolling file appender.
pub struct RollingFileAppenderConfig {
    path: String,
    append: Option<bool>,
    encoder: Option<EncoderConfig>,
    policy: Policy,
}
```
这个就列出了rolling_file appender的需要的配置项，其他的配置也是在***Config中，道理是一样的。

#### 【2】encoders
>An encoder is responsible for taking a log record, transforming it into the appropriate output format, and writing it out.

encoders主要由以下三部分组成：

- json ——	An encoder which writes a JSON object.
- pattern ——	A simple pattern-based encoder.
- writer —— 	Implementations of the encode::Write trait.

最常用的是pattern，所以着重分析一下pattern。这个有些类似与其他语言的格式化器或者日志属性等概念，链接：https://docs.rs/log4rs/0.7.0/log4rs/encode/pattern/index.html

```shell
d, date - The current time. By default, the ISO 8601 format is used. A custom format may be provided in the syntax accepted by chrono. The timezone defaults to local, but can be specified explicitly by passing a second argument of utc for UTC or local for local time.
{d} - 2016-03-20T14:22:20.644420340-08:00
{d(%Y-%m-%d %H:%M:%S)} - 2016-03-20 14:22:20
{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - 2016-03-20 22:22:20 UTC
f, file - The source file that the log message came from, or ??? if not provided.
h, highlight - Styles its argument according to the log level. The style is intense red for errors, red for warnings, blue for info, and the default style for all other levels.
{h(the level is {l})} - the level is ERROR
l``, level - The log level.
L, line - The line that the log message came from, or ??? if not provided.
m, message - The log message.
M, module - The module that the log message came from, or ??? if not provided.
n - A platform-specific newline.
t, target - The target of the log message.
T, thread - The name of the current thread.
I, thread_id - The ID of the current thread.
X, mdc - A value from the MDC. The first argument specifies the key, and the second argument specifies the default value if the key is not present in the MDC. The second argument is optional, and defaults to the empty string.
{X(user_id)} - 123e4567-e89b-12d3-a456-426655440000
{X(nonexistent_key)(no mapping)} - no mapping
An "unnamed" formatter simply formats its argument, applying the format specification.
{({l} {m})} - INFO hello
```



