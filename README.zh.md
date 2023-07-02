# 自动设置 Cloudflare 优选 IP

自动为 Cloudflare 域名 DNS 记录设置 Cloudflare 优选 IP。

## 安装

通过 Cargo 安装

```bash
cargo install cf-dynamic-best
```

接下来准备好 [CloudflareSpeedTest](https://github.com/XIU2/CloudflareSpeedTest)。下面的示例来源于他们的 README。不同平台请选择不同二进制文件，例子中是基于 AMD64 和 Linux 的。

```bash
# 如果是第一次使用，则建议创建新文件夹（后续更新时，跳过该步骤）
mkdir CloudflareST

# 进入文件夹（后续更新，只需要从这里重复下面的下载、解压命令即可）
cd CloudflareST

# 下载 CloudflareST 压缩包（自行根据需求替换 URL 中 [版本号] 和 [文件名]）
wget -N https://github.com/XIU2/CloudflareSpeedTest/releases/download/v2.2.4/CloudflareST_linux_amd64.tar.gz
# 如果你是在国内服务器上下载，那么请使用下面这几个镜像加速：
# wget -N https://download.fastgit.org/XIU2/CloudflareSpeedTest/releases/download/v2.2.4/CloudflareST_linux_amd64.tar.gz
# wget -N https://ghproxy.com/https://github.com/XIU2/CloudflareSpeedTest/releases/download/v2.2.4/CloudflareST_linux_amd64.tar.gz
# 如果下载失败的话，尝试删除 -N 参数（如果是为了更新，则记得提前删除旧压缩包 rm CloudflareST_linux_amd64.tar.gz ）

# 解压（不需要删除旧文件，会直接覆盖，自行根据需求替换 文件名）
tar -zxf CloudflareST_linux_amd64.tar.gz

# 赋予执行权限
chmod +x CloudflareST
```

## 使用

```bash
$ cf-dynamic-best  --help
A tool to automatically set best Cloudflare IP for a Cloudflare DNS record

Usage: cf-dynamic-best --config-dir <FILE> --cloudflare-st-dir <FILE>

Options:
  -c, --config-dir <FILE>         
  -s, --cloudflare-st-dir <FILE>  
  -h, --help                      Print help
  -V, --version                   Print version
```

* `--config` 为配置文件路径
* `--cloudflare-st-dir` 为 `CloudflareST` 可执行文件的目录。在上面的例子中就是 `./CloudflareST`

## 配置

参考 [`config.template.zh.yaml`](config.template.zh.yaml) 的中文注释。