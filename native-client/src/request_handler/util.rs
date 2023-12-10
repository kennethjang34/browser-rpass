use std::io::{self, Read, Write};

use rpass::pass;
use rpass::pass::Error;
use serde::Serialize;

pub fn get_message() -> pass::Result<serde_json::Value> {
    let mut raw_length = [0; 4];
    if let Err(read_length_res) = io::stdin().read_exact(&mut raw_length) {
        return Err(Error::Io(read_length_res));
    }
    let message_length = u32::from_le_bytes(raw_length);
    let mut message = vec![0; message_length as usize];
    io::stdin()
        .read_exact(&mut message)
        .expect("Failed to read message content");
    let parsed = serde_json::from_slice(message.as_slice());
    if let Err(err) = parsed {
        let error_message = format!("Failed to parse JSON: {:?}", err);
        return Err(Error::GenericDyn(error_message));
    } else {
        return Ok(parsed.unwrap());
    }
}

/// Encode a message for transmission
pub fn encode_message<T: Serialize>(message_content: &T) -> pass::Result<Vec<u8>> {
    let encoded_content = serde_json::to_string(message_content)?;
    let encoded_length = (encoded_content.len() as u32).to_le_bytes();
    Ok([&encoded_length, encoded_content.as_bytes()].concat())
}

/// Send an encoded message to stdout
pub fn send_message(encoded_message: &[u8]) -> pass::Result<()> {
    io::stdout()
        .write_all(encoded_message)
        .expect("Failed to write to stdout");
    io::stdout().flush().map_err(|e| Error::Io(e))
}
pub fn send_string_message(message: &str) -> pass::Result<()> {
    let encoded_message = encode_message(&message)?;
    send_message(&encoded_message)?;
    Ok(())
}
pub fn send_as_json<T: Serialize>(message_content: &T) -> pass::Result<()> {
    let json = serde_json::to_string(&message_content)?;
    let encoded_message = encode_message(&json)?;
    send_message(&encoded_message)?;
    Ok(())
}
pub fn filter_by_query(
    _query: &str,
    _entries: &Vec<pass::PasswordEntry>,
) -> Vec<pass::PasswordEntry> {
    todo!();
}
