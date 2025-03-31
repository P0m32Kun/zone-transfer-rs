# Zone-Transfer-rs

Zone-Transfer-rs is a simple zone transfer tool written in rust.

You can use it to check if a domain is vulnerable to zone transfer attacks.

Only for Linux and macOS.

Used `dig` to query DNS servers.

## Usage

```bash
Usage: zone-transfer-rs [OPTIONS]

Options:
  -d, --domain <DOMAIN>    要检测的单个域名
  -f, --file <FILE>        包含多个域名的文件路径
      --stdin              从标准输入读取域名
  -t, --threads <THREADS>  并发线程数 [default: 10]
  -s, --server <SERVER>    指定DNS服务器(不自动查询NS记录)
      --debug              显示详细调试信息
  -h, --help               Print help
  -V, --version            Print version
```
## Install

```bash
cargo build --release
```
