use crate::consts;

pub struct ParsedMessage {
    pub timestamp: u32,
    pub decoded: can_decode::DecodedMessage,
}

pub fn parse_log_files(
    in_folder: &std::path::Path,
    parser: &can_decode::Parser,
) -> Vec<ParsedMessage> {
    let mut all_parsed = Vec::new();
    let mut file_paths = std::fs::read_dir(in_folder)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("log")
        })
        .collect::<Vec<_>>();
    file_paths.sort();
    for path in file_paths {
        println!("Parsing log file: {}", path.display());
        let parsed = parse_log_file(&path, parser);
        all_parsed.extend(parsed);
    }

    all_parsed
}

fn parse_log_file(in_file: &std::path::Path, parser: &can_decode::Parser) -> Vec<ParsedMessage> {
    let content = std::fs::read(in_file).unwrap();
    let mut offset = 0;
    let mut parsed = Vec::new();
    while offset + consts::MSG_BYTE_LEN <= content.len() {
        let timestamp = u32::from_le_bytes(
            content[offset + consts::FRAME_TYPE_OFFSET..offset + consts::TIMESTAMP_OFFSET]
                .try_into()
                .unwrap(),
        );
        let can_id = u32::from_le_bytes(
            content[offset + consts::TIMESTAMP_OFFSET..offset + consts::ID_OFFSET]
                .try_into()
                .unwrap(),
        );
        // let bus_id = content[offset + consts::ID_OFFSET];
        let dlc = content[offset + consts::DLC_OFFSET];
        let data =
            &content[offset + consts::DATA_OFFSET..offset + consts::DATA_OFFSET + dlc as usize];
        offset += consts::MSG_BYTE_LEN;

        let is_extended = (can_id & consts::CAN_EFF_FLAG) != 0;
        let arb_id = if is_extended {
            can_id & consts::CAN_EXT_ID_MASK
        } else {
            can_id & consts::CAN_STD_ID_MASK
        };
        if let Some(decoded) = parser.decode_msg(arb_id, data) {
            parsed.push(ParsedMessage { timestamp, decoded });
        } else {
             log::error!(
                "Failed to parse: frame ID 0x{:X} ({}), data: {:02X?}",
                arb_id,
                arb_id,
                data
            );
        }
    }

    parsed
}

pub fn chunk_parsed(parsed: Vec<ParsedMessage>) -> Vec<Vec<ParsedMessage>> {
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut last_timestamp = None;

    for msg in parsed {
        if let Some(last_ts) = last_timestamp
            && (msg.timestamp < last_ts || msg.timestamp - last_ts > consts::MAX_JUMP_MS)
            && !current_chunk.is_empty()
        {
            chunks.push(current_chunk);
            current_chunk = Vec::new();
        }
        last_timestamp = Some(msg.timestamp);
        current_chunk.push(msg);
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    // Sort messages within each chunk by timestamp
    for chunk in &mut chunks {
        chunk.sort_by_key(|m| m.timestamp);
    }

    chunks
}
