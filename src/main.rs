mod cli;
mod config;
mod net;
mod protocol;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use net::IpMsgServer;
use protocol::{IpMsgPacket, commands};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // let server = IpMsgServer::new().await?;
    // 1. 加载配置（带回退逻辑）
    let config = match config::AppConfig::load("config.toml") {
        Ok(cfg) if cfg.network.is_valid() => cfg,
        _ => {
            println!("Using default configuration");
            config::AppConfig::default()
        }
    };

    let config_clone = Arc::new(config.clone());

    // 2. 初始化服务器（自动处理空地址）
    let server = net::IpMsgServer::new(Some(config.bind_addr())).await?;
    println!("Bound to {}", server.bound_addr());

    let server_clone = server.clone();
    // 消息接收线程
    tokio::spawn(async move {
        let _ = server_clone
            .listen(
                |packet, _| {
                    println!("\n[{}] {}", packet.sender_name, packet.additional_msg);
                },
                config_clone.clone(),
            )
            .await;
    });

    // 广播上线通知
    let entry_packet = IpMsgPacket {
        version: "lanMsg 0.1".to_string(),
        packet_no: rand::random(),
        sender_name: cli.name.clone(),
        sender_host: cli.host.clone(),
        command: commands::BR_ENTRY,
        additional_msg: "".to_string(),
        group_name: "".to_string(),
        ..Default::default()
    };
    server.broadcast(&entry_packet).await?;

    match cli.command {
        cli::Commands::Send { recipient, message } => {
            let recipient_full = format!("{}@{}", recipient, cli.host);
            if let Some(addr) = server.get_user_addr(&recipient).await {
                let packet = IpMsgPacket {
                    version: "lanMsg 0.1".to_string(),
                    packet_no: rand::random(),
                    sender_name: cli.name.clone(),
                    sender_host: cli.host.clone(),
                    command: commands::MSG,
                    additional_msg: message,
                    group_name: "".to_string(),
                    ..Default::default()
                };
                server.send_to(&packet, &addr).await?;
            } else {
                println!("User {} not found", recipient);
            }
        }
        cli::Commands::Broadcast { message } => {
            let packet = IpMsgPacket {
                version: "lanMsg 0.1".to_string(),
                packet_no: rand::random(),
                sender_name: cli.name.clone(),
                sender_host: cli.host.clone(),
                command: commands::MSG,
                additional_msg: message,
                group_name: "".to_string(),
                ..Default::default()
            };
            server.broadcast(&packet).await?;
        }
        cli::Commands::List => {
            // 等待2秒收集响应
            println!("Fetching online users...");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let users = server.get_online_users().await;
            println!("Online users ({}):", users.len());

            if users.is_empty() {
                println!("No online users found");
            } else {
                println!("┌──────────────┬──────────────┬──────────────┬──────┐");
                println!(
                    "│ {:<12} │ {:<12} │ {:<12} │ {:<4} │",
                    "Username", "Host", "IP", "Port"
                );
                println!("├──────────────┼──────────────┼──────────────┼──────┤");

                for user in users {
                    println!(
                        "│ {:<12} │ {:<12} │ {:<12} │ {:<4} │",
                        user.username, user.hostname, user.ip, user.port
                    );
                }
                println!("└──────────────┴──────────────┴──────────────┴──────┘");
            }
        }
        cli::Commands::Chat => {
            let (tx, mut rx) = mpsc::channel(100);

            // 用户输入线程
            // tokio::spawn(async move {
            //     let mut stdin = io::BufReader::new(io::stdin());
            //     loop {
            //         let mut line = String::new();
            //         stdin.read_line(&mut line).await.unwrap();
            //         let _ = tx.send(line.trim().to_string()).await;
            //     }
            // });

            // 用户输入处理
            let mut stdin = io::BufReader::new(io::stdin());
            loop {
                print!("> ");
                let mut input = String::new();
                stdin.read_line(&mut input).await?;
                let mut input = input.trim().to_string();

                // 退出命令处理
                if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
                    println!("Exiting chat...");
                    break;
                }

                if input.is_empty() {
                    continue;
                }

                // let packet = protocol::IpMsgPacket {
                //     version: 1,
                //     packet_no: rand::random(),
                //     sender_name: cli.name.clone(),
                //     sender_host: cli.host.clone(),
                //     command: protocol::commands::MSG,
                //     additional_msg: input.clone(),
                // };
                // server.broadcast(&packet, broadcast_addr.clone()).await?;
                let _ = tx.send(input).await;
            }
        }
    }

    // 发送下线通知
    let exit_packet = IpMsgPacket {
        version: "lanMsg 0.1".to_string(),
        packet_no: rand::random(),
        sender_name: cli.name.clone(),
        sender_host: cli.host.clone(),
        command: commands::BR_EXIT,
        additional_msg: "".to_string(),
        group_name: "".to_string(),
        ..Default::default()
    };
    server.broadcast(&exit_packet).await?;

    Ok(())
}
