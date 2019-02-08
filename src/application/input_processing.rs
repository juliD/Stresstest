const INPUT_SEPARATOR: &str = " ";

pub fn parse_user_input(input: &str) -> (Option<&str>, Option<&str>, Option<&str>) {
    match input.lines().next() {
        Some(trimmed) => {
            let split = trimmed.split(INPUT_SEPARATOR);
            let vec: Vec<&str> = split.collect();
            let vec_len = vec.len();
            if vec_len == 0 {
                // no command found
                (None, None, None)
            } else if vec_len == 1 {
                (Some(vec[0]), None, None)
            } else if vec_len == 2 {
                (Some(vec[0]), Some(vec[1]), None)
            } else {
               // more than three commands found -> return three commands and throw away the rest
                (Some(vec[0]), Some(vec[1]), Some(vec[2]))
            }
        }
        None => (None, None, None)
    }
}