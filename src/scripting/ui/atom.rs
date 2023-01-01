use std::{fmt::Display, collections::HashMap, mem};

use bevy::prelude::{Resource, warn};
use egui::text::LayoutJob;
use mlua::prelude::*;

use crate::{data::{lua::{LuaWorld, TransVar, ScriptVar}, palette::DynColor}, scripting::{LuaMod, color::RgbaColor}};

use super::{text::TextBuilder, elem::TextInst};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum JobCacheKey {
    Atom(usize),
    TextInst(u64),
}

#[derive(Debug, Resource, Default)]
pub struct LuaAtomRegistry {
    pub atoms:           Vec<LuaAtom>,
    pub layoutjob_cache: HashMap<JobCacheKey, LayoutJob>,
}
impl LuaAtomRegistry {
    /// Views the current atom state, acknowledging changes as viewed if it isn't already
    pub fn _acknowledge(&mut self, atom_ref: LuaAtomRef) -> TransVar {
        let atom = &mut self.atoms[atom_ref.index];
        atom.acknowledged = true;
        atom.last_eval.clone()
    }

    /// Gets either the value, or if it's an atom, looks up that value and acknowledges its changes
    pub fn acknowledge_or_else<T, E, F>(&mut self, or_atom: OrAtom<T>, f: F) -> T where T: Clone + TryFrom<TransVar, Error=E>, F: FnOnce() -> T, E: Display {
        match or_atom {
            OrAtom::Atom(a) => {
                let atom = &mut self.atoms[a.index];
                let already_acked = atom.acknowledged;
                atom.acknowledged = true;
                match T::try_from(atom.last_eval.clone()) {
                    Ok(t) => t,
                    Err(e) => {
                        if !already_acked {
                            warn!("atom#{} TransVar failed conversion upon acknowledgement {}", a.index, e);
                        }
                        f()
                    },
                }
            },
            OrAtom::Val(v) => v,
        }
    }

    /// Gets either the value, or if it's an atom, looks up that value and acknowledges its changes
    pub fn acknowledge_option<T, E>(&mut self, or_atom: OrAtom<Option<T>>) -> Option<T> where T: Clone + TryFrom<TransVar, Error=E>, E: Display {
        match or_atom {
            OrAtom::Atom(a) => {
                let atom = &mut self.atoms[a.index];
                let already_acked = atom.acknowledged;
                atom.acknowledged = true;
                if let TransVar::Var(ScriptVar::Nil) = atom.last_eval {
                    None
                } else {
                    T::try_from(atom.last_eval.clone()).map_err(|e| {
                        if !already_acked {
                            warn!("atom#{} TransVar failed conversion upon acknowledgement {}", a.index, e);
                        }
                    }).ok()
                }
            },
            OrAtom::Val(v) => v,
        }
    }

    pub fn acknowledge_layout_job<F, C>(&mut self, or_atom: &OrAtom<TextInst>, eval_color: C, or_else: F) -> LayoutJob where F: FnOnce() -> String, C: FnMut(&DynColor) -> RgbaColor {
        match or_atom {
            OrAtom::Atom(a) => {
                let atom = &mut self.atoms[a.index];
                if !atom.acknowledged {
                    let builder = match TextBuilder::try_from(atom.last_eval.clone()) {
                        Ok(t) => t,
                        Err(e) => {
                            warn!("atom#{} TransVar failed to convert to TextBuilder {}", a.index, e);
                            TextBuilder::plain(or_else())
                        },
                    };
                    let job = builder.to_layout_job(eval_color);
                    self.layoutjob_cache.insert(JobCacheKey::Atom(a.index), job.clone());
                    atom.acknowledged = true;
                    job
                } else {
                    self.layoutjob_cache.get(&JobCacheKey::Atom(a.index)).unwrap().clone()
                }

            },
            OrAtom::Val(v) => self.layoutjob_cache.entry(JobCacheKey::TextInst(v.id))
                .or_insert_with(|| v.builder.clone().to_layout_job(eval_color))
                .clone(),
        }
    }

    /// Views the current atom state without acknowledging it
    pub fn _peek(&mut self, atom_ref: LuaAtomRef) -> TransVar {
        self.atoms[atom_ref.index].last_eval.clone()
    }

    pub fn set<V>(&mut self, atom_ref: LuaAtomRef, v: V) where V: Into<TransVar> {
        let trans_var: TransVar = v.into();
        let atom = &mut self.atoms[atom_ref.index];
        atom.acknowledged = false;
        atom.is_last_rust_eval = true;
        atom.last_eval = trans_var.clone();
    }
}

