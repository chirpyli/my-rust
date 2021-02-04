主要介绍rust-crypto和tiny-keccak这两个Rust实现的密码学库。

### [rust-crypto](https://github.com/DaGenix/rust-crypto/)
Rust实现的密码学库，包含了密码学中常用的对称密码、公钥密码、单向散列函数、消息认证码、数字签名、随机数生成器等算法。目前支持以下算法：

|Name|Description|
|- |- |
|AES | 高级加密标准(Advanced Encryption Standard)为最常见的对称加密算法|
| Bcrypt|专门为密码存储而设计的算法，基于Blowfish加密算法变形而来，是一个跨平台的文件加密工具
|BLAKE2b|BLAKE的64位版本，它可以生成最高512位的任意长度哈希
 |BLAKE2s|BLAKE的32位版本，它可以生成最高256位的任意长度哈希
|Blowfish|Blowfish算法是一个64位分组及可变密钥长度的对称密钥分组密码算法
| ChaCha20|ChaCha系列流密码，作为salsa密码的改良版，具有更强的抵抗密码分析攻击的特性，“20”表示该算法有20轮的加密计算
| Curve25519|Curve25519 是目前最高水平的 Diffie-Hellman函数，适用于广泛的场景，由Daniel J. Bernstein教授设计
| ECB, CBC, and CTR block cipher modes|ECB模式、CBC模式、CTR模式（分组密码模式）
 |Ed25519|Ed25519是一个数字签名算法，签名和验证的性能都极高
 |Fortuna|  一种密码学安全的随机数发生器，适用于长生命周期的任务
 |Ghash|An implementaiton of GHASH as used in GCM
| HC128|HC-128算法是HC-256算法的简化版,为欧洲e STREAM工程最终胜出的7个序列密码算法之一。HC-128由初始化算法和密钥流产生算法两部分构成,为基于表驱动的适于软件实现的算法。
|HMAC|散列消息身份验证码(Hashed Message Authentication Code)
 |MD5 |Message-Digest Algorithm 5（信息-摘要算法5），为计算机安全领域广泛使用的一种散列函数，用以提供消息的完整性保护
| PBKDF2|Password-Based Key Derivation Function 2
|PKCS padding for CBC block cipher mode|分组密码CBC模式PKCS填充
 |Poly1305|A cryptographic message authentication code (MAC) created by Daniel J. Bernstein
| RC4|一种对称加密算法，RSA三人组中的头号人物Ronald Rivest在1987年设计的密钥长度可变的流加密算法簇
 |RIPEMD-160|RACE Integrity Primitives Evaluation Message Digest，RACE原始完整性校验消息摘要(比特币中有应用)
 |Salsa20 and XSalsa20|Salsa20是由Daniel J.Bernstein提出的基于hash函数设计的流密码算法
 |Scrypt|一种内存依赖型hash算法(区块链中有应用)
 |Sha1|安全哈希算法（Secure Hash Algorithm 1）
 |Sha2 (All fixed output size variants)|SHA-2 (Secure Hash Algorithm 2)
 |Sha3|SHA-3 (Secure Hash Algorithm 3)
 |Sosemanuk|一种基于软件的流密码算法
 |Whirlpool|一种基于分组密码的散列算法

