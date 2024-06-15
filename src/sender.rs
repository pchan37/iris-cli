use std::path::PathBuf;

use dialoguer::console::{style, Term};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use iris::{
    get_passphrase_from_str_wordlist, get_sender_communication_channels, CipherType,
    SenderProgressCommunication, SenderProgressMessage, WORDLIST,
};

#[cfg(unix)]
use crate::constants::IRIS_SECRET_ENV_VAR;

pub fn send(
    server_ip: String,
    port: String,
    files: Vec<PathBuf>,
    cipher_type: CipherType,
    show_log: bool,
) {
    let (worker_communication, progress_communication) = get_sender_communication_channels();
    let passphrase = get_passphrase_from_str_wordlist(&WORDLIST);
    let passphrase_clone = passphrase.clone();
    std::thread::spawn(move || {
        perform_task(
            server_ip,
            port,
            cipher_type,
            passphrase_clone,
            files,
            progress_communication,
        );
    });

    let term = Term::stdout();
    let _ = term.clear_screen();

    if !show_log {
        let bar = create_progress_bar();
        let mut size_of_current_file = 0;
        while let Ok(message) = worker_communication.read() {
            if let Some(message) = message {
                handle_sender_progress_message(
                    message,
                    &bar,
                    &passphrase,
                    &mut size_of_current_file,
                );
            }
        }
    } else {
        while let Ok(message) = worker_communication.read() {
            match message {
                Some(SenderProgressMessage::Error(e)) => {
                    eprintln!(
                        "{}",
                        style(format!("Could not finish because of error: {e}")).red()
                    );
                    std::process::exit(1);
                }
                _ => continue,
            }
        }
    }
}

fn perform_task(
    server_ip: String,
    port: String,
    cipher_type: CipherType,
    passphrase: String,
    files: Vec<PathBuf>,
    progress_communication: SenderProgressCommunication,
) {
    if let Err(e) = iris::simple_send(
        server_ip,
        port,
        cipher_type,
        &passphrase,
        files,
        &progress_communication,
    ) {
        progress_communication
            .write(SenderProgressMessage::Error(e))
            .unwrap();
    }

    drop(progress_communication);
}

fn handle_sender_progress_message(
    message: SenderProgressMessage,
    bar: &ProgressBar,
    passphrase: &str,
    size_of_current_file: &mut u64,
) {
    match message {
        SenderProgressMessage::AssignedRoomIdentifier { room_identifier } => {
            #[cfg(unix)]
            println!("Ask the receiver to run:\n\n\t$ {IRIS_SECRET_ENV_VAR}={room_identifier}-{passphrase} iris receive\n");
            #[cfg(unix)]
            println!("Alternatively, ask them to enter the following information after running:\n\n\t$ iris receive\n\n\tRoom #: {room_identifier}\n\tPassphrase: {passphrase}\n");
            #[cfg(windows)]
            println!("Ask them to enter the following information after running:\n\n\t$ iris receive\n\n\tRoom #: {room_identifier}\n\tPassphrase: {passphrase}\n");
        }
        SenderProgressMessage::SetCipher { cipher_type } => {
            bar.set_prefix("🔒");
            bar.set_message(format!("Cipher set to {cipher_type:?}"));
        }
        SenderProgressMessage::TransferMetadata {
            total_files: _,
            total_bytes,
        } => {
            bar.set_length(total_bytes);
        }
        SenderProgressMessage::FileMetadata {
            filename,
            file_size,
        } => {
            *size_of_current_file = file_size;
            bar.set_message(format!("Transferring {filename:?}"));
        }
        SenderProgressMessage::ChunkSent { size } => {
            bar.inc(size);
        }
        SenderProgressMessage::FileDone => (),
        SenderProgressMessage::DirectoryCreated => (),
        SenderProgressMessage::FileSkipped => {
            bar.inc(*size_of_current_file);
        }
        SenderProgressMessage::Error(e) => {
            bar.abandon_with_message(format!(
                "{}",
                style(format!("Could not finish because of error: {e}")).red()
            ));
        }
    }
}

fn create_progress_bar() -> ProgressBar {
    ProgressBar::new(100)
        .with_style(
            ProgressStyle::with_template(
                "{prefix} {bar:20.cyan/blue} {percent_precise}% {eta_precise}\n{wide_msg}",
            )
            .unwrap(),
        )
        .with_finish(ProgressFinish::WithMessage(
            format!("{}", style("Transfer complete!").green()).into(),
        ))
}
