# r53-ddns

## Brief

this was intended to solve the problem for when your local ISP force renews your PublicIP and no one can reach your might-as-well-be-a-toaster-minecraft server.

what this ended up as was a mockery of dynamic dns, progam/scripting, error handling, all in memory-safe rust. whatever that means, right?

> "this is the worst of example working code ive ever seen"
> // "ah but you have seen the code working" -- cpt. rskntroot 2024

## Assumptions

1. Your ISP randomly changes your PublicIP and that pisses you off.
1. You have no idea how DDNS is actually supposed to work.
    - dns is all smoke and mirrors, confirmed.
1. You just want something that will curl `ipv4.icanhazip.com` and push and update to Route53.
1. You plan on handjamming this into a cron job on your webserver/loadbalancer.
1. ...
1. Profit.

Congratulations, this is the package for you.

## Setup

1. use out of below command to create AWS IAM Policy.
    ``` zsh
    zone_id=<zone_id> envsubst < AllowRoute53RecordUpdate.policy
    ```
1. create IAM user, generate access keys for automated service
1. login on the machine where you built this binary
    ```
    aws sso login --profile
    ```
1. setup a cron job to poll at your leisure

## Usage

```
$ r53-ddns -h
A CLI tool for correcting drift between your PublicIP and Route53 DNS A RECORD

Usage: r53-ddns --zone-id <ZONE_ID> --domain-name <DOMAIN_NAME>

Options:
  -z, --zone-id <ZONE_ID>          DNS ZONE ID  (see AWS Console Route53)
  -d, --domain-name <DOMAIN_NAME>  DOMAIN NAME  (ex. 'docs.rskio.com.')
  -h, --help                       Print help
```

### Drift Detected

``` zsh
$ r53-ddns -z ${aws_dns_zone_id} -d smp.rskio.com.
```

```
The dynamic IP provided by the ISP has drifted. 10.0.11.201 -> 10.0.88.219
Requested DNS record update to PublicIP: 10.0.88.219
Change ID: /change/C04224022UE1ZQA26RE7O, Status: Pending
Change ID: /change/C04224022UE1ZQA26RE7O, Status: Insync
The change request has been completed.
```

### No Drift Detected

``` zsh
$ r53-ddns -z  ${aws_dns_zone_id} -d example.com.
```

```
The DNS record is currently up to date with the public IP: 10.0.88.219
```

### Tests

Yeah, I have em! Well... one of them.

```
$ cargo test
   Compiling r53-ddns v0.1.0 (~/workspace/r53-ddns)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.81s
     Running unittests src/main.rs (target/debug/deps/r53_ddns-9ff92b89721daeea)

running 1 test
test tests::get_public_ip_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

## Q&A

> Why are you doing AWS calls instead of us nslookup and compare that?

Why in the world would internal DNS give me a PublicIP? Imagine not implementing internal DNS.

> Why did you do create this monster?

To prove to myself that with the help of LLMs that even I could go from 0 to deployed tokio async rust binary in less than 8 hours. And thats exactly what I did.

> wen IPv6?

Stfu, John. How about, wen PR?

