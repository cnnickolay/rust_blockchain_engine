pub fn shorten_long_string(str: &str) -> String {
    let size = 20;
    let mut res = String::new();
    res += &str[0..size];
    res += "....";
    res += &str[str.len() - size..str.len()];
    res.to_string()
}
