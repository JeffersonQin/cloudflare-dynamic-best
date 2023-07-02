# Dynamic Best Cloudflare

A tool to automatically select best Cloudflare IP for a Cloudflare DNS record.

## Install

Install the binary via cargo and crates.io,

```bash
cargo install cf-dynamic-best
```

Then, get [CloudflareSpeedTest](https://github.com/XIU2/CloudflareSpeedTest) ready. Following is the installation guide from their repository, different platform may differ, the following is the guide for linux and amd64.

```bash
mkdir CloudflareST

cd CloudflareST

# select for your platform and arch in their release page
wget -N https://github.com/XIU2/CloudflareSpeedTest/releases/download/v2.2.4/CloudflareST_linux_amd64.tar.gz

tar -zxf CloudflareST_linux_amd64.tar.gz

chmod +x CloudflareST
```

## Usage

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

* `--config` is the file for config path.
* `--cloudflare-st-dir` is the directory where `CloudflareST` binary file is located.

## Configuration

Take a look at [`config.template.en.yaml`](config.template.en.yaml) for configuration file with English comment.
