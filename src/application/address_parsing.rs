use std::net::SocketAddr;

const SET_TARGET_COMMAND: &str = "target";
const COMMAND_PARAMETER_SEPARATOR: &str = " ";

// TODO: What is the best type to use here? &str?
pub fn verify_target_address(user_input: &str) -> bool {
  let parsed_address = user_input.parse::<SocketAddr>();
  // TODO: return address in parsed form here instead of throwing it away?
  match parsed_address {
    Ok(_) => true,
    Err(_) => false,
  }
}