use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::Path,
    fs,
};
use anyhow::{Context, Result};

// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub network: NetworkConfig,
    
    #[serde(default)]
    pub user: UserConfig,
    
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default)]
    pub encoding: EncodingConfig,
}

// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_bind_ip")]
    pub bind_ip: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
    
    #[serde(default = "default_broadcast_ip")]
    pub broadcast_ip: String,
    
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_user_name")]
    pub name: String,
    
    #[serde(default = "default_user_host")]
    pub host: String,
    
    #[serde(default)]
    pub auto_login: bool,
    pub group: String,
}

// 编码格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    #[serde(default = "default_gbk")]
    pub protocol: String, // 协议编码 (gbk/utf8)
    #[serde(default = "default_utf8")]
    pub display: String,  // 显示编码
}

// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    #[serde(default)]
    pub dump_packets: bool,
}

// 默认值函数
fn default_bind_ip() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 2425 }
fn default_broadcast_ip() -> String { "255.255.255.255".to_string() }
fn default_timeout_secs() -> u64 { 3 }
fn default_user_name() -> String { "anonymous".to_string() }
fn default_user_host() -> String { "localhost".to_string() }
fn default_user_group() -> String { "group".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_gbk() -> String { "gbk".to_string() }
fn default_utf8() -> String { "utf-8".to_string() }

// 实现默认配置
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            user: UserConfig::default(),
            debug: DebugConfig::default(),
            encoding: EncodingConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_ip: default_bind_ip(),
            port: default_port(),
            broadcast_ip: default_broadcast_ip(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: default_user_name(),
            host: default_user_host(),
            group: default_user_group(),
            auto_login: false,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            dump_packets: false,
        }
    }
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self{
            protocol: default_gbk(),
            display: default_utf8(),
        }
    }
}

// 配置方法实现
impl AppConfig {
    /// 从文件加载配置
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let config = match fs::read_to_string(path) {
            Ok(content) => {
                let cfg: Self = toml::from_str(&content)
                    .context("Failed to parse config file")?;
                
                if !cfg.network.is_valid() {
                    eprintln!("Invalid network config, using defaults");
                    Self::default()
                } else {
                    cfg
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("Config file not found, using defaults");
                Self::default()
            }
            Err(e) => return Err(e.into()),
        };
        
        Ok(config)
    }

    /// 获取绑定地址
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.network.bind_ip, self.network.port)
    }

    /// 获取广播地址
    pub fn broadcast_addr(&self) -> String {
        format!("{}:{}", self.network.broadcast_ip, self.network.port)
    }
}

impl NetworkConfig {
    /// 验证网络配置有效性
    pub fn is_valid(&self) -> bool {
        let ip_valid = self.bind_ip.parse::<IpAddr>().is_ok() 
            && self.broadcast_ip.parse::<IpAddr>().is_ok();
        let port_valid = self.port > 1024 && self.port < 65535;
        ip_valid && port_valid
    }
}

// 单元测试
// #[cfg(test)]
/*  mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    // use crate::config::fs::write;
    // use std::fmt::write;
    // use std::fs::write;
    // use std::ptr::write;

    #[test]
    fn test_load_config() {
        let toml_content = r#"
            [network]
            bind_ip = "192.168.1.100"
            port = 3000
            
            [user]
            name = "test_user"
            auto_login = true
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::write(&mut file, toml_content).unwrap();
        
        let config = AppConfig::load(file.path()).unwrap();
        assert_eq!(config.network.bind_ip, "192.168.1.100");
        assert_eq!(config.user.name, "test_user");
        assert!(config.user.auto_login);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.network.port, 2425);
        assert!(!config.debug.dump_packets);
    }
}

    */