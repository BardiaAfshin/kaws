use std::fs::create_dir_all;

use clap::ArgMatches;
use rusoto::ChainProvider;

use aws::credentials_provider;
use encryption::Encryptor;
use error::KawsResult;
use process::execute_child_process;

pub struct Admin<'a> {
    aws_credentials_provider: ChainProvider,
    cluster: &'a str,
    domain: Option<&'a str>,
    kms_master_key_id: Option<&'a str>,
    name: Option<&'a str>,
    region: Option<&'a str>,
}

impl<'a> Admin<'a> {
    pub fn new(matches: &'a ArgMatches) -> Self {
        Admin {
            aws_credentials_provider: credentials_provider(
                matches.value_of("aws-credentials-path"),
                matches.value_of("aws-credentials-profile"),
            ),
            cluster: matches.value_of("cluster").expect("clap should have required cluster"),
            domain: matches.value_of("domain"),
            kms_master_key_id: matches.value_of("kms-key"),
            name: matches.value_of("name"),
            region: matches.value_of("region"),
        }
    }

    pub fn create(&mut self) -> KawsResult {
        let name = self.name.expect("clap should have required name");

        let admin_key_path = format!(
            "clusters/{}/{}-key.pem",
            self.cluster,
            name,
        );

        let admin_csr_path = format!(
            "clusters/{}/{}.csr",
            self.cluster,
            name,
        );

        log_wrap!("Creating directory for the new administrator's credentials", {
            try!(create_dir_all(format!("clusters/{}", self.cluster)));
        });

        // create private key
        log_wrap!("Creating Kubernetes admin private key", {
            try!(execute_child_process("openssl", &[
                "genrsa",
                "-out",
                &admin_key_path,
                "2048",
            ]));
        });

        // create CSR
        log_wrap!("Creating Kubernetes admin certificate signing request", {
            try!(execute_child_process("openssl", &[
                "req",
                "-new",
                "-key",
                &admin_key_path,
                "-out",
                &admin_csr_path,
                "-subj",
                &format!("/CN={}-{}", name, self.cluster),
            ]));
        });

        Ok(Some(format!(
            "Certificate signing request created! Commit changes to Git and ask an\n\
            administrator to generate your client certificate."
        )))
    }

    pub fn install(&mut self) -> KawsResult {
        let domain = self.domain.expect("clap should have required domain");
        let name = self.name.expect("clap should have required name");

        log_wrap!("Configuring kubectl", {
            // set cluster
            try!(execute_child_process("kubectl", &[
                "config",
                "set-cluster",
                self.cluster,
                &format!("--server=https://kubernetes.{}", &domain),
                &format!("--certificate-authority=clusters/{}/ca.pem", self.cluster),
                "--embed-certs=true",
            ]));

            // set credentials
            try!(execute_child_process("kubectl", &[
                "config",
                "set-credentials",
                &format!("{}-{}", name, self.cluster),
                &format!("--client-certificate=clusters/{}/{}.pem", self.cluster, name),
                &format!("--client-key=clusters/{}/{}-key.pem", self.cluster, name),
                "--embed-certs=true",
            ]));

            // set context
            try!(execute_child_process("kubectl", &[
                "config",
                "set-context",
                self.cluster,
                &format!("--cluster={}", self.cluster),
                &format!("--user={}-{}", name, self.cluster),
            ]));
        });

        Ok(Some(format!(
            "Admin credentials for user \"{}\" installed for cluster \"{}\"!\n\
            To activate these settings as the current context, run:\n\n\
            kubectl config use-context {}\n\n\
            If the kubectl configuration file is ever removed or changed accidentally,\n\
            just run this command again to regenerate or reconfigure it.",
            name,
            self.cluster,
            self.cluster,
        )))
    }

    pub fn sign(&mut self) -> KawsResult {
        let name = self.name.expect("clap should have required name");
        let region = self.region.expect("clap should have required region");
        let kms_master_key_id = self.kms_master_key_id.expect("clap should have required kms key");

        let admin_csr_path = format!("clusters/{}/{}.csr", self.cluster, name);
        let admin_cert_path = format!("clusters/{}/{}.pem", self.cluster, name);
        let ca_cert_path = format!("clusters/{}/ca.pem", self.cluster);
        let ca_key_path = format!("clusters/{}/ca-key.pem", self.cluster);
        let encrypted_ca_key_path = format!("clusters/{}/ca-key-encrypted.base64", self.cluster);

        let mut encryptor = Encryptor::new(
            self.aws_credentials_provider.clone(),
            try!(region.parse()),
            kms_master_key_id,
        );

        // decrypt CA key
        try!(encryptor.decrypt_file(&encrypted_ca_key_path, &ca_key_path));

        // generate admin cert
        log_wrap!("Creating Kubernetes admin certificate", {
            try!(execute_child_process("openssl", &[
                "x509",
                "-req",
                "-in",
                &admin_csr_path,
                "-CA",
                &ca_cert_path,
                "-CAkey",
                &ca_key_path,
                "-CAcreateserial",
                "-out",
                &admin_cert_path,
                "-days",
                "365",
            ]));
        });

        Ok(Some(format!(
            "Client certificate for administrator \"{}\" created for cluster \"{}\"!\n\
            Commit changes to Git and ask the administrator to run `kaws admin install`.",
            name,
            self.cluster,
        )))
    }
}
