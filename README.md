# r53-ddns

Route53 Dynamic DNS

## Brief

Submits a Route53 `ChangeRequest` for updating `A` or `AAAA` records when PublicIP drift is detected.

Drift detection is determined by comparing http request to `icanhazip.com` and a DNS lookup to `cloudflare`.

This is intended to be installed on a public-facing loadbalancer.

## Assumptions

1. Your ISP randomly changes your PublicIP and that pisses you off.
1. You just want something that will curl `ipv4.icanhazip.com`, check 3rd-party dns, and update Route53.
1. Your Name records only contain a single IP. (future update maybe).

If so, this is for you.

## Setup

1. setup `Route53AllowRecordUpdate.policy`
    ```zsh
    DNS_ZONE_ID=YOURZONEIDHERE \
    envsubst < aws.policy > Route53AllowRecordUpdate.policy
    ```
1. create IAM user, generate access keys for automated service
1. log into aws with the acct on the machine where you install this binary
    ```
    aws sso login --profile
    ```
1. setup link in `/usr/bin`
    ``` zsh
    ln -s -f ~/r53-ddns/target/release/r53-ddns /usr/bin/r53-ddns
    ```
1. setup systemd service and then install as normal
    ```zsh
    DNS_ZONE_ID=YOURZONEIDHERE \
    DOMAIN_NAME=your.domain.com. \
    envsubst < r53-ddns.service | sudo  /etc/system/
    ```
## CLI Usage

```
$ r53-ddns -h
A CLI tool for correcting drift between your PublicIP and Route53 DNS A RECORD

Usage: r53-ddns --dns-zone-id <DNS_ZONE_ID> --domain-name <DOMAIN_NAME>

Options:
  -z, --dns-zone-id <DNS_ZONE_ID>  DNS ZONE ID  (see AWS Console Route53)
  -d, --domain-name <DOMAIN_NAME>  DOMAIN NAME  (ex. 'docs.rskio.com.')
  -h, --help                       Print help
```

### Example

```
$ systemctl status r53-ddns.service
● r53-ddns.service - Route53 Dynamic DNS Service
     Loaded: loaded (/etc/systemd/system/r53-ddns.service; enabled; vendor preset: enabled)
     Active: active (running) since Mon 2024-07-29 09:03:40 UTC; 7min ago
   Main PID: 215630 (r53-ddns)
      Tasks: 6 (limit: 18886)
     Memory: 3.6M
        CPU: 389ms
     CGroup: /system.slice/r53-ddns.service
             └─215630 /usr/bin/r53-ddns -z [##TRUNCATED##] -d rskio.com.

Jul 29 09:03:40 hostname systemd[1]: Started Route53 Dynamic DNS Service.
Jul 29 09:03:40 hostname r53-ddns[215630]: [2024-07-29T09:03:40Z INFO  r53_ddns] starting with options: -z [##TRUNCATED##] -d rskio.com.
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::dns] dynamic ip drift detected: 10.0.0.1 -> 71.211.88.219
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] requesting update to route53 record for A rskio.com. -> 71.211.88.219
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] change_id: /change/C02168177BNS6R50C32Q has status: Pending
Jul 29 09:10:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] change_id: /change/C02168177BNS6R50C32Q has status: Insync
```

## Q&A

> Why did you do create this monster in rust?

To be able to handle errors in the future.

> wen IPv6?

It should work with IPv6.

