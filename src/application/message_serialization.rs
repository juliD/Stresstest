use crate::application::message::Message;
use crate::application::input_processing::*;
use crate::application::address_parsing::*;

pub fn parse_message(message: &str) -> Option<Message> {
    let (input_part_1, input_part_2) = parse_user_input(message);
    match input_part_1 {
        Some("start") => Some(Message::Start),
        Some("stop") => Some(Message::Stop),
        Some("log") => Some(Message::Log),
        Some("help") => Some(Message::Help),
        Some("reportrequests") => {
            // TODO: prettify
            match input_part_2 {
                // TODO: handle parsing error
                Some(param) => Some(Message::ReportRequests(param.parse().unwrap())),
                None => None,
            }
        },
        Some("target") => {
            // TODO: prettify
            match input_part_2 {
                Some(address_raw) => {
                    if verify_target_address(address_raw) {
                        Some(Message::SetTarget(String::from(address_raw)))
                    } else {
                        // TODO: better error handling
                        println!("invalid target address");
                        None
                    }
                },
                None => None,
            }
        }
        _ => None
    }
}

pub fn serialize_message(message: Message) -> String {
    match message {
        Message::Start => "start".to_owned(),
        Message::Stop => "stop".to_owned(),
        Message::Log => "log".to_owned(),
        Message::Help => "help".to_owned(),
        Message::ReportRequests(count) => format!("reportrequests {}", count),
        _ => panic!("failed to serialize actor message"),
    }
}