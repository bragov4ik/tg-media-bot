/// Format action in chat for logging.
///
/// Add the chat id and a timestamp.
pub fn format_log_chat(message: &str, chat_id: i64) -> String {
    let time: String = get_time();
    let id_str: String = format_id(chat_id);
    time + id_str.as_str() + message
}

#[allow(dead_code)]
/// Format action for logging.
///
/// Add a timestamp.
pub fn format_log_time(message: &str) -> String {
    let time: String = get_time();
    time + message
}

// Get current timestamp.
fn get_time() -> String {
    format!("{}", chrono::offset::Local::now().format("[%H:%M:%S]"))
}

// Get formatted chat id.
fn format_id(chat_id: i64) -> String {
    format!("CHAT {}:", chat_id)
}
