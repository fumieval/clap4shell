fn main() {
    let input = std::io::read_to_string(std::io::stdin()).unwrap_or_else(|error| {
        eprintln!("clap4shell: {error}");
        std::process::exit(1);
    });
    match clap4shell::eval(&input, std::env::args_os()) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            match &error {
                clap4shell::Error::Message(message) => eprintln!("clap4shell: {message}"),
                clap4shell::Error::Usage(usage) => eprintln!("{usage}"),
            }
            std::process::exit(1);
        }
    }
}
