use dialoguer::console::style;

pub fn serve(ip_address: String, port: String) {
    if let Err(e) = iris::serve(ip_address, port) {
        eprintln!("{}", style(format!("Error: {e}")).red());
        std::process::exit(1);
    }
}
