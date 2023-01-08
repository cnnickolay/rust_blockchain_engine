pub fn shorten_long_string(str: &str) -> String {
    let mut res = String::new();
    res += &str[0..10];
    res += "....";
    res += &str[str.len() - 10..str.len()];
    res.to_string()
}