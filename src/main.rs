mod dns;
mod route53;

use clap::Parser;
use env_logger::Builder;
use log::info;
use reqwest::get;
use std::net::IpAddr;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[clap(
    name = "r53-ddns",
    about = "A CLI tool for correcting drift between your PublicIP and a Route53 DNS A|AAAA RECORD"
)]
struct Args {
    #[clap(short = 'z', long, help = "DNS ZONE ID\t(see AWS Console Route53)")]
    dns_zone_id: String,

    #[clap(short = 'd', long, help = "DOMAIN NAME\t(ex. 'docs.rskio.com.')")]
    domain_name: String,

    #[clap(
        short = 's',
        long,
        help = "SECONDS\trefresh timer in seconds",
        default_value = "180"
    )]
    seconds: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    Builder::new().filter(None, log::LevelFilter::Info).init();

    info!(
        "starting with options: -z {} -d {} -s {}",
        &args.dns_zone_id, &args.domain_name, &args.seconds,
    );

    let mut next_status_time = SystemTime::now();

    loop {
        let public_ip = get_public_ip().await?;

        // print the current public ip each week
        if SystemTime::now() > next_status_time {
            info!("current public address is: {}", public_ip);
            next_status_time += Duration::from_secs(7 * 24 * 60 * 60);
        }

        // update record if current public ip drifts from dns record
        if !dns::is_addr_current(&args.domain_name, public_ip).await? {
            route53::update_record(&args.dns_zone_id, &args.domain_name, public_ip).await?;
        }

        sleep(Duration::from_secs(args.seconds)).await;
    }
}

async fn get_public_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    Ok(get("http://icanhazip.com")
        .await?
        .text()
        .await?
        .trim()
        .to_string()
        .parse::<IpAddr>()?)
}

#[cfg(test)]
mod unit {
    #[tokio::test]
    async fn test_get_public_ip() {
        dbg!(super::get_public_ip().await.unwrap());
    }
}
