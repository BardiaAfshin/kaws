use std::cmp::Ordering;

use bitstring::BitString;
use cidr::Ipv4Cidr;
use clap::{App, AppSettings, Arg, SubCommand};

pub fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("kaws")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Deploys Kubernetes clusters using AWS, CoreOS, and Terraform")
        .after_help("\nStart by creating a new repository with the `init` command.")
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(admin())
        .subcommand(cluster())
        .subcommand(init())
}

fn admin<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("admin")
        .about("Commands for managing cluster administrators")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(admin_create())
        .subcommand(admin_install())
        .subcommand(admin_sign())
}

fn admin_create<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .about("Generates a private key and certificate signing request for a new administrator")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster the new administrator should be able to access")
        )
        .arg(
            Arg::with_name("name")
                .index(2)
                .required(true)
                .help("The new administrator's name")
        )
        .arg(
            Arg::with_name("group")
                .short("g")
                .long("group")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
            .help("A Kubernetes groups this user belongs to; this option can be specified more than once")
        )
        .after_help(
            "\nCreates the following files:\n\n\
            * clusters/CLUSTER/NAME-key.pem: The admin's unencrypted private key\n\
            * clusters/CLUSTER/NAME-csr.pem: The admin's certificate signing request\n\n\
            Generated files are only valid for the specified cluster. The private key should not be checked into Git."
        )
}

fn admin_install<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("install")
        .about("Configures kubectl for a new cluster and administrator")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to configure")
        )
        .arg(
            Arg::with_name("name")
                .index(2)
                .required(true)
                .help("The name of the administrator whose credentials are being installed")
        )
        .after_help(
            "\nThe following files are expected by this command:\n\n\
            * clusters/CLUSTER/k8s-ca.pem: The k8s CA certificate\n\
            * clusters/CLUSTER/NAME.pem: The admin's client certificate\n\
            * clusters/CLUSTER/NAME-key.pem: The admin's unencrypted private key"
        )
}

fn admin_sign<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sign")
        .about("Signs an administrator's certificate signing request, creating a new client certificate")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The name of the cluster the certificate will be valid for")
        )
        .arg(
            Arg::with_name("name")
                .index(2)
                .required(true)
                .help("The new administrator's name")
        )
        .after_help(
            "\nThe following files are expected by this command:\n\n\
            * clusters/CLUSTER/k8s-ca.pem: The CA certificate\n\
            * clusters/CLUSTER/k8s-ca-key-encrypted.base64: The KMS-encrypted CA private key\n\
            * clusters/CLUSTER/NAME-csr.pem: The requesting administrator's CSR"
        )
}

fn cluster<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("cluster")
        .about("Commands for managing a cluster's infrastructure")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(cluster_apply())
        .subcommand(cluster_destroy())
        .subcommand(cluster_generate_pki())
        .subcommand(cluster_init())
        .subcommand(cluster_output())
        .subcommand(cluster_plan())
        .subcommand(cluster_refresh())
}

fn cluster_apply<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("apply")
        .about("Applies the Terraform plan to the target cluster")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose plan should be applied")
        )
        .arg(
            Arg::with_name("aws-credentials-path")
                .long("aws-credentials-path")
                .takes_value(true)
                .help("Path to the AWS credentials file, defaults to ~/.aws/credentials")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
        .arg(
            Arg::with_name("terraform-args")
                .index(2)
                .multiple(true)
                .hidden(true)
                .help("Additional arguments to be passed on to `terraform apply`")
        )
        .after_help("\nAny arguments following a literal -- will be passed directly as options to `terraform apply`.")
}

fn cluster_destroy<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("destroy")
        .about("Destroys resources defined by the Terraform plan for the target cluster")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to destroy")
        )
        .arg(
            Arg::with_name("aws-credentials-path")
                .long("aws-credentials-path")
                .takes_value(true)
                .help("Path to the AWS credentials file, defaults to ~/.aws/credentials")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
        .arg(
            Arg::with_name("terraform-args")
                .index(2)
                .multiple(true)
                .hidden(true)
                .help("Additional arguments to be passed on to `terraform destroy`")
        )
        .after_help("\nAny arguments following a literal -- will be passed directly as options to `terraform destroy`.")
}

