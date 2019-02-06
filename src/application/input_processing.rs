const INPUT_SEPARATOR: &str = " ";

pub fn parse_user_input(input: &String) -> (Option<&str>, Option<&str>) {
  let split = input.split(INPUT_SEPARATOR);
  let vec: Vec<&str> = split.collect();
  let vec_len = vec.len();
  if vec_len == 0 {
    // no command found
    return (None, None);
  } else if vec_len == 1 {
    return (Some(vec[0]), None);
  } else {
    // more than one command found -> return two commands and throw away the rest
    return (Some(vec[0]), Some(vec[1]));
  }
}