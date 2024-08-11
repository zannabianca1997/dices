use lazy_regex::{regex, Lazy, Regex};

pub static ESCAPE_RE: &Lazy<Regex> =
    regex!(r#"\\([\\nrt0'"]|x[0-7][0-9a-fA-F]|u\{[0-9a-fA-F]{1,6}\})"#);

/*
   // escape code
      let mut rest = self.0;
       while let Some(matched) = ESCAPE_RE.find(rest) {
           let before = &rest[..matched.start()];
           let escaped = &rest[matched.range()];
           rest = &rest[matched.end()..];

           let escaped = match escaped.as_bytes()[1] {
               b'\\' => '\\',
               b'n' => '\n',
               b'r' => '\r',
               b't' => '\t',
               b'0' => '\0',
               b'\'' => '\'',
               b'"' => '"',

               b'x' => {
                   let code = u8::from_str_radix(&escaped[2..], 16).unwrap();
                   code as char
               }
               b'u' => {
                   let code = u32::from_str_radix(&escaped[3..escaped.len() - 1], 16).unwrap();
                   char::try_from(code).unwrap()
               }

               _ => unreachable!("The regex shouldn't match other escapes"),
           };
       }
       f.write_str(rest)
*/