#[derive(Debug)]
pub struct LuaAtom {
    /// key to actual lua value in the registry
    key:          LuaRegistryKey,
    /// value it was at last evaluation as a TransVar
    last_eval:    TransVar,
    /// bool marking if the last change to this has been acknowledged (and not just read)
    acknowledged: bool,
    /// bool marking if the last_eval was updated rust side. If so, we'll update the lua value too.
    is_last_rust_eval: bool,
}
impl LuaMod for LuaAtom {
    fn mod_name() -> &'static str { "Atom" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("create", lua.create_function(|lua, val: LuaValue| {
            let key          = lua.create_registry_value(val.clone())?;
            let last_eval    = TransVar::from_lua(val, lua)?;
            let acknowledged = false;

            let world   = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w   = world.write();
            let mut reg = w.resource_mut::<LuaAtomRegistry>();
            let index   = reg.atoms.len();
            reg.atoms.push(LuaAtom { key, last_eval, acknowledged, is_last_rust_eval: false });
            Ok(LuaAtomRef { index })
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LuaAtomRef {
    index: usize,
}
impl LuaAtomRef {
    pub fn index(&self) -> usize { self.index }

    pub fn get<'a>(&self, lua: &'a Lua) -> Result<LuaValue<'a>, mlua::Error> {
        let world   = lua.globals().get::<_, LuaWorld>("world").unwrap();
        let mut w   = world.write();
        let mut reg = w.resource_mut::<LuaAtomRegistry>();

        let should_update = reg.atoms[self.index].is_last_rust_eval;
        if should_update {
            reg.atoms[self.index].is_last_rust_eval = false;
            let key = &reg.atoms[self.index].key;
            lua.replace_registry_value(key, reg.atoms[self.index].last_eval.clone())?;
            reg.atoms[self.index].last_eval.clone().to_lua(lua)
        } else {
            lua.registry_value(&reg.atoms[self.index].key)
        }
    }

    pub fn set<'a>(&self, lua: &'a Lua, val: LuaValue) -> Result<LuaValue<'a>, mlua::Error> {
        let world     = lua.globals().get::<_, LuaWorld>("world").unwrap();
        let mut w     = world.write();
        let mut reg   = w.resource_mut::<LuaAtomRegistry>();
        let last_eval = mem::replace(&mut reg.atoms[self.index].last_eval, TransVar::from_lua(val, lua)?);
        
        reg.atoms[self.index].acknowledged = false;
        reg.atoms[self.index].is_last_rust_eval = false;
        last_eval.to_lua(lua)
    }

    pub fn update(&self, lua: &Lua, f: LuaFunction) -> Result<(), mlua::Error> {
        let world   = lua.globals().get::<_, LuaWorld>("world").unwrap();
        let mut w   = world.write();
        let mut reg = w.resource_mut::<LuaAtomRegistry>();

        let key = &reg.atoms[self.index].key;
        let val: LuaValue = if reg.atoms[self.index].is_last_rust_eval {
            reg.atoms[self.index].last_eval.clone().to_lua(lua)?
        } else { lua.registry_value(key)? };
        let val: LuaValue = f.call(val)?;
        lua.replace_registry_value(key, val.clone())?;
        reg.atoms[self.index].last_eval = TransVar::from_lua(val, lua)?;
        reg.atoms[self.index].acknowledged = false;
        reg.atoms[self.index].is_last_rust_eval = false;
        Ok(())
    }
}
impl LuaUserData for LuaAtomRef {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("index", |_, this| Ok(this.index));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Call, |lua, this, ()| this.get(lua));
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: LuaAtomRef| Ok(this.index == that.index));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("AtomRef#{}", this.index)));

        methods.add_method("get", |lua, this, ()|  this.get(lua));
        methods.add_method("map", |lua, this, f|   this.update(lua, f));
        methods.add_method("set", |lua, this, val| this.set(lua, val));
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum OrAtom<T> {
    Atom(LuaAtomRef),
    Val(T),
}
impl<T> OrAtom<T> {
    pub fn map<F, R>(self, f: F) -> OrAtom<R> where F: FnOnce(T) -> R {
        match self {
            OrAtom::Atom(r) => OrAtom::Atom(r),
            OrAtom::Val(t)  => OrAtom::Val(f(t)),
        }
    }
}
impl<'lua, T> FromLua<'lua> for OrAtom<T> where T: FromLua<'lua> {
    fn from_lua(v: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Some(b) = LuaAtomRef::from_lua(v.clone(), lua).ok() {
            Ok(OrAtom::Atom(b))
        } else if let Some(a) = T::from_lua(v.clone(), lua).ok() {
            Ok(OrAtom::Val(a))
        } else {
            Err(LuaError::RuntimeError(format!("Failed OrAtom conversion")))
        }
    }
}