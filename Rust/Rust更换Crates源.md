Rust编译时遇到如下问题：
```rust
Downloading futures v0.1.19
warning: spurious network error (2 tries remaining): [28] Timeout was reached (Operation timed out after 30857 milliseconds with 0 out of 0 bytes received)
error: unable to get packages from source                                       

Caused by:
  [35] SSL connect error (OpenSSL SSL_connect: SSL_ERROR_SYSCALL in connection to static.crates.io:443 )

```

解决办法：更换Crates源

Rust开发时有时使用官方的源太慢，可以考虑更换使用国内中科大的源。更换方法如下：

在 `$HOME/.cargo/config` 中添加如下内容：
```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```
如果所处的环境中不允许使用 git 协议，可以把上述地址改为：
```toml
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```


>为什么这么配置可以参考[The Cargo Book/Source Replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html).


>参考文档：     
[The Cargo Book/Source Replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html)  
[The Cargo Book/Configuration](https://doc.rust-lang.org/cargo/reference/config.html#configuration)     
[Rust Crates 源使用帮助](http://mirrors.ustc.edu.cn/help/rust-crates.html)