fn cluster_init<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Initializes all the configuration files for a new cluster")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The name of the cluster to create, e.g. \"production\"")
        )
        .arg(
            Arg::with_name("aws-account-id")
                .short("A")
                .long("aws-account-id")
                .takes_value(true)
                .required(true)
                .help("The numeric ID of the AWS account, e.g. \"123456789012\"")
        )
        .arg(
            Arg::with_name("ami")
                .short("a")
                .long("ami")
                .takes_value(true)
                .required(true)
                .help("EC2 AMI ID to use for all CoreOS instances, e.g. \"ami-1234\"")
        )
        .arg(
            Arg::with_name("availability-zone")
                .long("availability-zone")
                .takes_value(true)
                .required(true)
                .help("Availability Zone for etcd instances and EBS volumes, e.g. \"us-east-1a\"")
        )
        .arg(
            Arg::with_name("cidr")
                .short("C")
                .long("cidr")
                .takes_value(true)
                .required(true)
                .help("IPv4 network range of the subnet where Kubernetes nodes will run, e.g. \"10.0.2.0/24\"")
                .validator(|cidr| {
                    let cidr: Ipv4Cidr = match cidr.parse() {
                        Ok(cidr) => cidr,
                        Err(_) => return Err("Invalid CIDR provided.".to_string()),
                    };

                    let vpc_cidr: Ipv4Cidr = "10.0.0.0/16".parse().unwrap();
                    let elb_cidr: Ipv4Cidr = "10.0.0.0/24".parse().unwrap();
                    let etcd_cidr: Ipv4Cidr = "10.0.1.0/24".parse().unwrap();

                    match cidr.subset_cmp(&vpc_cidr) {
                        Some(Ordering::Less) => {}
                        _ => return Err("Provided CIDR must be a subset of 10.0.0.0/16.".to_string()),
                    }

                    match cidr.subset_cmp(&elb_cidr) {
                        Some(_) => return Err("Provided CIDR cannot overlap with 10.0.0.0/24, which is used for ELBs.".to_string()),
                        None => {}
                    }

                    match cidr.subset_cmp(&etcd_cidr) {
                        Some(_) => return Err("Provided CIDR cannot overlap with 10.0.1.0/24, which is used for etcd.".to_string()),
                        None => {}
                    }

                    Ok(())
                })
        )
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .takes_value(true)
                .required(true)
                .help("The base domain name for the cluster, e.g. \"example.com\"")
        )
        .arg(
            Arg::with_name("masters-max-size")
                .long("masters-max-size")
                .takes_value(true)
                .required(true)
                .help(
                    "The maximum number of EC2 instances the Kubernetes masters may autoscale to"
                )
        )
        .arg(
            Arg::with_name("masters-min-size")
                .long("masters-min-size")
                .takes_value(true)
                .required(true)
                .help(
                    "The minimum number of EC2 instances the Kubernetes masters may autoscale to"
                )
        )
        .arg(
            Arg::with_name("nodes-max-size")
                .long("nodes-max-size")
                .takes_value(true)
                .required(true)
                .help(
                    "The maximum number of EC2 instances the Kubernetes nodes may autoscale to"
                )
        )
        .arg(
            Arg::with_name("nodes-min-size")
                .long("nodes-min-size")
                .takes_value(true)
                .required(true)
                .help(
                    "The minimum number of EC2 instances the Kubernetes nodes may autoscale to"
                )
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .takes_value(true)
                .required(true)
                .help("AWS Region to create the resources in, e.g. \"us-east-1\"")
        )
        .arg(
            Arg::with_name("iam-user")
                .short("i")
                .long("iam-user")
                .takes_value(true)
                .multiple(true)
                .required(true)
                .number_of_values(1)
                .help("An IAM user name who will have access to cluster PKI secrets, e.g. \"alice\"; this option can be specified more than once")
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("instance-size")
                .takes_value(true)
                .required(true)
                .help("EC2 instance size to use for all instances, e.g. \"m3.medium\"")
        )
        .arg(
            Arg::with_name("ssh-key")
                .short("K")
                .long("ssh-key")
                .takes_value(true)
                .multiple(true)
                .required(true)
                .number_of_values(1)
                .help("SSH public key to add to ~/.ssh/authorized_keys on each server; this option can be specified more than once")
        )
        .arg(
            Arg::with_name("k8s-version")
                .short("v")
                .long("kubernetes-version")
                .takes_value(true)
                .required(true)
                .help("Version of Kubernetes to use, e.g. \"1.0.0\"")
                .validator(|version| {
                    let version = version.as_str();

                    if version.starts_with('v') {
                        return Err("Kubernetes version should be specified without the leading 'v'".to_string());
                    }

                    if version >= "1.7" {
                        return Ok(());
                    } else {
                        return Err("This version of kaws supports only Kubernetes 1.7.0 or greater".to_string());
                    }
                })
        )
        .arg(
            Arg::with_name("zone-id")
                .short("z")
                .long("zone-id")
                .takes_value(true)
                .required(true)
                .help("Route 53 hosted zone ID")
        )
}

