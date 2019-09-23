//! The `cargo audit` subcommand

use super::CargoAuditCommand;
use crate::{
    auditor::Auditor,
    config::{AuditConfig, OutputFormat},
    prelude::*,
};
use abscissa_core::{config::Override, FrameworkError};
use gumdrop::Options;
use rustsec::platforms::target::{Arch, OS};
use std::{path::PathBuf, process::exit};

/// Name of `Cargo.lock`
const CARGO_LOCK_FILE: &str = "Cargo.lock";

/// The `cargo audit` subcommand
#[derive(Command, Default, Debug, Options)]
pub struct AuditCommand {
    /// Get help information
    #[options(short = "h", long = "help", help = "output help information and exit")]
    help: bool,

    /// Get version information
    #[options(no_short, long = "version", help = "output version and exit")]
    version: bool,

    /// Colored output configuration
    #[options(
        short = "c",
        long = "color",
        help = "color configuration: always, never (default: auto)"
    )]
    color: Option<String>,

    /// Filesystem path to the advisory database git repository
    #[options(
        short = "D",
        long = "db",
        help = "advisory database git repo path (default: ~/.cargo/advisory-db)"
    )]
    db: Option<String>,

    /// Path to the lockfile
    #[options(
        short = "f",
        long = "file",
        help = "Cargo lockfile to inspect (or `-` for STDIN, default: Cargo.lock)"
    )]
    file: Option<String>,

    /// Advisory ids to ignore
    #[options(
        no_short,
        long = "ignore",
        meta = "ADVISORY_ID",
        help = "Advisory id to ignore (can be specified multiple times)"
    )]
    ignore: Vec<String>,

    /// Skip fetching the advisory database git repository
    #[options(
        short = "n",
        long = "no-fetch",
        help = "do not perform a git fetch on the advisory DB"
    )]
    no_fetch: bool,

    /// Allow stale advisory databases that haven't been recently updated
    #[options(no_short, long = "stale", help = "allow stale database")]
    stale: bool,

    /// Target CPU architecture to find vulnerabilities for
    #[options(
        no_short,
        long = "target-arch",
        help = "filter vulnerabilities by CPU (default: no filter)"
    )]
    target_arch: Option<Arch>,

    /// Target OS to find vulnerabilities for
    #[options(
        no_short,
        long = "target-os",
        help = "filter vulnerabilities by OS (default: no filter)"
    )]
    target_os: Option<OS>,

    /// URL to the advisory database git repository
    #[options(short = "u", long = "url", help = "URL for advisory database git repo")]
    url: Option<String>,

    /// Quiet mode - avoids printing extraneous information
    #[options(
        short = "q",
        long = "quiet",
        help = "Avoid printing unnecessary information"
    )]
    quiet: bool,

    /// Output reports as JSON
    #[options(no_short, long = "json", help = "Output report in JSON format")]
    output_json: bool,
}

impl Override<AuditConfig> for AuditCommand {
    fn override_config(&self, mut config: AuditConfig) -> Result<AuditConfig, FrameworkError> {
        if let Some(color) = &self.color {
            config.color = Some(color.clone());
        }

        if let Some(db) = &self.db {
            config.advisory_db_path = Some(db.into());
        }

        for advisory_id in &self.ignore {
            // TODO(tarcieri): handle/ignore duplicate advisory IDs between config and CLI opts
            config.ignore.push(advisory_id.parse().unwrap_or_else(|e| {
                status_err!("error parsing {}: {}", advisory_id, e);
                exit(1);
            }));
        }

        config.no_fetch |= self.no_fetch;
        config.allow_stale |= self.stale;

        if let Some(target_arch) = self.target_arch {
            config.target_arch = Some(target_arch);
        }

        if let Some(target_os) = self.target_os {
            config.target_os = Some(target_os);
        }

        if let Some(url) = &self.url {
            config.advisory_db_url = Some(url.clone())
        }

        config.quiet |= self.quiet;

        if self.output_json {
            config.output_format = OutputFormat::Json;
        }

        Ok(config)
    }
}

impl Runnable for AuditCommand {
    fn run(&self) {
        if self.help {
            Self::print_usage_and_exit(&[]);
        }

        if self.version {
            println!("cargo-audit {}", CargoAuditCommand::version());
            exit(0);
        }

        let lockfile_path = self
            .file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(CARGO_LOCK_FILE));

        self.auditor().audit(&lockfile_path);
    }
}

impl AuditCommand {
    /// Initialize `Auditor`
    pub fn auditor(&self) -> Auditor {
        let config = app_config();
        Auditor::new(&config)
    }
}
