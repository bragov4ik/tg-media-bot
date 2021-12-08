// Formats for logs some action in chat with specified id and time
pub fn format_log_chat(message: &str, chat_id: i64) -> String {
    let time: String = get_time();
    let id_str: String = format_id(chat_id);
    time + id_str.as_str() + message
}

#[allow(dead_code)]
// Formats for logs something with timestamp
pub fn format_log_time(message: &str) -> String {
    let time: String = get_time();
    time + message
}

fn get_time() -> String {
    format!("{}", chrono::offset::Local::now().format("[%H:%M:%S]"))
}

fn format_id(chat_id: i64) -> String {
    format!("CHAT {}:", chat_id)
}
