use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    path::Path,
};

use landscape_common::{
    config::dns::{DomainConfig, DomainMatchType},
    ip_mark::IpConfig,
};
use protos::geo::{mod_Domain::Type, Domain, GeoIPListOwned, GeoSiteListOwned};

mod protos;

pub async fn read_geo_sites_from_bytes(
    contents: impl Into<Vec<u8>>,
) -> HashMap<String, Vec<DomainConfig>> {
    let mut result = HashMap::new();
    let list = GeoSiteListOwned::try_from(contents.into()).unwrap();

    for entry in list.proto().entry.iter() {
        let domains = entry.domain.iter().map(convert_domain_from_proto).collect();
        result.insert(entry.country_code.to_string(), domains);
    }
    result
}

pub async fn read_geo_sites<T: AsRef<Path>>(
    geo_file_path: T,
) -> HashMap<String, Vec<DomainConfig>> {
    let mut result = HashMap::new();
    let data = tokio::fs::read(geo_file_path).await.unwrap();
    let list = GeoSiteListOwned::try_from(data).unwrap();

    for entry in list.proto().entry.iter() {
        let domains = entry.domain.iter().map(convert_domain_from_proto).collect();
        result.insert(entry.country_code.to_string(), domains);
    }
    result
}

pub fn convert_match_type_from_proto(value: Type) -> DomainMatchType {
    match value {
        Type::Plain => DomainMatchType::Plain,
        Type::Regex => DomainMatchType::Regex,
        Type::Domain => DomainMatchType::Domain,
        Type::Full => DomainMatchType::Full,
    }
}

pub fn convert_domain_from_proto(value: &Domain) -> DomainConfig {
    DomainConfig {
        match_type: convert_match_type_from_proto(value.type_pb),
        value: value.value.to_lowercase(),
    }
}

pub async fn read_geo_ips_from_bytes(
    contents: impl Into<Vec<u8>>,
) -> HashMap<String, Vec<IpConfig>> {
    let mut result = HashMap::new();
    let list = GeoIPListOwned::try_from(contents.into()).unwrap();

    for entry in list.proto().entry.iter() {
        let domains = entry.cidr.iter().filter_map(convert_ipconfig_from_proto).collect();
        result.insert(entry.country_code.to_string(), domains);
    }
    result
}

pub async fn read_geo_ips<T: AsRef<Path>>(geo_file_path: T) -> HashMap<String, Vec<IpConfig>> {
    let mut result = HashMap::new();
    let data = tokio::fs::read(geo_file_path).await.unwrap();
    let list = GeoIPListOwned::try_from(data).unwrap();

    for entry in list.proto().entry.iter() {
        let domains = entry.cidr.iter().filter_map(convert_ipconfig_from_proto).collect();
        result.insert(entry.country_code.to_string(), domains);
    }
    result
}

pub fn convert_ipconfig_from_proto(value: &crate::protos::geo::CIDR) -> Option<IpConfig> {
    let bytes = value.ip.as_ref();
    match bytes.len() {
        4 => {
            // IPv4 地址构造
            let ip = IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]));
            Some(IpConfig { ip, prefix: value.prefix })
        }
        // 16 => {
        //     // IPv6 地址构造
        //     let mut octets = [0u8; 16];
        //     octets.copy_from_slice(bytes);
        //     Some(IpAddr::V6(Ipv6Addr::from(octets)))
        // }
        _ => None, // 字节数不合法
    }
}

#[cfg(test)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(test)]
mod tests {

    use jemalloc_ctl::{epoch, stats};

    use crate::{protos::geo::GeoIPListOwned, read_geo_sites};

    fn test_memory_usage() {
        epoch::advance().unwrap();

        let allocated = stats::allocated::read().unwrap();
        let active = stats::active::read().unwrap();

        println!("Allocated memory: {} kbytes", allocated / 1024);
        println!("Active memory: {} kbytes", active / 1024);
    }

    #[tokio::test]
    async fn test() {
        test_memory_usage();
        let result = read_geo_sites("/root/.landscape-router/geosite.dat1").await;
        test_memory_usage();
        for (domain, domain_configs) in result {
            if domain == "test" {
                for domain_config in domain_configs {
                    println!("{domain:?}: {:?}", domain_config);
                }
            }
        }
        test_memory_usage();
    }

    #[tokio::test]
    async fn test_read() {
        test_memory_usage();
        let home_path = homedir::my_home().unwrap().unwrap().join(".landscape-router");
        let geo_file_path = home_path.join("geoip.dat");

        let data = tokio::fs::read(geo_file_path).await.unwrap();
        let list = GeoIPListOwned::try_from(data).unwrap();
        test_memory_usage();

        let mut sum = 0;
        for entry in list.proto().entry.iter() {
            // println!("{:?}", entry.country_code);
            if entry.country_code == "cn".to_uppercase() {
                println!("{:?}", entry.cidr.len());
            } else {
                sum += entry.cidr.len()
            }
            // println!("reverse_match : {:?}", entry.reverse_match);
            // if entry.reverse_match {
            //     println!("reverse_match : {:?}", entry.cidr);
            // }
        }
        println!("other count: {sum:?}");
        test_memory_usage();
    }
}
