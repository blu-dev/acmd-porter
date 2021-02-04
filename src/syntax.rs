use crate::acmd;
use crate::utils;

pub fn handle_rust_syntax(input: String) -> String {
    let mut output = Vec::new();

    let questionable = ['a', 'c', 'm', 'd', '!'];
    let mut questionable_str = Vec::new();
    let mut chars = input.chars();
    loop {
        let ch = chars.next();
        if ch.is_none() {
            break;
        }
        let ch = ch.unwrap();

        if ch == questionable[questionable_str.len()] {
            questionable_str.push(ch);
            let cur = questionable_str.clone().into_iter().collect::<String>();
            if questionable_str.len() == questionable.len() {
                if cur == questionable.into_iter().collect::<String>() {
                    let (skip_len, acmd_block) = utils::get_block(chars.clone().collect::<String>());
                    chars.advance_by(skip_len + 1).unwrap();
                    let parsed = handle_acmd_syntax(acmd_block, true);
                    output.extend_from_slice(parsed.as_bytes());
                }
                else {
                    output.extend_from_slice(questionable_str.iter().map(|c| *c as u8).collect::<Vec<u8>>().as_slice());
                }
                questionable_str.clear();
            }
        }
        else {
            if !questionable_str.is_empty() {
                output.extend_from_slice(questionable_str.iter().map(|c| *c as u8).collect::<Vec<u8>>().as_slice());
                questionable_str.clear();
            }
            output.push(ch as u8);
        }
    }

    String::from_utf8(output).unwrap()
}

pub fn handle_acmd_syntax(input: String, is_wrapped: bool) -> String {
    let mut internal = input.clone();
    if is_wrapped {
        internal = utils::extract_internal_block(internal);
        internal = utils::extract_internal_block(internal);
    }
    let internal = utils::remove_comments(internal);
    let mut output = Vec::new();

    let mut current_str = internal.clone();

    loop {
        let mut chars = current_str.chars();

        let (count, next_statement) = utils::get_block(chars.clone().collect());
        let next_statement = utils::remove_preceding_punctuation(next_statement);
        let next_statement = utils::remove_preceding_unused(next_statement);
        if next_statement.is_empty() {
            break;
        }
        if let Some(offset) = next_statement.find("rust") {
            if offset == 0 {
                let internal = utils::extract_internal_block(next_statement.clone());
                output.extend_from_slice(handle_rust_syntax(internal).as_bytes());
            }
            else {
                // output.extend_from_slice(/*replace_acmd_statement(next_statement).as_bytes()*/);
            }
        }
        else if let Some(mut offset) = next_statement.find("if") {
            if let Some(else_offset) = next_statement.find("else") {
                if else_offset < offset {
                    offset = else_offset;
                    if offset == 0 {
                        let internal = utils::extract_internal_block(next_statement.clone());
                        output.extend_from_slice(" else {\n".as_bytes());
                        output.extend_from_slice(handle_acmd_syntax(internal, false).as_bytes());
                        output.extend_from_slice("\n}".as_bytes());
                    }
                    else {
                        output.extend_from_slice(replace_acmd_statement(next_statement).as_bytes());
                    }
                    current_str = current_str.chars().skip(count).collect();
                    continue;
                }
            }
            if offset == 0 {
                let internal = utils::extract_internal_block(next_statement.clone());
                let conditional = utils::remove_whitespace(internal.clone());
                output.extend_from_slice("if ".as_bytes());
                if conditional == "is_execute" || conditional == "is_excute" {
                    output.extend_from_slice("is_excute(fighter)".as_bytes());
                }
                else {
                    output.extend_from_slice(internal.as_bytes());
                }
                output.extend_from_slice(" {\n".as_bytes());
                current_str = chars.skip(count).collect();
                let (count, internal) = utils::get_block(current_str.clone());
                let internal = utils::extract_internal_block(internal);
                output.extend_from_slice(handle_acmd_syntax(internal, false).as_bytes());
                output.extend_from_slice("\n}".as_bytes());
                current_str = current_str.chars().skip(count).collect();
                continue;
            }
            else {
                output.extend_from_slice(replace_acmd_statement(next_statement).as_bytes());
            }
        }
        else if let Some(offset) = next_statement.find("else") {
            if offset == 0 {
                let internal = utils::extract_internal_block(next_statement.clone());
                output.extend_from_slice(" else {\n".as_bytes());
                output.extend_from_slice(handle_acmd_syntax(internal, false).as_bytes());
                output.extend_from_slice("\n}".as_bytes());
            }
            else {
                output.extend_from_slice(replace_acmd_statement(next_statement).as_bytes());
            }
        }
        else if let Some(offset) = next_statement.find("for") {
            if offset == 0 {
                let internal = utils::extract_internal_block(next_statement.clone());
                let internal = utils::remove_preceding_whitespace(internal);
                let tokens = internal.split(' ').map(|s| s.to_owned()).collect::<Vec<String>>();
                let iterations = tokens.get(0).unwrap().parse::<i32>().unwrap();
                output.extend_from_slice(format!("for _ in 0..{}", iterations).as_bytes());
                output.extend_from_slice(" {\n".as_bytes());
                current_str = chars.skip(count).collect();
                let (count, internal) = utils::get_block(current_str.clone());
                let internal = utils::extract_internal_block(internal);
                output.extend_from_slice(handle_acmd_syntax(internal, false).as_bytes());
                output.extend_from_slice("\n}".as_bytes());
                current_str = current_str.chars().skip(count).collect();
                continue;
            }
            else {
                output.extend_from_slice(replace_acmd_statement(next_statement).as_bytes());
            }
        }
        else {
            output.extend_from_slice(replace_acmd_statement(next_statement).as_bytes());
        }
        current_str = chars.skip(count).collect();
    }

    String::from_utf8(output).unwrap()
}

