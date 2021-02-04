use crate::{ utils, processing, syntax, acmd };

pub struct LuaMove {
    pub agent: String,
    pub script: String,
    pub name: String,
    pub block: String
}

impl LuaMove {
    fn convert_acmd_move(other: acmd::AcmdMove) -> Self {
        let kind_str = other.kind.find("KIND_");
        if kind_str.is_none() {
            panic!("Cannot extract agent kind from: {}", other.kind);
        }
        let kind_str = kind_str.unwrap();
        let kind = other.kind.chars().skip(kind_str + "KIND_".len()).collect::<String>().to_lowercase();
        let internal_block = utils::extract_internal_block(other.body);
        let internal_block = utils::remove_comments(internal_block);
        let mut block = syntax::handle_rust_syntax(internal_block);
        block = processing::handle_indents(block);
        block = processing::handle_hash(block);
        block = processing::handle_lua_void(block);
        block = processing::handle_module_accessor(block);
        block = format!("    let lua_state = fighter.lua_state_agent;\n    let boma = sv_system::battle_object_module_accessor(lua_state);\n    {}", block);
        Self {
            agent: kind,
            script: other.animcmd,
            name: other.name,
            block: block
        }
    }
}

impl From<acmd::AcmdMove> for LuaMove {
    fn from(other: acmd::AcmdMove) -> Self {
        LuaMove::convert_acmd_move(other)
    }
}

impl std::fmt::Display for LuaMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agent: {}, Script: {}\n", self.agent, self.script)?;
        write!(f, "Name: {}\nBody:\n{}", self.name, self.block)
    }
}