use chrono::{DateTime, Local};
use clap::Parser;
use colored::Colorize;
use csv::ReaderBuilder;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_yaml;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{fs, thread};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config_dir: PathBuf,
    #[arg(short = 's', long, value_name = "FILE")]
    cloudflare_st_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    cloudflare: CloudflareConfig,
    email: EmailConfig,
}

#[derive(Debug, Deserialize)]
struct CloudflareConfig {
    x_auth_key: String,
    zone_id: String,
    email: String,
    dns_record_name: String,
    website_url: String,
    interval: u64,
    retry_interval: u64,
    fallback_raw: String,
}

#[derive(Debug, Deserialize)]
struct EmailConfig {
    enable: bool,
    email: String,
    smtp_username: String,
    smtp_password: String,
    smtp_server: String,
    subject_success: String,
    body_success: String,
    subject_failed: String,
    body_failed: String,
}

#[derive(Debug, Deserialize)]
struct CsvEntry {
    #[serde(rename = "IP 地址")]
    ip_address: String,
    #[serde(rename = "下载速度 (MB/s)")]
    download_speed: f64,
    // Add other fields as necessary
}

#[derive(Serialize)]
struct DnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

fn read_config_file(file_path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    println!("{} : {}", get_time_str(), "Start reading config file");

    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_yaml::from_str(&contents)?;

    println!("{} : {}", get_time_str(), "Config load success".green());
    Ok(config)
}

fn delete_result_file(cloudflare_st_file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} : {}", get_time_str(), "Try to delete previous result");

    if let Err(e) = fs::remove_file(cloudflare_st_file.join("result.csv")) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    println!("{} : {}", get_time_str(), "Delete previous success".green());
    Ok(())
}

fn run_tool(cloudflare_st_file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} : {}", get_time_str(), "Start running tool");

    let mut output = Command::new("./CloudflareST")
        .arg("-dn")
        .arg("20")
        .arg("-tl")
        .arg("250")
        .current_dir(cloudflare_st_file)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let _ = output.wait();

    println!("{} : {}", get_time_str(), "Run tool success".green());
    return Ok(());
}

fn parse_csv_result(
    cloudflare_st_file: &PathBuf,
) -> Result<(String, f64), Box<dyn std::error::Error>> {
    println!("{} : {}", get_time_str(), "Start parsing result csv");

    let mut file = File::open(cloudflare_st_file.join("result.csv"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut reader = ReaderBuilder::new().from_reader(contents.as_bytes());

    // Skip the header row if it exists
    let header = reader.headers()?.clone();

    // Read the first entry
    if let Some(result) = reader.records().next() {
        let record = result?;
        let entry: CsvEntry = record.deserialize(Some(&header))?;

        let ip_address = entry.ip_address;
        let download_speed = entry.download_speed;

        println!("{} : {}", get_time_str(), "Parse result success".green());
        Ok((ip_address, download_speed))
    } else {
        Err("No entries found in the CSV result.".into())
    }
}

async fn update_dns_record(
    config: &Config,
    ip: String,
    updating_raw: bool,
) -> Result<(), Box<dyn Error>> {
    println!("{} : {}", get_time_str(), "Start updating dns record");

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Auth-Email",
        HeaderValue::from_str(&config.cloudflare.email)?,
    );
    headers.insert(
        "X-Auth-Key",
        HeaderValue::from_str(&config.cloudflare.x_auth_key)?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Fetch DNS records
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        config.cloudflare.zone_id
    );
    let response = client
        .get(&url)
        .headers(headers.clone())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let result_arr = response["result"]
        .as_array()
        .ok_or(format!("Invalid response: {}", response))?;

    // Find the matching DNS record
    let mut site_id = "";
    for result in result_arr {
        let record_type = result["type"]
            .as_str()
            .ok_or(format!("Invalid response: {}", response))?;
        let name = result["name"]
            .as_str()
            .ok_or(format!("Invalid response: {}", response))?;
        if record_type == "A" && name == &config.cloudflare.website_url {
            site_id = result["id"]
                .as_str()
                .ok_or(format!("Invalid response: {}", response))?;
            break;
        }
    }

    // Update the DNS record
    let update_url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        config.cloudflare.zone_id, site_id
    );
    let dns_record = DnsRecord {
        record_type: String::from("A"),
        name: config.cloudflare.dns_record_name.clone(),
        content: ip.clone(), // Implement your logic to get the IP address
        ttl: 1,
        proxied: false,
    };
    let response = client
        .put(&update_url)
        .headers(headers)
        .json(&dns_record)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let success = response["success"]
        .as_bool()
        .ok_or(format!("Invalid response: {}", response))?;

    // Handle the result
    if success {
        if updating_raw {
            println!(
                "{} : UPLOAD SUCCESS : {}",
                get_time_str().yellow(),
                ip.yellow()
            );
        } else {
            println!(
                "{} : UPLOAD SUCCESS : {}",
                get_time_str().green(),
                ip.green()
            );
        }
        Ok(())
    } else {
        println!("{} : {}", get_time_str(), "UPLOAD FAILED".red());
        println!("SERVER RESULT: {}", response.to_string().red());
        Err("Upload failed".into())
    }
}

