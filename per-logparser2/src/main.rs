use clap::Parser;
mod consts;
mod parse;
mod table;

#[derive(clap::Parser, Debug)]
#[command(author, version, about = "PER log exporter")]
struct Args {
    /// Path to the .dbc file (must exist and be a file)
    vcan_dbc: std::path::PathBuf,

    /// Path to an input folder (must exist and be a directory)
    input_dir: std::path::PathBuf,

    /// Path to an output folder (may not exist; if it exists it must be a directory)
    output_dir: std::path::PathBuf,
}

fn validate_paths(args: &Args) -> Result<(), String> {
    // 1) vcan_dbc must exist and be a file
    if !args.vcan_dbc.exists() {
        return Err(format!(
            "DBC file does not exist: {}",
            args.vcan_dbc.display()
        ));
    }
    if !args.vcan_dbc.is_file() {
        return Err(format!(
            "DBC path is not a file: {}",
            args.vcan_dbc.display()
        ));
    }
    if args.vcan_dbc.extension().and_then(|s| s.to_str()) != Some("dbc") {
        return Err(format!(
            "DBC file does not have .dbc extension: {}",
            args.vcan_dbc.display()
        ));
    }

    // 2) input_dir must exist and be a directory
    if !args.input_dir.exists() {
        return Err(format!(
            "Input directory does not exist: {}",
            args.input_dir.display()
        ));
    }
    if !args.input_dir.is_dir() {
        return Err(format!(
            "Input path is not a directory: {}",
            args.input_dir.display()
        ));
    }

    // 3) output_dir may not exist. If it exists, it must be a directory.
    if args.output_dir.exists() && !args.output_dir.is_dir() {
        return Err(format!(
            "Output path exists but is not a directory: {}",
            args.output_dir.display()
        ));
    }

    Ok(())
}

fn main() {
    // env_logger::init();

    let args = Args::parse();
    if let Err(e) = validate_paths(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let parser = match can_unpack::Parser::from_dbc_file(&args.vcan_dbc) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing DBC file: {}", e);
            std::process::exit(1);
        }
    };

    // let msg_defs = parser.messages();
    // for msg in msg_defs {
    //     println!(
    //         "Message: {} (ID: {}), Transmitter: {:?}",
    //         msg.message_name(),
    //         match msg.message_id() {
    //             can_dbc::MessageId::Standard(id) => *id as u32,
    //             can_dbc::MessageId::Extended(id) => *id,
    //         },
    //         msg.transmitter()
    //     );
    // }

    let parsed = parse::parse_log_files(&args.input_dir, &parser);
    let chunked_parsed = parse::chunk_parsed(parsed);

    let mut table_builder = table::TableBuilder::new();
    table_builder.create_header(&parser);
    table_builder.create_and_write_tables(&args.output_dir, chunked_parsed);
}
