use clap::{App, AppSettings, Arg, SubCommand};

pub fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("kaws")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Deploys Kubernetes clusters using AWS, CoreOS, and Terraform")
        .after_help("Start by creating a new repository with the `init` command.\n")
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
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .after_help(
            "Creates the following files:\n\n\
            * clusters/CLUSTER/NAME-key.pem.encrypted: The KMS-encrypted private key\n\
            * clusters/CLUSTER/NAME.csr: The certificate signing request\n\n\
            Generated files are only valid for the specified cluster.\n"
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
                .help("The new administrator's name")
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
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .takes_value(true)
                .required(true)
                .help("The base domain name for the cluster, e.g. \"example.com\"")
        )
        .after_help(
            "The following files are expected by this command:\n\n\
            * clusters/CLUSTER/ca.pem: The CA certificate\n\
            * clusters/CLUSTER/NAME.pem: The client certificate\n\
            * clusters/CLUSTER/NAME-key.pem.encrypted: The KMS-encrypted private key\n"
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
        .arg(
            Arg::with_name("kms-key")
                .short("k")
                .long("kms-key")
                .takes_value(true)
                .required(true)
                .help("KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .after_help(
            "The following files are expected by this command:\n\n\
            * clusters/CLUSTER/ca.pem: The CA certificate\n\
            * clusters/CLUSTER/ca-key.pem.encrypted: The KMS-encrypted CA private key\n\
            * clusters/CLUSTER/NAME.csr: The requesting administrator's CSR\n"
        )
}

fn cluster<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("cluster")
        .about("Commands for managing a cluster's infrastructure")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(cluster_apply())
        .subcommand(cluster_destroy())
        .subcommand(cluster_init())
        .subcommand(cluster_plan())
        .subcommand(cluster_reencrypt())
}

fn cluster_apply<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("apply")
        .about("Applies the Terraform plan to the target cluster")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose plan should be applied")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
}

fn cluster_destroy<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("destroy")
        .about("Destroys resources defined by the Terraform plan for the target cluster")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster to destroy")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
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
            Arg::with_name("ami")
                .short("a")
                .long("ami")
                .takes_value(true)
                .required(true)
                .help("EC2 AMI ID to use for all CoreOS instances, e.g. \"ami-1234\"")
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
                .required(true)
                .help("Name of the SSH key in AWS for accessing EC2 instances, e.g. \"alice\"")
        )
        .arg(
            Arg::with_name("k8s-version")
                .short("v")
                .long("kubernetes-version")
                .takes_value(true)
                .required(true)
                .help("Version of Kubernetes to use, e.g. \"1.0.0\"")
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

fn cluster_plan<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("plan")
        .about("Displays the Terraform plan for the target cluster")
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
                .help("Path to AWS credentials file, defaults to \"~/.aws/credentials\"")
        )
        .arg(
            Arg::with_name("aws-credentials-profile")
                .long("aws-credentials-profile")
                .takes_value(true)
                .help("Name of the AWS credentials profile to use, defaults to \"default\"")
        )
}

fn cluster_reencrypt<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("reencrypt")
        .about("Re-encrypts the cluster's SSL keys using a new AWS KMS customer master key")
        .arg(
            Arg::with_name("cluster")
                .index(1)
                .required(true)
                .help("The cluster whose keys should be re-encrypted")
        )
        .arg(
            Arg::with_name("current-key")
                .long("current-key")
                .takes_value(true)
                .required(true)
                .help("Current KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
        .arg(
            Arg::with_name("new-key")
                .long("new-key")
                .takes_value(true)
                .required(true)
                .help("New KMS customer master key ID, e.g. \"12345678-1234-1234-1234-123456789012\"")
        )
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
