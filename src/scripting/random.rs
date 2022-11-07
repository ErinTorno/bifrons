use ::std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy_mod_scripting::{prelude::*};
use mlua::Lua;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[derive(Default)]
pub struct RandomAPIProvider;

impl APIProvider for RandomAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_random_lua(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

static NEXT_SEED: AtomicU64 = AtomicU64::new(2305843009213693951);
fn with_rng<F, R>(f: F) -> Result<R, LuaError> where F: Fn(&mut ChaCha8Rng) -> R {
    let seed = NEXT_SEED.load(Ordering::Relaxed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let _ = NEXT_SEED.compare_exchange(seed, rng.next_u64(), Ordering::AcqRel, Ordering::Relaxed);
    Ok(f(&mut rng))
}

fn attach_random_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("bool", ctx.create_function(|_ctx, ()| {
        with_rng(|r| r.next_u64() % 2 == 0)
    })?)?;
    table.set("int", ctx.create_function(|_ctx, (min, max)| {
        if let Some(min) = min {
            if let Some(max) = max {
                return with_rng(|r| r.gen_range(min..=max));
            }
        }
        with_rng(|r| r.next_u32())
    })?)?;
    table.set("key", ctx.create_function(|_ctx, table: LuaTable| {
        match table.len()? {
            l if l <= 0 => Ok(None),
            table_len => {
                let idx = with_rng(|r| r.gen_range(0..table_len as usize))?;
                let (key, _) = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                Ok(Some(key))
            },
        }
    })?)?;
    table.set("kv", ctx.create_function(|_ctx, table: LuaTable| {
        match table.len()? {
            l if l <= 0 => Ok(None),
            table_len => {
                let idx = with_rng(|r| r.gen_range(0..table_len as usize))?;
                let pair = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                let mut res = LuaMultiValue::new();
                res.push_front(pair.1);
                res.push_front(pair.0);
                Ok(Some(res.into_vec()))
            },
        }
    })?)?;
    table.set("number", ctx.create_function(|_ctx, (min, max)| {
        if let Some(min) = min {
            if let Some(max) = max {
                return with_rng(|r| r.gen_range(min..=max));
            }
        }
        with_rng(|r| r.gen_range(0.0..1.0))
    })?)?;
    table.set("value", ctx.create_function(|_ctx, table: LuaTable| {
        match table.len()? {
            l if l <= 0 => Ok(None),
            table_len => {
                let idx = with_rng(|r| r.gen_range(0..table_len as usize))?;
                let (_, value) = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                Ok(Some(value))
            },
        }
    })?)?;
    ctx.globals().set("Random", table)?;
    Ok(())
}