use crate::config::EncodingConfig;
use crate::config::AppConfig;
use encoding_rs::{GBK, UTF_8};
use serde::{Deserialize, Serialize};

/// IPMsg 报文格式
#[derive(Debug, Serialize, Deserialize)]
pub struct IpMsgPacket {
    pub version: String,
    pub packet_no: u32,
    pub sender_user: String,
    pub sender_host: String,
    pub command: u32,
    pub sender_name: String,
    pub group_name: String,
    pub additional_msg: String,
}

impl IpMsgPacket {
    /// 编码为协议字符串
    pub fn encode(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.version,
            self.packet_no,
            self.sender_name,
            self.sender_host,
            self.command,
            self.additional_msg
        )
    }

    /// 从字符串解析
    // pub fn decode(s: &str) -> anyhow::Result<Self> {
    //     // 先清理可能的垃圾数据
    //     let clean_str = s.split('\0').next().unwrap_or(s).trim();

    //     let parts: Vec<&str> = clean_str.split(':').collect();
    //     if parts.len() < 6 {
    //         return Err(anyhow::anyhow!("Invalid IPMsg packet format"));
    //     }

    //     Ok(Self {
    //         version: parts[0].to_string(),
    //         packet_no: parts[1].parse()?,
    //         sender_name: parts[2].to_string(),
    //         sender_host: parts[3].to_string(),
    //         command: parts[4].parse()?,
    //         additional_msg: parts[5].to_string(),
    //         group_name: "group".to_string(),
    //     })
    // }

    /// 数据打包
    pub fn encode_with_config(&self, config: &AppConfig) -> Vec<u8> {
        let additional = if self.group_name.is_empty() {
            self.additional_msg.clone()
        } else {
            format!("{}\x00{}", self.sender_name, self.group_name)
        };
        
        let packet_str = format!(
            "{}:{}:{}:{}:{}:{}",
            self.version,
            self.packet_no,
            "aaMsg".to_string(),
            // self.sender_name,
            self.sender_host,
            self.command,
            additional
        );
        
        // 根据配置选择编码
        let encoder = match config.encoding.protocol.as_str() {
            "gbk" => GBK,
            _ => UTF_8,
        };
        
        encoder.encode(&packet_str).0.to_vec()
    }

    /// 增强版协议包解码
    pub fn decode_with_config(data: &[u8], config: &AppConfig) -> anyhow::Result<IpMsgPacket> {
        // 先尝试完整解码
        let decoder = match config.encoding.protocol.as_str() {
            "gbk" => GBK,
            _ => UTF_8,
        };

        let (cow, _, had_errors) = decoder.decode(data);
        if had_errors {
            // 回退到提取可打印部分
            let fallback_str = extract_string_part2(data, config);
            return Self::decode_fallback(&fallback_str);
        }

        let s = cow.trim();
        Self::parse_packet_str(s)
    }

    /// 回退解析（当完整解码失败时使用）
    fn decode_fallback(s: &str) -> anyhow::Result<IpMsgPacket> {
        // 尝试提取基本字段
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 6 {
            return Err(anyhow::anyhow!("Invalid packet format"));
        }

        Ok(IpMsgPacket {
            version: parts[0].to_string(),
            packet_no: parts[1].parse().unwrap_or(0),
            sender_user: parts[2].to_string(),
            sender_host: parts[3].to_string(),
            command: parts[4].parse().unwrap_or(0),
            sender_name: parts[5].split('\0').next().unwrap_or("").to_string(),
            group_name: parts[5].split('\0').next().unwrap_or("").to_string(),
            additional_msg: parts[5].split('\0').next().unwrap_or("").to_string(),
        })
    }

    /// 核心解析逻辑
    fn parse_packet_str(s: &str) -> anyhow::Result<IpMsgPacket> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 6 {
            return Err(anyhow::anyhow!("Invalid packet format"));
        }

        let mut additional = parts[5];
        let mut split_iter = additional.split('\x00');
        let mut group = "";
        let mut name = "";
    
        name = split_iter.next().unwrap_or_default();
        group = split_iter.next().unwrap_or_default();
        additional = split_iter.next().unwrap_or_default();

        // if let Some(pos) = additional.find('\0') {
        //     group = &additional[pos + 1..];
        //     additional = &additional[..pos];
        // }

        Ok(IpMsgPacket {
            version: parts[0].to_string(),
            packet_no: parts[1].parse()?,
            sender_user: parts[2].to_string(),
            sender_host: parts[3].to_string(),
            command: parts[4].parse()?,
            sender_name: name.to_string(),
            group_name: group.to_string(),
            additional_msg: additional.to_string(),
            
        })
    }
}

