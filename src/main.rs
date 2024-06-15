mod constants;
mod errors;
mod receiver;
mod sender;
mod server;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use iris::{CipherType, ConflictingFileMode};

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
    },
}

fn main() {
    let args = App::parse();
    match args.subcommands {
        Subcommands::Server { ip_address, port } => server::serve(ip_address, port),
        Subcommands::Sender {
            server_ip,
            server_port,
            files,
            cipher_type,
        } => sender::send(server_ip, server_port, files, cipher_type),
        Subcommands::Receiver {
            server_ip,
            server_port,
            conflicting_file_mode,
        } => receiver::receive(server_ip, server_port, conflicting_file_mode),
    }
}