#### 代码示例1
```rust
//! SHA3-256 示例
extern crate crypto;
extern crate rustc_hex;

use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use rustc_hex::{ToHex,FromHex};

fn main() {
    // create a SHA3-256 object
    let mut hasher = Sha3::sha3_256();

    // write input message
    hasher.input_str("hello world");

    // read hash digest
    let hex = hasher.result_str();
    let res=hex.from_hex().unwrap();
    let res=res.as_slice();

    let expected: &[u8] = &[
        0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04,
        0x09, 0x99, 0xaa, 0xc8, 0x9e, 0x76, 0x22, 0xf3,
        0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94,
        0xa3, 0x1c, 0x3b, 0xfb, 0xf2, 0x4e, 0x39, 0x38
    ];

    assert_eq!(res,expected);
}

```
#### 代码示例2
```rust
//! AES256 CBC、CTR mode encrypt decrypt demo
use std::str;
use crypto::{symmetriccipher,buffer,aes,blockmodes};
use crypto::buffer::{ReadBuffer,WriteBuffer,BufferResult};
use crypto::aessafe::*;
use crypto::blockmodes::*;
use crypto::symmetriccipher::*;
use rand::{Rng,OsRng};

pub fn aes_cbc_mode(){
    let message="Hello World!";

    let mut key:[u8;32]=[0;32];
    let mut iv:[u8;16]=[0;16];

    // In a real program, the key and iv may be determined
    // using some other mechanism. If a password is to be used
    // as a key, an algorithm like PBKDF2, Bcrypt, or Scrypt (all
    // supported by Rust-Crypto!) would be a good choice to derive
    // a password. For the purposes of this example, the key and
    // iv are just random values.
    let mut rng=OsRng::new().ok().unwrap();
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut iv);

    let encrypted_data=aes256_cbc_encrypt(message.as_bytes(),&key,&iv).ok().unwrap();
    let decrypted_data=aes256_cbc_decrypt(&encrypted_data[..],&key,&iv).ok().unwrap();

    let crypt_message=str::from_utf8(decrypted_data.as_slice()).unwrap();

    assert_eq!(message,crypt_message);
    println!("{}",crypt_message);
}

// Encrypt a buffer with the given key and iv using AES-256/CBC/Pkcs encryption.
fn aes256_cbc_encrypt(data: &[u8],key: &[u8], iv: &[u8])->Result<Vec<u8>,symmetriccipher::SymmetricCipherError>{
    let mut encryptor=aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut final_result=Vec::<u8>::new();
    let mut read_buffer=buffer::RefReadBuffer::new(data);
    let mut buffer=[0;4096];
    let mut write_buffer=buffer::RefWriteBuffer::new(&mut buffer);

    loop{
        let result=try!(encryptor.encrypt(&mut read_buffer,&mut write_buffer,true));

        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow=>break,
            BufferResult::BufferOverflow=>{},
        }
    }

    Ok(final_result)
}

// Decrypts a buffer with the given key and iv using AES-256/CBC/Pkcs encryption.
fn aes256_cbc_decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}

pub fn aes_ctr_mode(){
    let message="Hello World! AES CTR MODE.";

    let mut key:[u8;32]=[0;32];
    let mut iv:[u8;16]=[0;16];

    let mut rng=OsRng::new().ok().unwrap();
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut iv);

    let encrypted_data=aes256_ctr_encrypt(message.as_bytes(),&key,&iv).ok().unwrap();
    let decrypted_data=aes256_ctr_decrypt(&encrypted_data[..],&key,&iv).ok().unwrap();

    let crypt_message=str::from_utf8(decrypted_data.as_slice()).unwrap();

    assert_eq!(message,crypt_message);
    println!("{}",crypt_message);
}

fn aes256_ctr_encrypt(data: &[u8],key: &[u8],iv: &[u8])->Result<Vec<u8>,symmetriccipher::SymmetricCipherError>{
    let mut final_result=Vec::<u8>::new();
    let mut read_buffer=buffer::RefReadBuffer::new(data);
    let mut buffer=[0;4096];
    let mut write_buffer=buffer::RefWriteBuffer::new(&mut buffer);

    let mut encoder=CtrMode::new(AesSafe256Encryptor::new(key),iv.to_vec());
    encoder.encrypt(&mut read_buffer,&mut write_buffer,true)?;

    final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
    Ok(final_result)
}

fn aes256_ctr_decrypt(encrypted_data: &[u8],key: &[u8], iv: &[u8])->Result<Vec<u8>,symmetriccipher::SymmetricCipherError>{
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    let mut decoder=CtrMode::new(AesSafe256Encryptor::new(key),iv.to_vec());
    decoder.decrypt(&mut read_buffer,&mut write_buffer,true)?;

    final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
    Ok(final_result)
}
```

### [tiny-keccak](https://github.com/debris/tiny-keccak)
SHA3、Keccak、SHAKE算法的实现，包含如下算法：

- shake128
- shake256
- keccak224
- keccak256
- keccak384
- keccak512
- sha3_224
- sha3_256
- sha3_384
- sha3_512

#### 关于Keccak算法与SHA3
Keccak是一种被选定为SHA-3标准的单向散列函数算法。Keccak可以生成任意长度的散列值，但为了配合SHA-2的散列值长度，SHA-3标准中规定了SHA3-224、SHA3-256、SHA3-384、SHA3-512这4种版本。在输入数据的长度上限方面，SHA-1为${2^{64}-1}$比特，SHA2为${2^{128}-1}$比特，而SHA-3则没有长度限制。

此外，FIPS 202中还规定了两个可输出任意长度散列值的函数（extendable-output function,XOF），分别为SHAKE128和SHAKE256。

#### 代码示例：
```rust
extern crate rustc_hex;
extern crate tiny_keccak;

use rustc_hex::{FromHex,ToHex};
use tiny_keccak::Keccak;

fn main() {
    let hello_str = "hello world".as_bytes().to_hex();
    let data=hello_str.from_hex().unwrap();
    let mut res:[u8;32]=[0;32];
    let expected: &[u8] = &[
        0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04,
        0x09, 0x99, 0xaa, 0xc8, 0x9e, 0x76, 0x22, 0xf3,
        0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94,
        0xa3, 0x1c, 0x3b, 0xfb, 0xf2, 0x4e, 0x39, 0x38
    ];

    let mut sha3=Keccak::new_sha3_256();
    sha3.update(&data);
    sha3.finalize(&mut res);
    assert_eq!(&res,expected);

    let mut keccak256=Keccak::new_keccak256();
    keccak256.update(&data);
    keccak256.finalize(&mut res);
    assert_ne!(&res,expected);
}

```

#### 标准文档

[NIST.FIPS.202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf)


>参考文档   
[A state-of-the-art Diffie-Hellman function](http://cr.yp.to/ecdh.html)     
[Curve25519: new Diffie-Hellman speed records](http://cr.yp.to/ecdh/curve25519-20060209.pdf)        
[Ed25519: high-speed high-security signatures](http://ed25519.cr.yp.to/)        
[High-speed high-security signatures](http://ed25519.cr.yp.to/ed25519-20110926.pdf)     
[HMAC: Keyed-Hashing for Message Authentication](https://tools.ietf.org/html/rfc2104)       