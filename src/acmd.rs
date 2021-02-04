use crate::utils;

pub struct AcmdMove {
    pub category: String,
    pub kind: String,
    pub animation: String,
    pub animcmd: String,
    pub name: String,
    pub body: String
}

impl std::fmt::Display for AcmdMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Category: {}, Kind: {}, Animation: {}, Animcmd: {}\n", self.category, self.kind, self.animation, self.animcmd)?;
        write!(f, "Name: {}\nBody:\n{}", self.name, self.body)
    }
}

impl AcmdMove {
    fn parse_attribute(attribute: String, ident: &'static str) -> String {
        let attribute = utils::remove_comments(attribute);
        let attribute = utils::remove_whitespace(attribute);
        if let Some(equals) = attribute.find("=") {
            if attribute.chars().take(equals).collect::<String>() != ident {
                panic!("Invalid {}: no {} ident.", ident, ident);
            }
            utils::get_unqualified_name(attribute.chars().skip(equals + 1).collect())
        }
        else {
            panic!("Invalid {}: missing assignment token.", ident);
        }
    }

    // Name + body
    fn parse_function_decl(decl: String) -> (String, String) {
        let (skip_len, sig) = utils::get_block(decl.clone());
        let decl = decl.chars().skip(skip_len).collect();
        let (_, body) = utils::get_block(decl);
        
        // get name
        let paren = sig.find("(");
        if paren.is_none() {
            panic!("Invalid function signature.");
        }
        let paren = paren.unwrap();
        let fn_token = sig.find("fn");
        if fn_token.is_none() {
            panic!("Invalid function signature: no fn token.");
        }
        let fn_token = fn_token.unwrap();
        let paren = paren - fn_token - 2;
        let name = sig.chars().skip(fn_token + 2).take(paren).collect();
        let name = utils::remove_preceding_whitespace(name);
        (name, body)
    }
    
    fn parse_macro_syntax(_macro: String) -> Self {
        let macro_args = utils::extract_internal_block(_macro);
        let paren = macro_args.find("(");
        if paren.is_none() {
            panic!("Unable to parse macro ident.");
        }
        let paren = paren.unwrap();
        let qualified = macro_args.chars().take(paren).collect();
        let unq = utils::get_unqualified_name(qualified);
        if unq != "acmd_func" {
            println!("{}", unq);
            panic!("Wrong macro invokation.");
        }
        let unpacked = utils::extract_internal_block(macro_args);
        let args: Vec<String> = unpacked.split(',').map(|x| utils::remove_preceding_whitespace(x.to_owned())).collect();
        assert!(args.len() == 4, "Wrong number of macro arguments.");
        for string in args.iter() {
            println!("{}", string);
        }
        let category = AcmdMove::parse_attribute(args.get(0).unwrap().clone(), "battle_object_category");
        let kind = AcmdMove::parse_attribute(args.get(1).unwrap().clone(), "battle_object_kind");
        let animation = AcmdMove::parse_attribute(args.get(2).unwrap().clone(), "animation");
        let animcmd = AcmdMove::parse_attribute(args.get(3).unwrap().clone(), "animcmd");
        Self {
            category: category,
            kind: kind,
            animation: animation,
            animcmd: animcmd,
            name: String::new(),
            body: String::new()
        }
    }

    pub fn new(input: String) -> Self {
        let (skip_len, _macro) = utils::get_block(input.clone());
        let input = input.chars().skip(skip_len).collect();
        let mut ret = AcmdMove::parse_macro_syntax(_macro);
        let (name, body) = AcmdMove::parse_function_decl(input);
        ret.name = name;
        ret.body = body;
        ret
    }
}