use regex::Regex;

pub fn get_thumbs_up() -> String { String::from("👍") }

pub fn get_thumbs_down() -> String { String::from("👎") }

pub fn is_url(url: &str) -> bool {
    let re = Regex::new(
        r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"
    ).unwrap();
    re.is_match(url)
}
