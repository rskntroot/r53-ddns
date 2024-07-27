use aws_config::meta::region::RegionProviderChain;
use aws_sdk_route53 as route53;
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet,
};

use clap::Parser;

use std::net::Ipv4Addr;

#[derive(Parser)]
#[clap(
    name = "r53-ddns",
    about = "A CLI tool for correcting drift between your PublicIP and Route53 DNS A RECORD"
)]
struct Args {
    #[clap(short, long, help = "DNS ZONE ID\t(see AWS Console Route53)")]
    zone_id: String,

    #[clap(short, long, help = "DOMAIN NAME\t(ex. 'docs.rskio.com.')")]
    domain_name: String,
}

const RECORD_TYPE: &'static str = "A";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // get aws r53 client
    let region_provider = RegionProviderChain::default_provider();
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = route53::Client::new(&config);

    // get a list of resource_record_sets
    let list_resource_record_sets = client
        .list_resource_record_sets()
        .hosted_zone_id(&args.zone_id)
        .start_record_name(&args.domain_name)
        .send()
        .await?;

    // match a single resource record_set
    let mut resource_record_set: Option<ResourceRecordSet> = None;

    for rrs in list_resource_record_sets.resource_record_sets {
        if rrs.name.as_str() == &args.domain_name && rrs.r#type.as_str() == "A" {
            resource_record_set = Some(rrs);
            break;
        }
    }

    // if record is none: exit
    // else shadow resource_record_set with a safe unwrap
    let resource_record_set = match resource_record_set.is_none() {
        true => {
            println!(
                "No ResourceRecordSet found in Zone: {} for Record like: {} {}",
                &args.zone_id, RECORD_TYPE, &args.domain_name,
            );
            std::process::exit(1);
        }
        false => resource_record_set.unwrap(),
    };

    // if record contains empty resource_records, exit
    // else shadow resource_records with a safe unwrap
    let resource_records = match resource_record_set.resource_records.is_none() {
        true => {
            println!(
                "No ResourceRecord found Zone: {} for Record like: {} {}",
                &args.zone_id, RECORD_TYPE, &args.domain_name,
            );
            std::process::exit(1);
        }
        false => &resource_record_set.resource_records.unwrap(),
    };

    // get the first ip in the DNS record
    let record_ip = resource_records[0]
        .value
        .parse::<Ipv4Addr>()
        .expect("Failed to parse IP address");

    let public_ip: Ipv4Addr = get_public_ip().await?;

    // if no drift detected, exit
    if record_ip == public_ip {
        println!(
            "The DNS record is currently up to date with the public IP: {}",
            record_ip
        );
        return Ok(());
    }

    let msg: String = format!("The dynamic IP provided by the ISP has drifted.",);

    println!("{} {} -> {}", msg, record_ip, public_ip);

    // prepare aws r53 change request
    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(
            ResourceRecordSet::builder()
                .name(resource_record_set.name.clone())
                .r#type(resource_record_set.r#type.clone())
                .ttl(resource_record_set.ttl.unwrap())
                .resource_records(
                    ResourceRecord::builder()
                        .set_value(Some(public_ip.to_string()))
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        ) // Build the ResourceRecordSet
        .build()
        .unwrap(); // Build the Change

    // Change the resource record set
    let response = client
        .change_resource_record_sets()
        .hosted_zone_id(&args.zone_id)
        .change_batch(
            ChangeBatch::builder()
                .set_changes(Some(vec![change]))
                .set_comment(Some(msg))
                .build()
                .unwrap(),
        )
        .send()
        .await?;

    println!("Requested DNS record update to PublicIP: {}", public_ip);

    // Get the change ID from the response
    let change_id = response.change_info.unwrap().id;

    // Check the status of the change request every 60 seconds
    loop {
        let change_response = client
            .get_change()
            .id(&change_id)
            .send()
            .await?;

        // check the status
        if let Some(change_info) = change_response.change_info {
            println!("Change ID: {}, Status: {:?}", change_id, change_info.status);

            // break loop if the change is insync
            if change_info.status == route53::types::ChangeStatus::Insync {
                println!("The change request has been completed.");
                break;
            }
        }

        // sleep for 60 seconds before checking again...
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    Ok(())
}

async fn get_public_ip() -> Result<Ipv4Addr, Box<dyn std::error::Error>> {
    Ok(reqwest::get("http://ipv4.icanhazip.com")
        .await?
        .text()
        .await?
        .trim()
        .to_string()
        .parse::<Ipv4Addr>()?)
}

#[cfg(test)]
mod tests {
    use super::get_public_ip;

    #[tokio::test]
    async fn get_public_ip_works() {
        dbg!(get_public_ip().await.unwrap());
    }
}
