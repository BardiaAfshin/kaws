extern crate ansi_term;
extern crate env_logger;
extern crate clap;
#[macro_use]
extern crate log;
extern crate hyper;
extern crate rusoto;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tempdir;

macro_rules! log_wrap {
    ($m:expr, $b:block) => {
        debug!("{}...", $m);
        $b
        debug!("...done.");
    }
}

mod admin;
mod aws;
mod cli;
mod cluster;
mod dependencies;
mod encryption;
mod error;
mod pki;
mod process;
mod repository;
mod terraform;

use std::process::exit;

use ansi_term::Colour::{Green, Red};

use admin::Admin;
use cluster::{ExistingCluster, NewCluster};
use dependencies::ensure_dependencies;
use error::KawsResult;
use repository::Repository;
use terraform::Terraform;

fn main() {
    env_logger::init().expect("Failed to initialize logger.");

    let mut failed = false;

    match execute_cli() {
        Ok(success) => {
            if let Some(message) = success {
                println!("{}", Green.paint(message.to_string()));
            }
        },
        Err(error) => {
            let error_output = format!("Error:\n{}", error);

            println!("{}", Red.paint(error_output));

            failed = true;
        },
    }

    if failed {
        exit(1);
    }
}

fn execute_cli() -> KawsResult {
    let app_matches = cli::app().get_matches();

    match app_matches.subcommand() {
        ("admin", Some(admin_matches)) => {
            ensure_dependencies()?;

            match admin_matches.subcommand() {
                ("create", Some(matches)) => Admin::new(matches).create(),
                ("install", Some(matches)) => Admin::new(matches).install(),
                ("sign", Some(matches)) => Admin::new(matches).sign(),
                _ => {
                    println!("{}", admin_matches.usage());

                    Ok(None)
                }
            }
        },
        ("cluster", Some(cluster_matches)) => {
            ensure_dependencies()?;

            match cluster_matches.subcommand() {
                ("apply", Some(matches)) => Terraform::new(matches).apply(),
                ("destroy", Some(matches)) => Terraform::new(matches).destroy(),
                ("init", Some(matches)) => NewCluster::new(matches).init(),
                ("genpki", Some(matches)) => ExistingCluster::new(matches).generate_pki(),
                ("output", Some(matches)) => Terraform::new(matches).output(),
                ("plan", Some(matches)) => Terraform::new(matches).plan(),
                ("refresh", Some(matches)) => Terraform::new(matches).refresh(),
                _ => {
                    println!("{}", cluster_matches.usage());

                    Ok(None)
                }
            }
        },
        ("init", Some(matches)) => {
            ensure_dependencies()?;

            Repository::new(matches).create()
        }
        _ => {
            println!("{}", app_matches.usage());

            Ok(None)
        },
    }
}
