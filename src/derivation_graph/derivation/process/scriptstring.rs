use regex::Regex;
use std::array::IntoIter;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;
use std::vec::Vec;
use steel::SteelVal;
use steel::rvals::{Custom, IntoSteelVal};
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;
use steel_derive::Steel;

#[derive(Debug, Clone)]
pub struct ScriptString {
    pub string_fragments: VecDeque<String>,
    pub interpolations: Vec<SteelVal>,
}

impl ScriptString {
    pub fn new(script: String) -> Result<ScriptString, String> {
        let script = indent_string(script)?;
        let ss = Lexer::new(&script).parse().map_err(|x| x.to_string())?;
        Ok(ss)
    }
    pub fn set_interpolations(&mut self, new_interpolations: Vec<SteelVal>) {
        self.interpolations = new_interpolations;
    }

    pub fn interpolations(&self) -> Vec<SteelVal> {
        self.interpolations.clone()
    }
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_type::<ScriptString>("ScriptString?");
    module.register_fn("ScriptString", ScriptString::new);
    module.register_fn(
        "ScriptString::interpolations",
        ScriptString::interpolations,
    );
    module.register_fn(
        "ScriptString::set_interpolations",
        ScriptString::set_interpolations,
    );
}

impl std::fmt::Display for ScriptString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut script_fragments = self.string_fragments.clone();
        let mut s = script_fragments
            .pop_front()
            .expect("couldn't get string fragments");
        for (i, frag) in
            std::iter::zip(self.interpolations.iter(), script_fragments.iter())
        {
            let is = i.to_string();
            s = s + &is + frag;
        }
        write!(f, "{}", s)
    }
}

pub fn indent_string(s: String) -> Result<String, String> {
    let mut strings = s.split("\n").peekable();
    //strings.next(); // consumes first element of iterator (will be needed to add script annotations like 'bash')
    let whitespace_regex =
        Regex::new(r"^(\s*)").expect("Couldn't make whitespace regex");
    let first_elem = match strings.peek() {
        Some(v) => v,
        None => return Err("String is empty!!".to_string()),
    };
    let indents = match whitespace_regex.captures(first_elem) {
        Some(v) => v.get(1).expect("indent regex failed").as_str(),
        None => "",
    };

    let s: String = strings
        .map(|i| match i.strip_prefix(indents) {
            Some(v) => v,
            None => i,
        })
        .map(|i| i.to_string())
        .collect::<std::vec::Vec<String>>()
        .join("\n");

    Ok(s)
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self{
        Self{
            chars: s.chars().peekable()
       }
    }
    fn parse(&mut self) -> Result<ScriptString, String> {
        let mut fragments = VecDeque::new();
        let mut interpolations = Vec::new();
        let mut current_string = String::new();
        while let Some(ch) = self.chars.next() {
            match ch {
                '\\' => {
                    if let Some('{') = self.chars.peek() {
                        let parsed = self.parse_double_braces(false)?;
                        if !(parsed.starts_with("{{") & parsed.ends_with("}}"))
                        {
                            current_string.push('\\')
                        }
                        current_string.push_str(&parsed)
                    } else{
                        current_string.push('\\')
                    }
                    
                }
                '{' => {
                    if let Some('{') = self.chars.peek() {
                        let parsed = self.parse_double_braces(true)?;
                        let parsed = &parsed[2..parsed.len()-2];
                        interpolations.push(
                            parsed
                                .into_steelval()
                                .map_err(|x| x.to_string())?,
                        );
                        fragments.push_back(current_string);
                        current_string = "".to_string();
                    } else {
                        current_string.push('{');
                    }
                }
                c => current_string.push(c)
            }
        }
        fragments.push_back(current_string);

        Ok(ScriptString {
            string_fragments: fragments,
            interpolations,
        })
    }
    fn parse_double_braces(&mut self, err: bool) -> Result<String, String> {
        let mut s = String::new();
        s.push('{'); // add first brace (it has already been eaten)
        if let Some('{') = self.chars.peek() {
            self.chars.next(); // eat second {
            s.push('{');
            loop {
                match self.chars.next() {
                    Some('}') => {
                        if let Some('}') = self.chars.peek() {
                            break;
                        }
                        s.push('}')
                    }
                    Some(c) => s.push(c),
                    None => {
                        return if err {
                            Err("Unmatched closing }} in script string".into())
                        } else {
                            Ok(s)
                        };
                    }
                }
            }
            self.chars.next(); // eats }}
            s.push_str("}}")
        }

        Ok(s)
    }
}

impl Custom for ScriptString {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(format!(
            "{:?}",
            self.interpolations
        )))
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn basic_interpolation(){
        let ss = Lexer::new("a {{interp}} b").parse().expect("couldn't parse");
        println!("{}",ss.interpolations[0]);
        assert!(ss.interpolations.len() == 1);
    }

    #[test]
    fn double_braces(){
        let ss = Lexer::new("{interp}}").parse_double_braces(true).expect("couldn't parse");
        assert!(ss == "{{interp}}")
    }
}
