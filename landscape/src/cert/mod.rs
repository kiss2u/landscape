use std::path::PathBuf;

use landscape_common::{TLS_DEFAULT_CERT, TLS_DEFAULT_KEY};
use pem::parse_many;
use rcgen::generate_simple_self_signed;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::fs;

/// 尝试加载本地证书，如果不存在则生成新的
/// TODO: Maybe create an ACME service
pub async fn load_or_generate_cert(home_path: PathBuf) -> ServerConfig {
    let cert_path = home_path.join(TLS_DEFAULT_CERT);
    let key_path = home_path.join(TLS_DEFAULT_KEY);

    if cert_path.is_dir() {
        fs::remove_dir_all(&cert_path).await.unwrap();
    }
    if key_path.is_dir() {
        fs::remove_dir_all(&key_path).await.unwrap();
    }

    // 如果文件不存在，生成新的证书和私钥
    if !cert_path.exists() || !key_path.exists() {
        tracing::info!("default cert is not exists, gen a new cert");
        let subject_alt_names = vec!["localhost".to_string()];
        let rcgen::CertifiedKey { cert, signing_key } =
            generate_simple_self_signed(subject_alt_names).unwrap();

        let cert_pem = cert.pem();
        let key_pem = signing_key.serialize_pem();

        fs::write(&cert_path, cert_pem).await.unwrap();
        fs::write(&key_path, key_pem).await.unwrap();
    }

    // 加载证书和私钥
    let cert_pem = fs::read(&cert_path).await.expect("read cert");
    let key_pem = fs::read(&key_path).await.expect("read key");

    // 解析 PEM 格式的证书
    let cert_pems = parse_many(&cert_pem).expect("failed to parse cert PEM");
    let certs: Vec<CertificateDer> = cert_pems
        .into_iter()
        .filter(|p| p.tag() == "CERTIFICATE")
        .map(|p| CertificateDer::from(p.contents().to_vec()))
        .collect();

    if certs.is_empty() {
        panic!("No valid certificate found");
    }

    // 解析 PEM 格式的私钥
    let key_pems = parse_many(&key_pem).expect("failed to parse key PEM");
    let private_key = key_pems
        .into_iter()
        .find_map(|p| match p.tag() {
            "PRIVATE KEY" => Some(PrivateKeyDer::Pkcs8(p.contents().to_vec().into())),
            "RSA PRIVATE KEY" => Some(PrivateKeyDer::Pkcs1(p.contents().to_vec().into())),
            _ => None,
        })
        .expect("No valid private key found");

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .expect("invalid TLS config");

    config
}
