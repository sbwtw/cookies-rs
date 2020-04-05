use std::io::{Read, BufRead, BufReader};

use cookie::{Cookie, CookieJar};
use regex::Regex;
use time::OffsetDateTime;

pub struct Parser<R: Read> {
    reader: BufReader<R>,
    cookie_jar: CookieJar,
}

impl<R: Read> Parser<R>
{
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            cookie_jar: CookieJar::new(),
        }
    }

    pub fn parse(&mut self) -> bool {
        while let Some(cookie) = self.next_cookie() {
            self.cookie_jar.add(cookie);
        }

        true
    }

    pub fn cookie_jar(self) -> CookieJar {
        self.cookie_jar
    }

    fn next_cookie(&mut self) -> Option<Cookie<'static>> {
        let line = self.next_cookie_line()?;
        let regex = Regex::new(r"\s+").unwrap();
        let parts = regex.splitn(&line, 7).collect::<Vec<_>>();
        if parts.len() != 7 {
            return self.next_cookie();
        }

        let http_only = parts[0].starts_with("#HttpOnly_");
        let truncate = if http_only { 10 } else { 0 };
        let domain = parts[0][truncate..].to_owned();
        let timestamp = parts[4].parse().unwrap_or(0);
        // let include_subdomains = parts[1] == "TRUE";
        let cookie = Cookie::build(parts[5], parts[6])
            .domain(domain)
            .path(parts[2])
            .secure(parts[3] == "TRUE")
            .expires(OffsetDateTime::from_unix_timestamp(timestamp))
            .http_only(http_only)
            .finish();

        Some(cookie.into_owned())
    }

    fn next_cookie_line(&mut self) -> Option<String> {
        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(s) => if s == 0 { return None },
                Err(_) => return None,
            }

            if !line.starts_with("#HttpOnly_") && line.starts_with("#") {
                continue;
            }

            let trim = line.trim();
            return if trim.len() != 0 {
                Some(trim.to_owned())
            } else {
                self.next_cookie_line()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_cookie_parse() {
        let cookies = "#HttpOnly_www.example.com    FALSE   /   FALSE   1591442409  name   value";
        let mut parser = Parser::new(cookies.as_bytes());
        let cookie = parser.next_cookie().unwrap();

        assert_eq!(cookie.domain(), Some("www.example.com"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.name(), "name");
        assert_eq!(cookie.value(), "value");
    }

    #[test]
    fn test_skip_comment() {
        let cookies = "# Comment \n#HttpOnly_www.example.com    FALSE   /   FALSE   1591442409  name   value";
        let mut parser = Parser::new(cookies.as_bytes());
        assert!(parser.next_cookie_line().is_some());
        assert_eq!(None, parser.next_cookie_line());
    }

    #[test]
    fn test_cookie_jar() {
        let cookies = "# Comment \n# Comment\n\n#HttpOnly_www.example.com    FALSE   /   FALSE   1591442409  name   value";
        let mut parser = Parser::new(cookies.as_bytes());
        parser.parse();
        let jar = parser.cookie_jar();

        assert_eq!(jar.iter().count(), 1);
        let cookie = jar.get("name");
        assert!(cookie.is_some());
        assert_eq!(cookie.unwrap().name(), "name");
        assert_eq!(cookie.unwrap().value(), "value");
    }

    // #[test]
    // fn test_read_file() {
    //     let f = File::open("/home/cookies.txt").unwrap();
    //     let mut parser = Parser::new(f);
    //     parser.parse();
    //     let jar = parser.cookie_jar().unwrap();
    //
    //     println!("{}", jar.iter().count());
    // }
}
