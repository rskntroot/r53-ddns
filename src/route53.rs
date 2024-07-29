use std::error::Error;
use std::fmt;
use std::net::IpAddr;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_route53 as r53;
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet,
};
use log::info;
use thiserror::Error;
use tokio::time::{sleep, Duration};

pub async fn update_record(
    dns_zone_id: &str,
    domain_name: &str,
    public_ip: IpAddr,
) -> Result<(), Box<dyn Error>> {
    let record_type: RecordType = match public_ip {
        IpAddr::V4(_) => RecordType::A,
        IpAddr::V6(_) => RecordType::AAAA,
    };
    let client: r53::Client = get_client().await?;
    let resource_record_set: Option<ResourceRecordSet> =
        get_single_record_set(&client, &dns_zone_id, &domain_name, &record_type).await?;

    match resource_record_set.is_none() {
        true => return Err(Box::new(Route53UpdateError::NoRecordAvailable)),
        false => {
            info!(
                "requesting update to route53 record for {} {} -> {}",
                record_type, domain_name, public_ip
            );
            return Ok(submit_single_change_request(
                &client,
                resource_record_set.unwrap(),
                &public_ip,
                &dns_zone_id,
            )
            .await?);
        }
    }
}

pub async fn get_client() -> Result<aws_sdk_route53::Client, Box<dyn Error>> {
    // get aws r53 client
    Ok(r53::Client::new(
        &aws_config::from_env()
            .region(RegionProviderChain::default_provider())
            .load()
            .await,
    ))
}

pub async fn get_single_record_set(
    client: &r53::Client,
    dns_zone_id: &str,
    domain_name: &str,
    record_type: &RecordType,
) -> Result<Option<ResourceRecordSet>, Box<dyn Error>> {
    // get a list of resource_record_sets
    let list_resource_record_sets = client
        .list_resource_record_sets()
        .hosted_zone_id(dns_zone_id.to_string())
        .start_record_name(domain_name.to_string())
        .send()
        .await?;

    // match a single resource record_set
    let mut resource_record_set: Option<ResourceRecordSet> = None;
    for rrs in list_resource_record_sets.resource_record_sets {
        if rrs.name.as_str() == domain_name && rrs.r#type.as_str() == record_type.to_string() {
            resource_record_set = Some(rrs);
            break;
        }
    }

    Ok(resource_record_set)
}

pub async fn submit_single_change_request(
    client: &r53::Client,
    resource_record_set: ResourceRecordSet,
    public_ip: &IpAddr,
    dns_zone_id: &str,
) -> Result<(), Box<dyn Error>> {
    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(
            ResourceRecordSet::builder()
                .name(resource_record_set.name.clone())
                .r#type(resource_record_set.r#type.clone())
                .ttl(resource_record_set.ttl.clone().unwrap())
                .resource_records(
                    ResourceRecord::builder()
                        .set_value(Some(public_ip.to_string()))
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    let msg: String = format!("ISP provided dynamic IP has drifted.");

    // submit batch change request of the resource record set
    let response = client
        .change_resource_record_sets()
        .hosted_zone_id(dns_zone_id)
        .change_batch(
            ChangeBatch::builder()
                .set_changes(Some(vec![change]))
                .set_comment(Some(msg))
                .build()
                .unwrap(),
        )
        .send()
        .await?;

    let change_id = response.change_info.unwrap().id;

    // check change request status every 60 seconds
    loop {
        let change_response = client.get_change().id(&change_id).send().await?;

        if let Some(change_info) = change_response.change_info {
            info!(
                "change_id: {} has status: {:?}",
                change_id, change_info.status
            );

            // break loop if the change is insync
            if change_info.status == r53::types::ChangeStatus::Insync {
                return Ok(());
            }
        }

        sleep(Duration::from_secs(180)).await;
    }
}

#[derive(Error, Debug)]
pub enum Route53UpdateError {
    #[error("zone does not contain requested name record")]
    NoRecordAvailable,
}

#[derive(Debug, PartialEq)]
pub enum RecordType {
    A,
    AAAA,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::AAAA => write!(f, "AAAA"),
        }
    }
}
