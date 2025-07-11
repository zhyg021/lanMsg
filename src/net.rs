use crate::protocol::extract_string_part;
use crate::config::AppConfig;
use crate::protocol::{IpMsgPacket, commands};
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::RwLock;

pub const IPMSG_PORT: u16 = 2425;
const FILE_PORT: u16 = 2426;

#[derive(Debug, Clone)]
pub struct OnlineUser {
    pub username: String,
    pub hostname: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct IpMsgServer {
    socket: Arc<UdpSocket>, // 使用 Arc 共享 socket
    users: Arc<RwLock<HashMap<String, SocketAddr>>>,
    default_bind: String,
}

impl IpMsgServer {
    /// 创建新实例（如果 addr 为空则使用默认值）
    pub async fn new(addr: Option<String>) -> anyhow::Result<Self> {
        let bind_addr = addr.unwrap_or_else(|| format!("0.0.0.0:{}", IPMSG_PORT));

        let socket = Arc::new(UdpSocket::bind(&bind_addr).await?);
        socket.set_broadcast(true)?;

        Ok(Self {
            socket,
            users: Arc::new(RwLock::new(HashMap::new())),
            default_bind: bind_addr,
        })
    }

    /// 获取实际绑定地址
    pub fn bound_addr(&self) -> &str {
        &self.default_bind
    }

    pub async fn broadcast(&self, packet: &IpMsgPacket) -> Result<()> {
        self.socket
            .send_to(
                packet.encode().as_bytes(),
                format!("255.255.255.255:{}", IPMSG_PORT),
            )
            .await?;
        Ok(())
    }

    pub async fn send_to(&self, packet: &IpMsgPacket, addr: &SocketAddr) -> Result<()> {
        self.socket
            .send_to(packet.encode().as_bytes(), addr)
            .await?;
        Ok(())
    }

    pub async fn listen<F>(&self, callback: F, config: Arc<AppConfig>) -> Result<()>
    where
        F: Fn(IpMsgPacket, SocketAddr),
    {
        let mut buf = [0; 1024];
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u8 = 5;

        loop {
            // 1. 接收数据
            let (len, addr) = match self.socket.recv_from(&mut buf).await {
                Ok(res) => {
                    consecutive_errors = 0;
                    res
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("[Error] Receive failed ({}): {}", consecutive_errors, e);

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("[Fatal] Too many errors, shutting down listener");
                        return Err(e.into());
                    }
                    continue;
                }
            };
            println!("[Recv] {} bytes from {}", len, addr);

            // 1. 根据配置解码原始字节
            match IpMsgPacket::decode_with_config(&buf[..len], &config) {
                Ok(packet) => {
                    println!(
                        "[Recv] From {}: {}@{} (Cmd: {:#x})",
                        addr, packet.sender_name, packet.group_name, packet.command
                    );
                    self.handle_packet(&packet, &addr).await;
                    callback(packet, addr);
                }
                Err(e) => {
                    // 调试用：输出原始十六进制
                    let hex_str = buf[..len]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    println!(
                        "[Warn] Decode failed from {}: {}\nRaw({} bytes): {}",
                        addr, e, len, hex_str
                    );
                }
            }

            // // 1. 提取可打印字符串部分
            // let string_part = extract_string_part(&buf[..len]);
            // println!("[Debug] Received ({} bytes): {}", len, string_part);

            // // 2. 尝试解析协议包
            // match IpMsgPacket::decode(&string_part) {
            //     Ok(packet) => {
            //         self.handle_packet(&packet, &addr).await;
            //         callback(packet, addr);
            //     }
            //     Err(e) => {
            //         println!(
            //             "[Warn] Decode failed from {}: {} (Raw: {})",
            //             addr, e, string_part
            //         );
            //     }
            // }
        }
    }

    /// 获取当前在线用户（基础版）
    pub async fn get_online_users_basic(&self) -> Vec<String> {
        self.users.read().await.keys().cloned().collect()
    }

    /// 获取带详细信息的在线用户
    pub async fn get_online_users(&self) -> Vec<OnlineUser> {
        self.users
            .read()
            .await
            .iter()
            .map(|(name, addr)| {
                let (username, hostname) = name.split_once('@').unwrap_or(("unknown", "unknown"));

                OnlineUser {
                    username: username.to_string(),
                    hostname: hostname.to_string(),
                    ip: addr.ip().to_string(),
                    port: addr.port(),
                }
            })
            .collect()
    }

    pub async fn get_user_addr(&self, username: &str) -> Option<SocketAddr> {
        let users = self.users.read().await;
        users.get(username).cloned()
    }
    // 更新 handle_packet 存储完整用户名
    async fn handle_packet(&self, packet: &IpMsgPacket, addr: &SocketAddr) {
        let mut users = self.users.write().await;
        let username = format!("{}@{}", packet.sender_name, packet.sender_host);
        let command = packet.command & 0xff;
        match command {
            commands::BR_ENTRY => {
                users.insert(username, *addr);
            }
            commands::IPMSG_ANSENTRY => {
                users.insert(username, *addr);
            }
            commands::BR_EXIT => {
                users.remove(&username);
            }
            _ => {}
        }
    }
}
