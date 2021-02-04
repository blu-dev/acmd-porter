#![feature(iter_advance_by)]
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process;

mod utils;
mod acmd;
mod syntax;
mod processing;
mod lua;
fn main() {
    let mut args = env::args();
    let _ = args.next(); // executable name

    loop {
        let filename = args.next();
        if filename.is_none() {
            break;
        }
        let filename = filename.unwrap();

        let mut file = File::open(&filename).expect("Unable to open file");
    
        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");
    
        // println!("{}", lua::LuaMove::from(acmd::AcmdMove::new(src)));
        // println!("{}", syntax::replace_module_access_call("sv_module_access::on_flag(SOME_FLAG)".to_owned()).unwrap());
    
        let macro_positions = utils::find_macro_positions(src.clone());
        let mut moves = Vec::new();
        for position in macro_positions {
            let input = src.chars().skip(position).collect();
            let _move = acmd::AcmdMove::new(input);
            moves.push(lua::LuaMove::from(_move));
        }
    
        let output = print_to_file(&moves);
        let mut backup_file = File::create(&format!("{}_backup", filename)).unwrap();
        let mut out_file = File::create(&filename).unwrap();
        backup_file.write(src.as_bytes());
        out_file.write(output.as_bytes());
    }
}

static USE_BLOCK: &'static str = r#"// ported using blujay's porter, ya bitch
use smash::app::{sv_system, sv_animcmd::{frame, wait}, self, lua_bind::*};
use smash::phx::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::L2CAgentBase;
use smash::app::utility::*;
use smash_script::*;
use smash_script::macros::*;
use crate::utils::hdr;
use crate::vars::*;
"#;

fn make_macro_decl(_move: &lua::LuaMove) -> String {
    format!("#[script( agent = \"{}\", script = {} )]\n", _move.agent, _move.script)
}

fn make_function(_move: &lua::LuaMove) -> String {
    format!("unsafe fn {}(fighter: &mut L2CAgentBase) {{\n{}\n}}\n\n", _move.name, _move.block)
}

fn make_install_fn(moves: &Vec<lua::LuaMove>) -> String {
    let mut output = Vec::new();

    output.extend_from_slice("pub fn install() {\n    replace_scripts!(\n".as_bytes());
    
    for _move in moves.iter() {
        output.extend_from_slice(format!("        {},\n", _move.name).as_bytes());
    }

    output.extend_from_slice("    );\n}\n\n".as_bytes());

    String::from_utf8(output).unwrap()
}

fn print_to_file(moves: &Vec<lua::LuaMove>) -> String {
    let mut output = Vec::new();
    output.extend_from_slice(format!("{}\n\n", USE_BLOCK).as_bytes());

    for _move in moves.iter() {
        output.extend_from_slice(make_macro_decl(_move).as_bytes());
        output.extend_from_slice(make_function(_move).as_bytes());
    }

    output.extend_from_slice(make_install_fn(moves).as_bytes());

    String::from_utf8(output).unwrap()
}