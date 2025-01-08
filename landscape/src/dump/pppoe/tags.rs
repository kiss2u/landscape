macro_rules! pppoe_tags {
    ( $( { $num:expr, $name:ident, $comment:expr, $type:ty } ),* $(,)? ) => {
        #[derive(Debug, Clone)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[repr(u16)]
        pub enum PPPoETag {
            $(
                #[doc = $comment]
                $name($type) = $num,
            )*

            #[doc = "other"]
            Other(u16, Vec<u8>) = 65535,
        }

        impl PPPoETag {
            pub fn from_bytes(data: &[u8]) -> Vec<PPPoETag> {
                let mut tags = vec![];

                let mut start_index = 0;
                loop {
                    if 4 + start_index > data.len() {
                        break;
                    }
                    let tag_type = ((data[start_index] as u16) << 8) | (data[start_index + 1] as u16);
                    let tag_len = ((data[start_index + 2] as u16) << 8) | (data[start_index + 3] as u16);
                    let next_index = 4 + start_index + tag_len as usize;
                    if next_index > data.len() {
                        break;
                    }
                    let tag_data =
                        if tag_len == 0 { vec![] } else { data[start_index + 4..next_index].to_vec() };
                    match tag_type {
                        $(
                            $num => {
                                let option = <$type as TagDefined>::decode(&tag_data);
                                if let Some(option) = option {
                                    tags.push(Self::$name(option));
                                } else {
                                    tags.push(PPPoETag::Other($num, tag_data.to_vec()));
                                }
                            }
                        )*
                        tag_type_value => {
                            tags.push(PPPoETag::Other(tag_type_value, tag_data.to_vec()));
                            break;
                        },
                    }
                    start_index = next_index;
                }
                tags
            }


            pub fn decode_options(self) -> Vec<u8> {
                match self {
                    $(
                        PPPoETag::$name(value) => {
                            let data = <$type as TagDefined>::encode(value);
                            let length = data.len() as u16;
                            [($num as u16).to_be_bytes().to_vec(), length.to_be_bytes().to_vec(), data].concat()
                        }
                    )*
                    PPPoETag::Other(tag_code, data) => {
                        let length = data.len() as u16;
                        [tag_code.to_be_bytes().to_vec(), length.to_be_bytes().to_vec(), data].concat()
                    }
                }
            }
        }
    }

}

pub trait TagDefined {
    fn decode(data: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn encode(self) -> Vec<u8>;
}

// 定义 EmptyTag 结构体和实现 OptionDefined trait
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct EmptyTag;

impl TagDefined for EmptyTag {
    fn decode(_: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        Some(EmptyTag)
    }

    fn encode(self) -> Vec<u8> {
        vec![]
    }
}

impl TagDefined for Vec<u8> {
    fn decode(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        Some(data.to_vec())
    }

    fn encode(self) -> Vec<u8> {
        self
    }
}

impl TagDefined for u32 {
    fn decode(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    fn encode(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}
pppoe_tags! {
    {0x0000, EndOfList, "0x0000: End-Of-List", EmptyTag},
    {0x0101, ServiceName, "0x0101: Service-Name", Vec<u8>},
    {0x0102, AcName, "0x0102: AC-Name", Vec<u8>},
    {0x0103, HostUniq, "0x0103: Host-Uniq", u32},
    {0x0104, AcCookie, "0x0104: AC-Cookie", Vec<u8>},
    {0x0105, VendorSpecific, "0x0105: Vendor-Specific", Vec<u8>},
    {0x0106, Credits, "0x0106: Credits", u32},
    {0x0107, Metrics, "0x0107: Metrics", Vec<u8>},
    {0x0108, SequenceNumber, "0x0108: Sequence Number", u32},
    {0x0109, CreditScaleFactor, "0x0109: Credit Scale Factor", u32},
    {0x0110, RelaySessionId, "0x0110: Relay-Session-Id", Vec<u8>},
    {0x0111, HURL, "0x0111: HURL", Vec<u8>},
    {0x0112, MOTM, "0x0112: MOTM", Vec<u8>},
    {0x0120, PPPMaxPayload, "0x0120: PPP-Max-Payload", u32},
    {0x0121, IPRouteAdd, "0x0121: IP_Route_Add", Vec<u8>},
    {0x0201, ServiceNameError, "0x0201: Service-Name-Error", Vec<u8>},
    {0x0202, ACSystemError, "0x0202: AC-System-Error", Vec<u8>},
    {0x0203, GenericError, "0x0203: Generic-Error", Vec<u8>},
}
