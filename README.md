# r53-ddns

[https://github.com/rskntroot/r53-ddns](https://github.com/rskntroot/r53-ddns)

Route53 Dynamic DNS

## Brief

Submits a Route53 `ChangeRequest` for updating `A` or `AAAA` records when PublicIP drift is detected.

Drift detection is determined by comparing http request to `icanhazip.com` and a DNS lookup to `cloudflare`.

This is intended to be installed on a public-facing loadbalancer.

## Assumptions

1. Your ISP randomly changes your PublicIP and that upsets you.
1. You just want something that will curl `ipv4.icanhazip.com`, check 3rd-party dns, and update Route53.
1. Your Name records only contain a single IP.

If so, this is for you.

## Setup
1. get
1. in [aws console](https://console.aws.amazon.com):
    - create IAM user
    - attach policy `aws.policy` file provided
    - generate access keys for automated service
1. log into aws cli with the account you created above
    ```
    aws configure
    ```
1. setup link in `/usr/bin`
    ``` zsh
    ln -sf ~/r53-ddns/target/release/r53-ddns /usr/bin/r53-ddns
    ```
1. get your hosted_zone_id
    ``` zsh
    aws route53 list-hosted-zones
    ```
1. setup systemd service and then install as normal
    ``` zsh
    DNS_ZONE_ID=YOURZONEIDHERE \
    DOMAIN_NAME=your.domain.com. \
    envsubst < r53-ddns.service | sudo tee /etc/systemd/system/r53-ddns.service
    ```

## Usage

```
$ r53-ddns -h
A CLI tool for correcting drift between your PublicIP and Route53 DNS A RECORD

Usage: r53-ddns --dns-zone-id <DNS_ZONE_ID> --domain-name <DOMAIN_NAME>

Options:
  -z, --dns-zone-id <DNS_ZONE_ID>  DNS ZONE ID  (see AWS Console Route53)
  -d, --domain-name <DOMAIN_NAME>  DOMAIN NAME  (ex. 'docs.rskio.com.')
  -s, --seconds <SECONDS>          SECONDS	refresh timer in seconds [default: 180]
  -h, --help                       Print help
```

### Service

``` zsh
export DNS_ZONE_ID=YOUR-DNS-ZONE-ID
export DOMAIN_NAME=YOUR-DOMAIN-NAME
export USER=$(whoami)
```

``` zsh
envsubst < r53-ddns.service | sudo tee -a /etc/systemd/system/r53-ddns.service
sudo systemctl daemon-reload
sudo systemctl start r53-ddns.service
sudo systemctl status r53-ddns.service
```

```
$ envsubst < r53-ddns.service | sudo tee /etc/systemd/system/r53-ddns.service
$ sudo systemctl enable --now r53-ddns.service
$ sudo systemctl status r53-ddns.service
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
Jul 29 09:03:40 hostname r53-ddns[215630]: [2024-07-29T09:03:40Z INFO  r53_ddns] current public address is: 10.0.0.1
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::dns] dynamic ip drift detected: 10.0.0.1 -> 71.211.88.219
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] requesting update to route53 record for A rskio.com. -> 71.211.88.219
Jul 29 09:09:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] change_id: /change/C02168177BNS6R50C32Q has status: Pending
Jul 29 09:10:41 hostname r53-ddns[215630]: [2024-07-29T09:09:41Z INFO  r53_ddns::route53] change_id: /change/C02168177BNS6R50C32Q has status: Insync
```

## FAQs

> Does this handle multiple record updates?

No. The goal here was for a single server to sync its dns record. If you are running multiple services from the same host, then consider using CNAMEs to point at a global A|AAAA record for this to update.

> What if I need to update only a single address in the record?

Let me know. I have been considering this use-case, but haven't implemented it yet.