fn cluster_generate_pki<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("generate-pki")
        .about("Generates public key infrastructure for a cluster")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(cluster_generate_pki_all())
        .subcommand(cluster_generate_pki_etcd())
        .subcommand(cluster_generate_pki_etcd_peer())
        .subcommand(cluster_generate_pki_kubernetes())
}

fn cluster_generate_pki_all<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("all")
        .about("Generates all necessary public key infrastructure for a new cluster")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to generate PKI assets for")
        )
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .takes_value(true)
                .required(true)
                .help("The base domain name for the cluster, e.g. \"example.com\"")
        )
        .arg(
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .takes_value(true)
                .required(true)
                .help("AWS Region where the KMS key lives, e.g. \"us-east-1\"")
        )
}

fn cluster_generate_pki_etcd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("etcd")
        .about("Generates public key infrastructure for etcd's client API")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to generate PKI assets for")
        )
        .arg(
            Arg::with_name("subject")
                .index(2)
                .required(true)
                .possible_values(&["ca", "client", "server"])
                .help("The subject to generate PKI assets for")
        )
        .arg(
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .takes_value(true)
                .required(true)
                .help("AWS Region where the KMS key lives, e.g. \"us-east-1\"")
        )
}

fn cluster_generate_pki_etcd_peer<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("etcd-peer")
        .about("Generates public key infrastructure for etcd's peer API")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to generate PKI assets for")
        )
        .arg(
            Arg::with_name("subject")
                .index(2)
                .required(true)
                .possible_values(&["ca", "peer"])
                .help("The subject to generate PKI assets for")
        )
        .arg(
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .takes_value(true)
                .required(true)
                .help("AWS Region where the KMS key lives, e.g. \"us-east-1\"")
        )
}

fn cluster_generate_pki_kubernetes<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("kubernetes")
        .about("Generates public key infrastructure for Kubernetes")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to generate PKI assets for")
        )
        .arg(
            Arg::with_name("subject")
                .index(2)
                .required(true)
                .possible_values(&["ca", "masters", "nodes"])
                .help("The subject to generate PKI assets for")
        )
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .takes_value(true)
                .required(true)
                .help("The base domain name for the cluster, e.g. \"example.com\"")
        )
        .arg(
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .takes_value(true)
                .required(true)
                .help("AWS Region where the KMS key lives, e.g. \"us-east-1\"")
        )
}

fn cluster_output<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("output")
        .about("Displays the Terraform outputs for the target cluster")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose plan should be displayed")
        )
        .arg(
            Arg::with_name("output")
                .index(2)
                .help("The name of an individual output to display")
        )
}

fn cluster_plan<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("plan")
        .about("Displays the Terraform plan for the target cluster")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose plan should be displayed")
        )
        .arg(
            Arg::with_name("aws-credentials-path")
                .long("aws-credentials-path")
                .takes_value(true)
                .help("Path to the AWS credentials file, defaults to ~/.aws/credentials")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
        .arg(
            Arg::with_name("terraform-args")
                .index(2)
                .multiple(true)
                .hidden(true)
                .help("Additional arguments to be passed on to `terraform plan`")
        )
        .after_help("\nAny arguments following a literal -- will be passed directly as options to `terraform plan`.")
}

fn cluster_refresh<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("refresh")
        .about("Refreshes the Terraform state for the target cluster")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose plan should be displayed")
        )
        .arg(
            Arg::with_name("aws-credentials-path")
                .long("aws-credentials-path")
                .takes_value(true)
                .help("Path to the AWS credentials file, defaults to ~/.aws/credentials")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
        .arg(
            Arg::with_name("terraform-args")
                .index(2)
                .multiple(true)
                .hidden(true)
                .help("Additional arguments to be passed on to `terraform refresh`")
        )
        .after_help("\nAny arguments following a literal -- will be passed directly as options to `terraform refresh`.")
}

fn init<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Initializes a new repository for managing Kubernetes clusters")
        .arg(
            Arg::with_name("name")
                .index(1)
                .required(true)
                .help("The name of the repository to create, e.g. \"example-company-infrastructure\"")
        )
        .arg(
            Arg::with_name("terraform-source")
                .short("t")
                .long("terraform-source")
                .takes_value(true)
                .help("Custom source value for the Terraform module to use")
        )
}