pub fn replace_acmd_statement(statement: String) -> String {
    if let Some(result) = replace_lua_macro(statement.clone()) {
        result
    }
    else if let Some(result) = replace_module_access_call(statement.clone()) {
        result
    }
    else {
        replace_lua_bind(statement)
    }
}

pub fn replace_lua_bind(statement: String) -> String {
    let paren = statement.find("(");
    if paren.is_none() {
        panic!("Cannot parse invalid function -- no arguments");
    }
    let paren = paren.unwrap();

    let qualified_name = utils::remove_whitespace(statement.chars().take(paren).collect());
    let qualified_name = utils::remove_preceding_unused(qualified_name);
    let func_block = utils::extract_internal_block(statement);
    make_lua_bind_call(qualified_name, func_block)
}

pub fn replace_module_access_call(statement: String) -> Option<String> {
    let paren = statement.find("(");
    if paren.is_none() {
        panic!("Cannot parse invalid function -- no arguments");
    }
    let paren = paren.unwrap();

    let qualified_name = utils::remove_whitespace(statement.chars().take(paren).collect());
    let qualified_name = utils::remove_preceding_unused(qualified_name);
    if qualified_name.contains("sv_module_access::") || qualified_name.contains("sv_battle_object::") {
        let unq_name = utils::get_unqualified_name(qualified_name);
        let args = utils::extract_internal_block(statement);
        return Some(make_module_access_call(unq_name, args));
    }
    None
}

pub fn replace_lua_macro(statement: String) -> Option<String> {
    let paren = statement.find("(");
    let paren = statement.find("(");
    if paren.is_none() {
        println!("{}", statement);
        panic!("Cannot parse invalid function -- no arguments");
    }
    let paren = paren.unwrap();
    // special case for frame/wait calls
    let qualified_name = utils::remove_whitespace(statement.chars().take(paren).collect());
    let qualified_name = utils::remove_preceding_unused(qualified_name);
    if qualified_name == "frame" || qualified_name == "wait" {
        let func_block = utils::extract_internal_block(statement);
        return Some(make_new_lua_state_call(qualified_name, func_block));
    }
    if !qualified_name.contains("::") {
        let func_block = utils::extract_internal_block(statement);
        return Some(make_new_lua_fighter_call(qualified_name, func_block));
    }
    None
}

pub fn make_lua_bind_call(ident: String, block: String) -> String {
    let block = reformat_args(block);
    if ident == "MotionModule::set_rate" {
        change_motion_module_set_rate(block)
    }
    else if block.is_empty() {
        format!("{}(boma);\n", ident)
    }
    else {
        format!("{}(boma, {});\n", ident, utils::space_separate_args(block))
    }
}

pub fn make_new_lua_state_call(ident: String, block: String) -> String {
    let block = reformat_args(block);
    let f = block.parse::<f32>();
    if f.is_err() {
        format!("{}(lua_state, {});\n", ident, block)
    }
    else {
        format!("{}(lua_state, {:.1});\n", ident, f.unwrap())
    }
        
}

