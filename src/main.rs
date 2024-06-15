mod constants;
mod errors;
mod receiver;
mod sender;
mod server;

use std::{io, path::PathBuf};

use clap::{Parser, Subcommand};
use iris::{CipherType, ConflictingFileMode};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

/// A file transfer utility that allows the transfer of data between two machines behind NAT
/// using a relay server.
#[derive(Debug, Parser)]
#[clap(about, about, long_about=None)]
struct App {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Runs the server component that acts as a relay between two clients wanting to transfer
    /// data between them.
    #[command(name("serve"))]
    Server {
        /// The ip address where the server will listen at
        #[arg(long, default_value_t=String::from("0.0.0.0"))]
        ip_address: String,
        /// The port where the server will listen at
        #[arg(long, default_value_t=String::from("10000"))]
        port: String,
    },
    /// Send the file(s) to the server which would relay them to the recipient.
    #[command(name("send"))]
    Sender {
        /// The ip address where the server is listening at
        #[arg(long)]
        server_ip: String,
        /// The port where the server is listening at
        #[arg(long)]
        server_port: String,
        /// One or more files to send to the recipient
        #[arg(required(true))]
        files: Vec<PathBuf>,
        /// The cipher type to use when encrypting the file(s)
        #[arg(long, value_enum, default_value_t=CipherType::default())]
        cipher_type: CipherType,
        /// Show progress as a log instead of via progressbar
        #[arg(long, default_value_t = false)]
        show_log: bool,
    },
    /// Receive the file(s) sent using the room number and passphrase provided by the sender.
    #[command(name("receive"))]
    Receiver {
        /// The ip address where the server is listening at
        #[arg(long)]
        server_ip: String,
        /// The port where the server is listening at
        #[arg(long)]
        server_port: String,
        /// Specify how to transfer files/directories when any file/directory at the destination
        /// shares the same name
        ///
        /// Overwrite: overwrite existing files/directories
        /// Skip: skip over any errors from conflicting files/directories
        /// Resume: resume an unfinished transfer
        /// Error: abort if existing files/directories share the same name
        #[arg(long, value_enum, default_value_t=ConflictingFileMode::default(), verbatim_doc_comment)]
        conflicting_file_mode: ConflictingFileMode,
        /// Show progress as a log instead of via progressbar
        #[arg(long, default_value_t = false)]
        show_log: bool,
    },
}

pub fn get_trace_settings() -> EnvFilter {
    let crate_name = env!("CARGO_CRATE_NAME");

    let debug_profile = format!("{}=debug,iris=debug", crate_name).into();
    let release_profile = format!("{}=info,iris=info", crate_name).into();

    tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(if cfg!(debug_assertions) {
        debug_profile
    } else {
        release_profile
    })
}

fn setup_tracing() {
    let collector = tracing_subscriber::registry()
        .with(get_trace_settings())
        .with(tracing_subscriber::fmt::layer().with_writer(io::stdout));

    tracing::subscriber::set_global_default(collector).expect("unable to set global collector");
}

fn main() {
    let args = App::parse();
    match args.subcommands {
        Subcommands::Server { ip_address, port } => {
            setup_tracing();
            server::serve(ip_address, port)
        }
        Subcommands::Sender {
            server_ip,
            server_port,
            files,
            cipher_type,
            show_log,
        } => {
            if show_log {
                setup_tracing();
            }
            sender::send(server_ip, server_port, files, cipher_type, show_log)
        }
        Subcommands::Receiver {
            server_ip,
            server_port,
            conflicting_file_mode,
            show_log,
        } => {
            if show_log {
                setup_tracing();
            }
            receiver::receive(server_ip, server_port, conflicting_file_mode, show_log)
        }
    }
}
