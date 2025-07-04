use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ipmsg", version = "0.1")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, default_value = "anonymous")]
    pub name: String,

    #[arg(short, long, default_value = "localhost")]
    pub host: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 发送消息给指定用户
    Send {
        recipient: String,
        message: String,
    },
    /// 广播消息给所有人
    Broadcast {
        message: String,
    },
    /// 列出在线用户
    List,
    /// 启动交互式会话
    Chat,
}