pub fn make_new_lua_fighter_call(ident: String, block: String) -> String {
    let mut block = reformat_args(block);
    if ident == "ATTACK" {
        block = handle_pos2_args(block, 13);
    }
    else if ident == "CATCH" {
        block = handle_pos2_args(block, 7);
    }
    else if ident == "SEARCH" {
        block = handle_pos2_args(block, 8);
    }
    if block.is_empty() {
        format!("{}(fighter);\n", ident)
    }
    else {
        format!("{}(fighter, {});\n", ident, utils::space_separate_args(block))
    }
}

pub fn make_module_access_call(ident: String, block: String) -> String {
    let block = reformat_args(block);
    if block.is_empty() {
        format!("{}!(fighter);\n", ident)
    }
    else {
        format!("{}!(fighter, {});\n", ident, utils::space_separate_args(block))
    }
}

pub fn remove_acmd_idents(block: String) -> String {
    let args = block.split(',').map(|s| s.to_owned()).collect::<Vec<String>>();
    let mut output = Vec::new();
    let mut count = 0;
    for arg in args.iter() {
        count += 1;
        let mut out = arg.clone();
        if let Some(offset) = arg.find("=") {
            out = out.chars().skip(offset + 1).collect();
        }
        output.extend_from_slice(out.as_bytes());
        if count != args.len() {
            output.push(',' as u8);
        }
    }
    String::from_utf8(output).unwrap()
}

pub fn fix_lua_consts(block: String) -> String {
    let args = block.split(',').map(|s| s.to_owned()).collect::<Vec<String>>();
    let mut output = Vec::new();
    let mut count = 0;
    if args.get(0).unwrap().is_empty() {
        return args.get(0).unwrap().clone();
    }
    for arg in args.iter() {
        count += 1;
        let mut out = arg.clone();
        if arg.get(0..1).unwrap().chars().all(|c| c.is_uppercase()) {
            out = format!("*{}", out);
        }
        output.extend_from_slice(out.as_bytes());
        if count != args.len() {
            output.push(',' as u8);
        }
    }
    String::from_utf8(output).unwrap()
}

pub fn fix_hashes(block: String) -> String {
    let args = block.split(',').map(|s| utils::remove_preceding_whitespace(s.to_owned())).collect::<Vec<String>>();
    let mut out_args = Vec::new();
    let mut count = 0;
    for arg in args.iter() {
        count += 1;
        let mut out = arg.clone();
        if arg.contains(" as u64") {
            let internal = arg.split(' ').map(|s| s.to_owned()).collect::<Vec<String>>();
            out = format!("Hash40::new_raw({})", internal.get(0).unwrap());
        }
        else if arg.contains("hash40") {
            out = arg.replace("hash40", "Hash40::new");
        }
        out_args.extend_from_slice(out.as_bytes());
        if count != args.len() {
            out_args.push(',' as u8);
        }
    }
    String::from_utf8(out_args).unwrap()
}

pub fn reformat_args(block: String) -> String {
    let block = remove_acmd_idents(block);
    let block = fix_hashes(block);
    let block = fix_lua_consts(block);
    block
}

pub fn handle_pos2_args(block: String, arg_no: usize) -> String {
    let mut args = block.split(',').map(|s| s.to_owned()).collect::<Vec<String>>();
    for x in (arg_no - 1)..(arg_no + 2) {
        let arg = args.get_mut(x).unwrap();
        if arg != "*LUA_VOID" && arg != "LUA_VOID" {
            *arg = format!("Some({})", arg);
        }
    }
    let mut output = Vec::new();
    let mut count = 0;
    for arg in args.iter() {
        count += 1;
        let mut out = arg.clone();
        output.extend_from_slice(out.as_bytes());
        if count != args.len() {
            output.push(',' as u8);
        }
    }
    String::from_utf8(output).unwrap()
}

pub fn change_motion_module_set_rate(block: String) -> String {
    let args = block.split(',').map(|s| s.to_owned()).collect::<Vec<String>>();
    let f_arg = args.last().unwrap();
    let processed = utils::remove_whitespace(f_arg.clone());
    let f: f32 = processed.parse().unwrap();
    format!("FT_MOTION_RATE(fighter, {:.3});\n", 1.0 / f)
}