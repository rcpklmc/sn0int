use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;
use crate::json;


pub fn json_decode(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("json_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        json::decode(&x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_encode(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("json_encode", hlua::function1(move |x: AnyLuaValue| -> Result<String> {
        json::encode(x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_decode_stream(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("json_decode_stream", hlua::function1(move |x: String| -> Result<Vec<AnyLuaValue>> {
        json::decode_stream(&x)
            .map_err(|err| state.set_error(err))
    }))
}
