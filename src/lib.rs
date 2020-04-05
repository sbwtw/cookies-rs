
mod parser;

use std::io;
use std::path::Path;
use std::convert::AsRef;

use cookie::CookieJar;
use parser::Parser;
use std::fs::File;

#[doc(hidden)]
pub fn read_from_file<P: AsRef<Path>>(p: P) -> io::Result<CookieJar> {
    let f = File::open(p.as_ref())?;
    let mut parser = Parser::new(f);
    parser.parse();
    Ok(parser.cookie_jar())
}

#[macro_export]
macro_rules! load_cookies {
    ($file:expr) => {{
        $crate::read_from_file($file)
    }};
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_macro() {
    //     let jar = load_cookies!("/home/cookies.txt").unwrap();
    //     for cookie in jar.iter() {
    //         println!("{:?}", cookie);
    //     }
    // }
}

