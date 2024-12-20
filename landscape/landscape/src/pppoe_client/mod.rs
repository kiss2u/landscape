pub mod pppoe_client_v2;

const DEFAULT_TIME_OUT: u64 = 3;
const LCP_ECHO_INTERVAL: u64 = 20;

/// 接近于永久不触发的 秒数  s    m    h
// const PAUSE_FOREVER: u64 = 60 * 60 * 24 * 365 * 10;

const DEFAULT_CLIENT_MRU: u16 = 1492;
const ETH_P_PPOED: u16 = 0x8863;
const ETH_P_PPOES: u16 = 0x8864;
