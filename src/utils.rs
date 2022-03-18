pub fn get_time() -> String {
    format!("{}", chrono::offset::Local::now().format("[%H:%M:%S]"))
}

macro_rules! log_chat {
    ($lvl:expr, $chat_id:expr, $($arg:tt)+) => {
        log::log!($lvl, "{} CHAT {}:{}", $crate::utils::get_time(), $chat_id, format!($($arg)+));
    };
}

macro_rules! log_time {
    ($lvl:expr, $($arg:tt)+) => {
        log::log!($lvl, "{}:{}", $crate::utils::get_time(), format!($($arg)+));
    };
}

pub(crate) use log_chat;
pub(crate) use log_time;