fn get_time_str() -> String {
    let local: DateTime<Local> = Local::now();
    let formatted_time = local.format("%Y/%m/%d %H:%M:%S").to_string();
    return formatted_time;
}

fn send_email(config: &Config, ip: String) -> Result<(), Box<dyn Error>> {
    println!("{} : {}", get_time_str(), "Try to send email notification");

    let subject = if config.cloudflare.fallback_raw == ip {
        config.email.subject_success.replace("%IP%", &ip)
    } else {
        config.email.subject_failed.replace("%IP%", &ip)
    };
    let body = if config.cloudflare.fallback_raw == ip {
        config.email.body_success.replace("%IP%", &ip)
    } else {
        config.email.body_failed.replace("%IP%", &ip)
    };

    let email = Message::builder()
        .from(config.email.email.clone().parse().unwrap())
        .to(config.email.email.clone().parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .unwrap();

    let creds = Credentials::new(
        config.email.smtp_username.clone(),
        config.email.smtp_password.clone(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&config.email.smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => {
            println!("{} : {}", get_time_str(), "SEND EMAIL SUCCESS".green());
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = read_config_file(cli.config_dir).unwrap();
    let mut using_raw = false;

    loop {
        // delete result file
        match delete_result_file(&cli.cloudflare_st_dir) {
            Err(e) => {
                println!(
                    "{} : {}",
                    get_time_str(),
                    "DELETE LAST EXECUTION FILE FAILED".red()
                );
                println!("Error Info: {}", e.to_string().red());
                thread::sleep(Duration::from_secs(config.cloudflare.retry_interval));
                continue;
            }
            _ => {}
        }
        // run tool
        match run_tool(&cli.cloudflare_st_dir) {
            Err(e) => {
                println!("{} : {}", get_time_str(), "RUN TOOL FAILED".red());
                println!("Error Info: {}", e.to_string().red());
                thread::sleep(Duration::from_secs(config.cloudflare.retry_interval));
                continue;
            }
            _ => {}
        }
        // parse csv result
        let ip = match parse_csv_result(&cli.cloudflare_st_dir) {
            Err(e) => {
                println!("{} : {}", get_time_str(), "PARSE CSV RESULT FAILED".red());
                println!("Error Info: {}", e.to_string().red());
                thread::sleep(Duration::from_secs(config.cloudflare.retry_interval));
                continue;
            }
            Ok((ip, download_speed)) => {
                if download_speed < 0.05 {
                    config.cloudflare.fallback_raw.clone()
                } else {
                    ip
                }
            }
        };
        // update dns record
        let current_using_raw = match update_dns_record(
            &config,
            ip.clone(),
            ip == config.cloudflare.fallback_raw,
        )
        .await
        {
            Err(e) => {
                println!("{} : {}", get_time_str(), "UPDATE DNS RECORD FAILED".red());
                println!("Error Info: {}", e.to_string().red());
                thread::sleep(Duration::from_secs(config.cloudflare.retry_interval));
                continue;
            }
            Ok(_) => {
                if ip == config.cloudflare.fallback_raw {
                    true
                } else {
                    false
                }
            }
        };
        // send email if
        // 1. functionality is enabled
        // 2. `using_raw` changed
        if config.email.enable {
            if current_using_raw ^ using_raw {
                match send_email(&config, ip.clone()) {
                    Err(e) => {
                        println!("{} : {}", get_time_str(), "SEND EMAIL FAILED".red());
                        println!("Error Info: {}", e.to_string().red());
                        thread::sleep(Duration::from_secs(config.cloudflare.retry_interval));
                        continue;
                        // if send email failed, we should not update `using_raw`
                        // so that we can send email next time
                    }
                    Ok(_) => {
                        using_raw = current_using_raw;
                    }
                }
            }
        }
        // everything success, sleep for a while
        thread::sleep(Duration::from_secs(config.cloudflare.interval));
    }
}
