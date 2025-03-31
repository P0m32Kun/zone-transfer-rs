use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::{
    io::{self, BufRead},
    process::Command,
};
use tokio::task;

const VERSION: &str = concat!(
    "\n",
    "                           _                        __\n",                        
    " _______  _ __   ___      | |_ _ __ __ _ _ __  ___ / _| ___ _ __      _ __ ___\n", 
    "|_  / _ \\| '_ \\ / _ \\_____| __| '__/ _` | '_ \\/ __| |_ / _ \\ '__|____| '__/ __|\n",
    " / / (_) | | | |  __/_____| |_| | | (_| | | | \\__ \\  _|  __/ | |_____| |  \\__ \\\n",
    "/___\\___/|_| |_|\\___|      \\__|_|  \\__,_|_| |_|___/_|  \\___|_|       |_|  |___/\n",                                                                            
    " v0.1.0\n",
    " Author: P0m32Kun\n"
);

#[derive(Parser, Debug)]
#[command(
    version = VERSION,
    about = "DNS域传送漏洞检测工具",
    long_about = None
)]
struct Args {
    /// 要检测的单个域名
    #[arg(short, long)]
    domain: Option<String>,

    /// 包含多个域名的文件路径
    #[arg(short, long)]
    file: Option<String>,

    /// 从标准输入读取域名
    #[arg(long)]
    stdin: bool,

    /// 并发线程数
    #[arg(short, long, default_value_t = 10)]
    threads: usize,

    /// 指定DNS服务器(不自动查询NS记录)
    #[arg(short, long)]
    server: Option<String>,

    /// 显示详细调试信息
    #[arg(long)]
    debug: bool,
}

async fn get_nameservers(domain: &str) -> Result<Vec<String>> {
    let output = Command::new("dig")
        .arg("+short")
        .arg("NS")
        .arg(domain)
        .output()
        .context("查询NS记录失败")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

async fn check_domain_with_nameserver(domain: &str, nameserver: &str) -> Result<(bool, String)> {
    let output = Command::new("dig")
        .arg(format!("@{}", nameserver))
        .arg("-t")
        .arg("axfr")
        .arg(domain)
        .output()
        .context("执行dig命令失败")?;

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    let is_vulnerable = output_str.lines().any(|line| {
        line.contains(" IN ") || 
        line.contains("XFR size") ||
        (line.trim().ends_with(".") && line.split_whitespace().count() >= 4)
    });
    Ok((is_vulnerable, output_str))
}

async fn process_domain(domain: String, server: Option<String>, debug: bool) {
    let nameservers = match server {
        Some(s) => vec![s],
        None => match get_nameservers(&domain).await {
            Ok(ns) if !ns.is_empty() => ns,
            Ok(_) => {
                if debug {
                    println!("{} {}", "未找到DNS服务器:".yellow(), domain);
                }
                return;
            }
            Err(e) => {
                if debug {
                    println!("{} {}: {}", "查询DNS服务器失败:".red(), domain, e);
                }
                return;
            }
        },
    };

    for ns in nameservers {
        match check_domain_with_nameserver(&domain, &ns).await {
            Ok((is_vuln, output)) => {
                println!("{} {} {}",
                    domain,
                    ns,
                    if is_vuln { "存在域传送漏洞!🔥🔥🔥".red().bold() } else { "安全!✅✅✅".green() }
                );

                if debug {
                    println!("\n{} {} @ {}",
                        "详细结果:".bold(),
                        domain,
                        ns.yellow()
                    );
                    println!("{}", output);
                    if is_vuln {
                        println!("{}", "存在域传送漏洞!🔥🔥🔥".red().bold());
                    } else {
                        println!("{}", "安全!✅✅✅".green());
                    }
                    println!("{}", "=".repeat(50));
                }
            }
            Err(e) => {
                if debug {
                    println!("{} {} @ {}: {}", "检测失败:".red(), domain, ns, e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let domains = match (&args.domain, &args.file, args.stdin) {
        (Some(domain), None, false) => vec![domain.clone()],
        (None, Some(file_path), false) => tokio::fs::read_to_string(file_path)
            .await?
            .lines()
            .map(|s| s.to_string())
            .collect(),
        (None, None, true) => {
            let stdin = io::stdin();
            stdin.lock().lines().filter_map(|line| line.ok()).collect()
        }
        _ => anyhow::bail!("必须指定--domain、--file或--stdin参数之一"),
    };

    let mut tasks = Vec::new();
    for domain in domains {
        tasks.push(task::spawn(process_domain(
            domain, 
            args.server.clone(),
            args.debug
        )));
    }

    for task in tasks {
        task.await?;
    }

    Ok(())
}