impl Default for IpMsgPacket {
    fn default() -> Self {
        Self {
            version: "lanMsg 0.1".to_string(),
            packet_no: 0,
            sender_user: "default_user".to_string(),  // 默认值
            sender_host: String::new(),
            command: 0,
            sender_name: String::new(),
            group_name: String::new(),
            additional_msg: String::new(),
        }
    }
}

/// 命令常量
pub mod commands {
    pub const BR_ENTRY: u32 = 0x00000001; // 上线通知
    pub const BR_EXIT: u32 = 0x00000002; // 下线通知
    pub const IPMSG_ANSENTRY: u32 = 0x00000003; //通报新上线
    pub const IPMSG_BR_ABSENCE: u32 = 0x00000004; //更改为离开状态
    pub const MSG: u32 = 0x00000020; // 文本消息
    pub const FILE: u32 = 0x00000060; // 文件传输
}

/// 从字节流中提取可打印字符串部分
pub(crate) fn extract_string_part(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len());
    for &byte in data {
        if byte.is_ascii_graphic() || byte == b' ' {
            result.push(byte as char);
        } else if byte == b'\0' {
            break; // 遇到空字符停止
        } else {
            break; // 遇到非可打印ASCII字符停止
        }
    }
    result.trim().to_string()
}

/// 改进的字符串提取函数（支持GBK双字节字符）
pub(crate) fn extract_string_part2(data: &[u8], config: &AppConfig) -> String {
    match config.encoding.protocol.as_str() {
        "gbk" => {
            // GBK模式：尝试解码整个字节流
            let (cow, _, _) = GBK.decode(data);
            let s = cow.trim();
            // 找到第一个非法字符位置
            s.chars()
                .take_while(|&c| c != '\0' && !c.is_control())
                .collect()
        }
        _ => {
            // UTF-8模式：传统ASCII处理
            let mut result = String::with_capacity(data.len());
            for &byte in data {
                if byte.is_ascii_graphic() || byte == b' ' {
                    result.push(byte as char);
                } else if byte == b'\0' {
                    break;
                } else {
                    break;
                }
            }
            result.trim().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_string() {
        // assert_eq!(
        //     extract_string_part(b"1:100:Alice:PC-1:32:Hello\x00\x01"),
        //     "1:100:Alice:PC-1:32:Hello"
        // );

        // assert_eq!(
        //     extract_string_part(b"1_iptux 0.76:100:a:a-PC-1:259:Hello\x00\x00\x01"),
        //     "1_iptux 0.76:100:a:a-PC-1:259:Hello"
        // );

        // assert_eq!(extract_string_part(b"Text\xFFMore"), "Text");

        let config = AppConfig {
            encoding: EncodingConfig { protocol: "gbk".into(), display: "utf-8".into() },
            ..Default::default()
        };

        // 模拟实际数据：1:12345:pc-usr:DESKTOP-ABC:0:张三\x00开发组
        let data = b"1:12345:pc-usr:DESKTOP-ABC:0:\xd5\xc5\xc8\xfd\x00\xbf\xaa\xb7\xa2\xd7\xe9\x00";
        
        let packet = IpMsgPacket::decode_with_config(data, &config).unwrap();
        
        assert_eq!(packet.version, "1");
        assert_eq!(packet.packet_no, 12345);
        assert_eq!(packet.sender_user, "pc-usr");
        assert_eq!(packet.sender_host, "DESKTOP-ABC");
        assert_eq!(packet.command, 0);
        assert_eq!(packet.sender_name, "张三");
        assert_eq!(packet.group_name, "开发组");
    }
}
