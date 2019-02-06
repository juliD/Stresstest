use crate::application::message::Message;

pub fn parse_string_message(message: &str) -> Option<Message> {
    match message {
        "start" => Some(Message::Start),
        "stop" => Some(Message::Stop),
        "log" => Some(Message::Log),
        "help" => Some(Message::Help),
        _ => {
            let split = message.split(" ");
            let vec: Vec<&str> = split.collect();
            match vec[0] {
                "reportrequests" => Some(Message::ReportRequests(vec[1].parse().unwrap())),
                _ => None,
            }
        }
    }
}

pub fn serialize_string_message(message: Message) -> String {
    match message {
        Message::Start => "start".to_owned(),
        Message::Stop => "stop".to_owned(),
        Message::Log => "log".to_owned(),
        Message::Help => "help".to_owned(),
        Message::ReportRequests(count) => format!("reportrequests {}", count),
        _ => panic!("failed to serialize actor message"),
    }
}