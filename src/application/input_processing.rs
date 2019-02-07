const INPUT_SEPARATOR: &str = " ";

pub fn parse_user_input(input: &str) -> (Option<&str>, Option<&str>) {
    match input.lines().next() {
        Some(trimmed) => {
            let split = trimmed.split(INPUT_SEPARATOR);
            let vec: Vec<&str> = split.collect();
            let vec_len = vec.len();
            if vec_len == 0 {
                // no command found
                (None, None)
            } else if vec_len == 1 {
                (Some(vec[0]), None)
            } else {
                // more than one command found -> return two commands and throw away the rest
                (Some(vec[0]), Some(vec[1]))
            }
        }
        None => (None, None)
    }
}
