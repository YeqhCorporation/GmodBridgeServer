pub mod gabp;

use gmod::{gmod13_close, gmod13_open, lua::State};

#[gmod13_open]
unsafe fn gmod13_open(lua: State) -> i32 {
    gabp::runtime::install_lua_api(lua);
    eprintln!("[gmod-gabp] module loaded");
    0
}

#[gmod13_close]
unsafe fn gmod13_close(_lua: State) -> i32 {
    gabp::runtime::shutdown_global_runtime();
    eprintln!("[gmod-gabp] module unloaded");
    0
}
