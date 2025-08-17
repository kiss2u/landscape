use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Copy, Eq, Hash, TS)]
#[ts(export, export_to = "flow.ts")]
#[serde(tag = "t")]
#[serde(rename_all = "snake_case")]
pub enum FlowMark {
    /// 按照当前 Flow 的配置继续
    #[default]
    KeepGoing,
    /// 忽略 Flow 转发配置, 直接发送
    /// 对于默认 flow 无效果
    Direct,
    /// 丢弃匹配的数据包
    Drop,
    /// 转发到指定的流
    Redirect { flow_id: u8 },
    /// 允许 NAT 端口共享
    AllowReusePort,
}

impl FlowMark {
    pub fn need_insert_in_ebpf_map(&self) -> bool {
        match self {
            FlowMark::KeepGoing => false,
            _ => true,
        }
    }
}

const FLOW_KEEP_GOING: u8 = 0;
const FLOW_DIRECT: u8 = 1;
const FLOW_DROP: u8 = 2;
const FLOW_REDIRECT: u8 = 3;
const FLOW_ALLOW_REUSE: u8 = 4;

const FLOW_ID_MASK: u32 = 0x000000FF;
const FLOW_ACTION_MASK: u32 = 0x0000FF00;

impl From<u32> for FlowMark {
    fn from(value: u32) -> Self {
        let action = ((value & FLOW_ACTION_MASK) >> 8) as u8;
        let flow_id = (value & FLOW_ID_MASK) as u8;

        match action {
            FLOW_KEEP_GOING => FlowMark::KeepGoing,
            FLOW_DIRECT => FlowMark::Direct,
            FLOW_DROP => FlowMark::Drop,
            FLOW_REDIRECT => FlowMark::Redirect { flow_id },
            FLOW_ALLOW_REUSE => FlowMark::AllowReusePort,
            _ => FlowMark::KeepGoing,
        }
    }
}

impl Into<u32> for FlowMark {
    fn into(self) -> u32 {
        match self {
            FlowMark::KeepGoing => (FLOW_KEEP_GOING as u32) << 8,
            FlowMark::Direct => (FLOW_DIRECT as u32) << 8,
            FlowMark::Drop => (FLOW_DROP as u32) << 8,
            FlowMark::Redirect { flow_id } => (FLOW_REDIRECT as u32) << 8 | (flow_id as u32),
            FlowMark::AllowReusePort => (FLOW_ALLOW_REUSE as u32) << 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u32() {
        // action = 1 (Direct)
        assert_eq!(FlowMark::from(0x0100), FlowMark::Direct);
        // action = 3 (Redirect), index = 5
        assert_eq!(FlowMark::from(0x0305), FlowMark::Redirect { flow_id: 5 });
        // action = 4 (SymmetricNat)
        assert_eq!(FlowMark::from(0x0400), FlowMark::AllowReusePort);
    }

    #[test]
    fn test_into_u32() {
        // Direct -> action = 1
        let mark: u32 = FlowMark::Direct.into();
        assert_eq!(mark, 0x0100);

        // Redirect { flow_id: 5 } -> action = 3, index = 5
        let mark: u32 = FlowMark::Redirect { flow_id: 5 }.into();
        assert_eq!(mark, 0x0305);

        // SymmetricNat -> action = 4
        let mark: u32 = FlowMark::AllowReusePort.into();
        assert_eq!(mark, 0x0400);
    }
}
