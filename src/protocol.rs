use serde::{Serialize, Deserialize};

/// IPMsg 报文格式
#[derive(Debug, Serialize, Deserialize)]
pub struct IpMsgPacket {
    pub version: String,
    pub packet_no: u32,
    pub sender_name: String,
    pub sender_host: String,
    pub command: u32,
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
    pub fn decode(s: &str) -> anyhow::Result<Self> {
        // 先清理可能的垃圾数据
        let clean_str = s.split('\0').next().unwrap_or(s).trim();
        
        let parts: Vec<&str> = clean_str.split(':').collect();
        if parts.len() < 6 {
            return Err(anyhow::anyhow!("Invalid IPMsg packet format"));
        }

        Ok(Self {
            version: parts[0].to_string(),
            packet_no: parts[1].parse()?,
            sender_name: parts[2].to_string(),
            sender_host: parts[3].to_string(),
            command: parts[4].parse()?,
            additional_msg: parts[5].to_string(),
        })
    }
}

/// 命令常量
pub mod commands {
    pub const BR_ENTRY: u32 = 0x00000001;  // 上线通知
    pub const BR_EXIT: u32 = 0x00000002;    // 下线通知
    pub const IPMSG_ANSENTRY: u32 = 0x00000003;     //通报新上线
    pub const IPMSG_BR_ABSENCE: u32 = 0x00000004;   //更改为离开状态
    pub const MSG: u32 = 0x00000020;       // 文本消息
    pub const FILE: u32 = 0x00000060;      // 文件传输
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_string() {
        assert_eq!(
            extract_string_part(b"1:100:Alice:PC-1:32:Hello\x00\x01"),
            "1:100:Alice:PC-1:32:Hello"
        );

        assert_eq!(
            extract_string_part(b"1_iptux 0.76:100:a:a-PC-1:259:Hello\x00\x00\x01"),
            "1_iptux 0.76:100:a:a-PC-1:259:Hello"
        );
        
        assert_eq!(
            extract_string_part(b"Text\xFFMore"),
            "Text"
        );
    }
}