use crate::utils;

pub fn find_first(text: String, to_find: &[char]) -> Option<(usize, char)> {
    let mut chars = text.chars();
    let mut count = 0;
    let mut quote = crate::utils::Quote::new();
    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();
        quote.update(ch);
        if quote.state == crate::utils::QuoteState::None {
            for c in to_find.iter() {
                if *c == ch {
                    return Some((count, ch)); 
                }
            }
        }
        count += 1;
    }
    None
}

pub fn get_indent_str(level: i32) -> String {
    let mut output = Vec::new();
    for _ in 0..level {
        output.extend_from_slice(&[' ', ' ', ' ', ' ']);
    }
    output.into_iter().collect()
}

pub fn handle_indents(block: String) -> String {
    let mut output = Vec::new();
    let mut workable = block;
    let mut chars = workable.chars();
    // remove new lines
    let mut is_indent = true;
    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();
        if is_indent {
            if !ch.is_whitespace() {
                is_indent = false;
                output.push(ch);
            }
        }
        else {
            if ch == '\n' || ch == '\r' {
                is_indent = true;
            }
            else {
                output.push(ch);
            }
        }
    }
    workable = output.into_iter().collect();
    let mut output = Vec::new();

    let mut depth = 1;
    let search = ['{', ';', '}'];
    let mut cur_str = workable.clone();
    loop {

        let found = find_first(cur_str.clone(), &search);
        if found.is_none() {
            break;
        }
        let (loc, ch) = found.unwrap();
        output.extend_from_slice(cur_str.chars().take(loc + 1).collect::<String>().as_bytes());
        cur_str = cur_str.chars().skip(loc + 1).collect();
        match ch {
            '{' => {
                let workable = format!("{{{}", cur_str);
                let string = utils::extract_internal_block(workable.clone());
                if string.contains(";") {
                    output.push('\n' as u8);
                    depth += 1;
                }
                else {
                    let (count, _) = utils::get_block(workable.clone());
                    output.push(' ' as u8);
                    output.extend_from_slice(cur_str.chars().take(count - 2).collect::<String>().as_bytes());
                    output.push(' ' as u8);
                    output.push('}' as u8);
                    cur_str = cur_str.chars().skip(count - 1).collect();
                    continue;
                }
            },
            ';' => {
                output.push('\n' as u8);
            },
            '}' => {
                for _ in 0..5 {
                    output.pop();
                }
                output.push('}' as u8);
                output.push('\n' as u8);
                depth -= 1;
            },
            _ => {
                panic!("Invalid character found.");
            }
        }
        output.extend_from_slice(get_indent_str(depth).as_bytes());
    }

    String::from_utf8(output).unwrap()
}

pub fn handle_hash(block: String) -> String {
    let block = block.replace("*WorkModule", "WorkModule");
    block.replace("*Hash40", "Hash40")
}

pub fn handle_lua_void(block: String) -> String {
    let mut cur_str = block.clone();
    let mut output = Vec::new();
    loop {
        if let Some(offset) = cur_str.find("*LUA_VOID") {
            output.extend_from_slice(cur_str.chars().take(offset).collect::<String>().as_bytes());
            cur_str = cur_str.chars().skip(offset + "*LUA_VOID".len()).collect();
            output.extend_from_slice("None".as_bytes());
        }
        else if let Some(offset) = cur_str.find("LUA_VOID") {
            output.extend_from_slice(cur_str.chars().take(offset).collect::<String>().as_bytes());
            cur_str = cur_str.chars().skip(offset + "LUA_VOID".len()).collect();
            output.extend_from_slice("None".as_bytes());
        }
        else {
            output.extend_from_slice(cur_str.as_bytes());
            break;
        }
    }
    String::from_utf8(output).unwrap()
}

pub fn handle_module_accessor(block: String) -> String {
    let mut cur_str = block.clone();
    let mut output = Vec::new();
    loop {
        if let Some(offset) = cur_str.find("module_accessor") {
            output.extend_from_slice(cur_str.chars().take(offset).collect::<String>().as_bytes());
            cur_str = cur_str.chars().skip(offset + "module_accessor".len()).collect();
            output.extend_from_slice("boma".as_bytes());
        }
        else {
            output.extend_from_slice(cur_str.as_bytes());
            break;
        }
    }
    String::from_utf8(output).unwrap()
}