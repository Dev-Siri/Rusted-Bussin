use regex::Regex;

// idk what to call it
trait RegexReplace {
    fn replace_fr(&self, target: &str, replacement: &str) -> String;
    fn replace_regex(&self, pattern: &str, replacement: &str) -> String;
}

impl RegexReplace for String {
    // I originally used regex here, but turns out, the regex crate's parse method
    // does not have support for look-around, including look-ahead and look-behind.
    // so replaced it with a chatgpt implementation. (I'm too dumb to
    // implement whatever the hell that regex searched for.)
    fn replace_fr(&self, target: &str, replacement: &str) -> String {
        let mut result = String::with_capacity(self.len());

        let mut iter = self.chars().peekable();

        while let Some(c) = iter.next() {
            if c.is_ascii_alphanumeric() {
                let mut word = String::new();
                word.push(c);

                while let Some(&next_char) = iter.peek() {
                    if next_char.is_ascii_alphanumeric() {
                        word.push(iter.next().unwrap());
                    } else {
                        break;
                    }
                }

                if word == target && iter.peek().map_or(true, |&c| !c.is_ascii_alphanumeric()) {
                    result.push_str(replacement);
                } else {
                    result.push_str(&word);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn replace_regex(&self, pattern: &str, replacement: &str) -> String {
        let regex = Regex::new(pattern).expect("Invalid Regex");

        regex.replace_all(self, replacement).to_string()
    }
}

pub fn transcribe(code: String) -> String {
    code.replace_fr(";", "!")
        .replace_fr("rn", ";")
        .replace_fr("be", "=")
        .replace_fr("lit", "let")
        .replace_fr("mf", "const")
        .replace_fr("waffle", "println")
        .replace_fr("sus", "if")
        .replace_fr("fake", "null")
        .replace_fr("impostor", "else")
        .replace_fr("nah", "!=")
        .replace_fr("fr", "==")
        .replace_fr("btw", "&&")
        .replace_fr("carenot", "|")
        .replace_fr("bruh", "fn")
        .replace_fr("nerd", "math")
        .replace_fr("yall", "for")
        .replace_fr("smol", "<")
        .replace_fr("thicc", ">")
        .replace_fr("nocap", "true")
        .replace_fr("cap", "false")
        .replace_fr("fuck_around", "try")
        .replace_fr("find_out", "catch")
        .replace_fr("clapback", "exec")
        .replace_fr("yap", "input")
        .replace_regex(": number", "")
        .replace_regex(": string", "")
        .replace_regex(": object", "")
        .replace_regex(": boolean", "")
}
