Google Protocol Buffer(简称 Protobuf)是一种轻便高效的结构化数据存储格式，平台无关、语言无关、可扩展，可用于通讯协议和数据存储等领域。下面给出在Rust中使用Protocol Buffers的基本流程步骤。


下面以Ubuntu-16.04LTS为例：
### 一、安装protoc

0.预先安装
```
sudo apt-get install autoconf automake libtool curl make g++ unzip
```

1.获取源码，生成configure
```
git clone https://github.com/google/protobuf.git
cd protobuf
git submodule update --init --recursive
./autogen.sh
```

2.编译安装
```
./configure  #By default, the package will be installed to /usr/local
make
make check
sudo make install
sudo ldconfig # refresh shared library cache.
```
>安装步骤可参考：https://github.com/google/protobuf/blob/master/src/README.md

### 二、安装protoc-gen-rust插件
使用cargo 安装：
```
cargo install protobuf --vers 1.7.4 #1.7.4为版本号，可选填。默认安装到~/.cargo/bin目录中
```

还可使用源码安装，从github上clone源码，编译安装，加入环境变量。安装步骤可参考：https://github.com/stepancheg/rust-protobuf/tree/master/protobuf-codegen

### 三、编写proto文件生成对应rust文件
proto文件语法规则可参考：[Language Guide (proto3)](https://developers.google.com/protocol-buffers/docs/proto3)

举例说明（在当前目录下生成foo.proto对应的rust文件）：
```
protoc --rust_out . foo.proto 
```
如果是其他语言，可在[Third-Party Add-ons for Protocol Buffers](https://github.com/google/protobuf/blob/master/docs/third_party.md)中找相关语言的插件等。
### 四、工程应用
1. 在rust工程中Cargo.toml中的添加protobuf
```toml
[dependencies]
protobuf = "1.7"		//注意版本问题，1.x与2.x，同时这里的版本 须与上面安装的protobuf版本相一致
```
2. 添加引用的crate:
```rust
extern crate protobuf;
```
3. 引用相关api......

>学习文档： [Developer Guide](https://developers.google.com/protocol-buffers/docs/overview)