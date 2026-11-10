//! Control sample: constructs a pack for this language must tell apart.
#![allow(dead_code)]

fn parse(input: &str) -> Result<u32, String> {
    let value = read(input)?;
    if value > 10 {
        Err(format!("too large: {value}"))
    } else {
        Ok(value)
    }
}

fn read(input: &str) -> Result<u32, String> {
    let text = format!("{input}");
    if text.is_empty() {
        panic!("empty input");
    }
    text.parse::<u32>().map_err(|e| e.to_string())
}

mod checks {
    #![cfg(test)]

    fn helper(x: u32) -> bool {
        if x > 1 {
            return true;
        }
        false
    }
}