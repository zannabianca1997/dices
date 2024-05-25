use std::{borrow::Cow, rc::Rc};

use either::Either::{Left, Right};
use peg::str::LineCol;

use crate::{
    expr::{
        Expr::{self, *},
        Receiver,
    },
    identifier::IdentStr,
};

peg::parser! {
  grammar grammar() for str {
    /// A unsigned number literal
    rule number() -> i64
        = n:$(['0'..='9']+) {? n.parse().or(Err("i64")) }

    /// An identifier
    rule ident() -> &'input IdentStr
        = i:$(
            (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
            ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
        ) {? IdentStr::new(i).ok_or("identifier") }


        /// A string literal
        rule str_lit() -> std::borrow::Cow<'input,str>
            = ['"'] parts: (
                lit: $( [^ '"' | '\\']+ ) { Left(lit) }
                / "\\" escape:(
                      "n" {'\n'}
                      / "r" {'\r'}
                      / "t" {'\t'}
                      / "0" {'\0'}
                      / "\\" {'\\'}
                      / "\'" {'\''}
                      / "\"" {'"'}
                      / hex:(
                        "x" hex: $(['0'..='7']['a'..='f'|'A'..='F'|'0'..='9']) {hex}
                        / "u{" hex: $(['a'..='f'|'A'..='F'|'0'..='9']*<,6>) "}" {hex}) {?
                             char::from_u32(u32::from_str_radix(hex, 16).unwrap())
                                .ok_or("unicode codepoint")
                        }
                      / expected!("escape code")

                ) {Right(escape)}
             )* ['"'] {
                if let [Left(part)] = &*parts {
                    Cow::Borrowed(*part)
                } else {
                    // some escapes remains. We need to build a string
                    let mut buf = std::string::String::with_capacity(
                        parts.iter().map(|p| match p {
                            Left(s) => s.len(),
                            Right(c) => c.len_utf8(),
                        }).sum()
                    );
                    for p in parts {
                        match p {
                            Left(s) => buf.push_str(s),
                            Right(ch) => buf.push(ch),
                        }
                    }
                    Cow::Owned(buf)
                }
             }

    /// A receiver for a set expression
    rule receiver() -> Receiver
        = "_"       { Receiver::Discard        }
        / i:ident() { Receiver::Set(i.into())  }
        / "let" _ i:ident() { Receiver::Let(i.into()) }


    /// Parse an expression
    rule expr() -> Expr
        = precedence!{
            receiver:receiver() _ "=" _ value:@ { Set { receiver, value: Box::new(value) }}
            --
            "|" _ p:((i: ident() { Rc::<IdentStr>::from(i) }) ** (_ "," _)) _ "|" _ body:@ {
                Expr::Function { params: p.into(), body : Rc::new(body)}
            }
            --
            a:(@) _ "+" _ b:@ { Sum(vec![a,b]) }
            a:(@) _ "-" _ b:@ { Sum(vec![a, Neg(Box::new(b))]) }
            --
            a:(@) _ "~" _ b:@ { Join(Box::new(a), Box::new(b)) }
            --
            a:(@) _ "*" _ b:@ { Mul(Box::new(a), Box::new(b)) }
            a:(@) _ "/" _ b:@ { Div(Box::new(a), Box::new(b)) }
            a:(@) _ "%" _ b:@ { Rem(Box::new(a), Box::new(b)) }
            --
            a:(@) _ "^" _ b:@ { Rep(Box::new(a), Box::new(b)) }
            --
            "+" _ a:@ { Sum(vec![a]) }
            "-" _ a:@ { Neg(Box::new(a)) }
            --
            "d" _ f:@ { Dice(Box::new(f))}
            n:@ _ "d" _ f:(@) { Rep(Box::new(Dice(Box::new(f))), Box::new(n))}
            --
            f:@ _ "(" _ p:(expr() ** (_ "," _)) _ ")" {
                Expr::Call { fun: Box::new(f), params: p }
            }
            --
            "null"      { Null }
            "true"      { Bool(true) }
            "false"     { Bool(false) }
            n:number()  { Number(n) }
            i:ident()   { Reference(i.into()) }
            s:str_lit() { String(s.into()) }

            "[" _ l:(expr() ** (_ "," _)) _ "]" {
                List(l)
            }

            "<|" _
                elems:(
                    (
                        n:(n:ident() {n.as_ref().into()} / s:str_lit() {s.into()}) _ ":" _ v:expr() {
                            (n,v)
                        }
                    ) ** (_ "," _)
                )
            _ "|>" {
                Map(elems.into_iter().collect())
            }

            "(" _ e:expr() _ ")" { e }

            "{" _  inner:scope_inner() _ "}" { Scope(inner)}
        }
        / expected!("expression")


    /// Parse a command - a statetement with optional space
    pub rule scope_inner() -> Vec<Expr>
        = _  stms:expr() ** (_ ";" _) dangling:";"? _ {
            let mut stms = stms;
            if dangling.is_some() {
                stms.push(Expr::Null)
            }
            stms
        }

    /// Parse whitespace and comments, discarding them
    rule _ -> ()
        = quiet!{
            (
                [' ' | '\t' | '\r' | '\n']       // Whitespace
                / "//" [^'\n']* (['\n'] / ![_])  // C-style line comment
                / "/*" (!"*/" [_])* "*/"         // C-style block comment
            )* { () }
        }
  }
}

pub type ParseError = peg::error::ParseError<LineCol>;

pub fn parse_exprs(input: &str) -> Result<Vec<Expr>, ParseError> {
    grammar::scope_inner(input)
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr::{self, *};

    use super::parse_exprs;

    fn parse_test(src: &str, res: &[Expr]) {
        match parse_exprs(src) {
            Ok(parsed) => assert_eq!(parsed, res),
            Err(err) => panic!("{err:#}"),
        }
    }

    macro_rules! parse_tests {
        ()=>{};
        (
            $name:ident : $src:literal => $res:expr ;
            $($rest:tt)*
        ) => {
            #[test]
            fn $name() {
                parse_test($src, &[$res])
            }
            parse_tests!{$($rest)*}
        };
        (
            $name:ident : $src:literal => [$($res:expr),* $(,)?] ;
            $($rest:tt)*
        ) => {
            #[test]
            fn $name() {
                parse_test($src, &[$($res),*])
            }
            parse_tests!{$($rest)*}
        };
    }

    parse_tests! {
        empty: "" => Null;
        line_comment: "// comment" => Null;
        block_comment: "/* comment */" => Null;
        number: "3" => Number(3);
        null:"null"=> Null;
        bool_t:"true"=> Bool(true);
        bool_f:"false"=> Bool(false);

        list: "[3, 2 ,false , null]" => List(vec![Number(3), Number(2), Bool(false), Null]);
        list_empty: "[]" => List(vec![]);
        list_nested: "[[]]" => List(vec![List(vec![])]);

        map: "<|a:3, b:2, __0:false , o:null|>" => Map(From::from([("a".into(),Number(3)), ("b".into(),Number(2)), ("__0".into(),Bool(false)), ("o".into(),Null)]));
        map_empty: "<||>" => Map(From::from([]));

        // TODO: TEST MORE
    }
}
