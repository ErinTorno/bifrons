use std::{sync::atomic::{AtomicU64, Ordering}};

use mlua::prelude::*;
use rand::{prelude::*, distributions::uniform::SampleUniform};
use rand_chacha::ChaCha8Rng;

use super::LuaMod;
static NEXT_SEED: AtomicU64 = AtomicU64::new(2305843009213693951);
pub fn with_rng<F, R>(f: F) -> R where F: Fn(&mut ChaCha8Rng) -> R {
    let seed = NEXT_SEED.load(Ordering::Relaxed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let _ = NEXT_SEED.compare_exchange(seed, rng.next_u64(), Ordering::AcqRel, Ordering::Relaxed);
    f(&mut rng)
}
pub fn set_seed(next_seed: u64) {
    let seed = NEXT_SEED.load(Ordering::Relaxed);
    let _ = NEXT_SEED.compare_exchange(seed, next_seed, Ordering::AcqRel, Ordering::Relaxed);
}
pub fn random_range<I>(min: I, max: I) -> I  where I: Copy + SampleUniform + PartialOrd {
    return with_rng(|r| r.gen_range(min..=max));
}

#[derive(Default)]
pub struct RandomAPI;
impl LuaMod for RandomAPI {
    fn mod_name() -> &'static str { "Random" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("bool", lua.create_function(|_lua, ()| {
            Ok(with_rng(|r| r.next_u64() % 2 == 0))
        })?)?;
        table.set("int", lua.create_function(|_lua, (min, max)| {
            Ok(random_range::<i64>(min, max))
        })?)?;
        table.set("key", lua.create_function(|_lua, table: LuaTable| {
            match table.len()? {
                l if l <= 0 => Ok(None),
                table_len => {
                    let idx = with_rng(|r| r.gen_range(0..table_len as usize));
                    let (key, _) = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                    Ok(Some(key))
                },
            }
        })?)?;
        table.set("kv", lua.create_function(|_lua, table: LuaTable| {
            match table.len()? {
                l if l <= 0 => Ok(None),
                table_len => {
                    let idx = with_rng(|r| r.gen_range(0..table_len as usize));
                    let pair = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                    let mut res = LuaMultiValue::new();
                    res.push_front(pair.1);
                    res.push_front(pair.0);
                    Ok(Some(res.into_vec()))
                },
            }
        })?)?;
        table.set("number", lua.create_function(|_lua, (min, max)| {
            if let Some(min) = min {
                if let Some(max) = max {
                    return Ok(random_range::<f64>(min, max));
                }
            }
            Ok(with_rng(|r| r.gen_range(0.0..1.0)))
        })?)?;
        table.set("value", lua.create_function(|_lua, table: LuaTable| {
            match table.len()? {
                l if l <= 0 => Ok(None),
                table_len => {
                    let idx = with_rng(|r| r.gen_range(0..table_len as usize));
                    let (_, value) = table.pairs::<LuaValue, LuaValue>().nth(idx).unwrap()?;
                    Ok(Some(value))
                },
            }
        })?)?;
        Ok(())
    }
}