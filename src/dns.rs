use std::error::Error;
use std::net::IpAddr;

use log::info;
use trust_dns_proto::rr::record_type::RecordType;
use trust_dns_proto::rr::RecordData;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

pub async fn is_addr_current(domain: &str, ip_addr: IpAddr) -> Result<bool, Box<dyn Error>> {
    let response = TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default())
        .lookup(
            domain,
            match ip_addr {
                IpAddr::V4(_) => RecordType::A,
                IpAddr::V6(_) => RecordType::AAAA,
            },
        )
        .await?;

    let mut record_ip: Option<IpAddr> = None;
    for record in response.into_iter() {
        record_ip = record.into_rdata().ip_addr();
        if !record_ip.is_none() && record_ip == Some(ip_addr) {
            return Ok(true);
        }
    }

    info!(
        "dynamic ip drift detected: {} -> {}",
        record_ip.unwrap(),
        ip_addr
    );

    Ok(false)
}

#[cfg(test)]
mod unit {
    use std::net::IpAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_is_addr_current() {
        let domain = "rskio.com";
        let ip_addr = IpAddr::from_str("0.0.0.0").unwrap();
        assert_eq!(
            super::is_addr_current(domain, ip_addr).await.unwrap(),
            false
        );
    }
}
