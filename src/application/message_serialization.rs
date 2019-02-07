use crate::application::address_parsing::*;
use crate::application::input_processing::*;
use crate::application::message::Message;

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
                Some(param) => {
                    let int_param: Option<u64> = match param.parse() {
                        Ok(p) => Some(p),
                        Err(error) => {
                            println!("could not parse argument: {}", error);
                            None
                        }
                    };
                    int_param.map(|p| Message::ReportRequests(p))
                }
                None => None,
            }
        }
        Some("target") => {
            // TODO: prettify
            match input_part_2 {
                Some(address_raw) => {
                    if verify_target_address(address_raw) {
                        Some(Message::SetTarget(String::from(address_raw)))
                    } else {
                        // TODO: better error handling
                        println!("invalid target address: {}", message);
                        None
                    }
                }
                None => {
                    println!("could not parse target message: {}", message);
                    None
                }
            }
        }
        _ => {
            println!("could not parse message: {}", message);
            None
        }
    }
}

pub fn serialize_message(message: Message) -> String {
    let serialized = match message {
        Message::Start => "start".to_owned(),
        Message::Stop => "stop".to_owned(),
        Message::Log => "log".to_owned(),
        Message::Help => "help".to_owned(),
        Message::ReportRequests(count) => format!("reportrequests {}", count),
        _ => panic!("failed to serialize actor message"),
    };
    return format!("{}\n", serialized);
}
