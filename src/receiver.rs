use dialoguer::console::{style, Term};
use dialoguer::{Input, Password};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use iris::{
    get_receiver_communication_channels, ConflictingFileMode, IrisError,
    ReceiverProgressCommunication, ReceiverProgressMessage,
};
use snafu::ResultExt;

use crate::constants::IRIS_SECRET_ENV_VAR;
use crate::errors::{Error, MissingPassphraseSnafu, MissingRoomIdentifierSnafu};

pub fn receive(server_ip: String, port: String, conflicting_file_mode: ConflictingFileMode) {
    let (worker_communication, progress_communication) = get_receiver_communication_channels();
    std::thread::spawn(move || {
        perform_task(
            server_ip,
            port,
            conflicting_file_mode,
            progress_communication,
        );
    });

    let term = Term::stdout();
    let _ = term.clear_screen();

    let bar = create_progress_bar();
    let mut size_of_current_file = 0;
    while let Ok(message) = worker_communication.read() {
        if let Some(message) = message {
            handle_receiver_progress_message(message, &bar, &mut size_of_current_file);
        }
    }
}

fn perform_task(
    server_ip: String,
    port: String,
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: ReceiverProgressCommunication,
) {
    let result = match get_passphrase() {
        Ok((room_identifier, passphrase)) => iris::simple_receive(
            server_ip,
            port,
            &room_identifier,
            &passphrase,
            conflicting_file_mode,
            &progress_communication,
        ),
        Err(e) => {
            eprintln!("{}", style(format!("Error: {e}")).red());
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        progress_communication
            .write(ReceiverProgressMessage::Error(e))
            .unwrap();
    }

    drop(progress_communication);
}

fn handle_receiver_progress_message(
    message: ReceiverProgressMessage,
    bar: &ProgressBar,
    size_of_current_file: &mut u64,
) {
    match message {
        ReceiverProgressMessage::SetCipher { cipher_type } => {
            bar.set_prefix("🔒");
            bar.set_message(format!("Cipher set to {cipher_type:?}"));
        }
        ReceiverProgressMessage::TransferMetadata {
            total_files: _,
            total_bytes,
        } => {
            bar.set_length(total_bytes);
        }
        ReceiverProgressMessage::FileMetadata {
            filename,
            file_size,
        } => {
            *size_of_current_file = file_size;
            bar.set_message(format!("Transferring {filename:?}"));
        }
        ReceiverProgressMessage::ChunkReceived { size } => {
            bar.inc(size);
        }
        ReceiverProgressMessage::FileDone => (),
        ReceiverProgressMessage::DirectoryCreated => (),
        ReceiverProgressMessage::FileSkipped => {
            bar.inc(*size_of_current_file);
        }
        ReceiverProgressMessage::Error(e) => {
            bar.abandon_with_message(format!(
                "{}",
                style(format!("Could not finish because of error: {e}")).red()
            ));
        }
    }
}

fn get_passphrase() -> Result<(String, String), Error> {
    match std::env::var(IRIS_SECRET_ENV_VAR) {
        Ok(passphrase) => match passphrase.split_once('-') {
            Some((room_identifier, passphrase)) => {
                Ok((room_identifier.to_string(), passphrase.to_string()))
            }
            None => Err(IrisError::InvalidPassphrase)?,
        },
        Err(_) => {
            let room_identifier: String = Input::new()
                .with_prompt("Room #")
                .interact_text()
                .context(MissingRoomIdentifierSnafu)?;
            let passphrase = Password::new()
                .with_prompt("Passphrase")
                .interact()
                .context(MissingPassphraseSnafu)?;
            println!();

            Ok((room_identifier, passphrase))
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
