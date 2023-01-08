// it's not a mod, cause that's a reserved keyword

use std::{collections::{HashMap, HashSet}, path::Path, cmp::Ordering};

use bevy::{asset::{AssetLoader, LoadedAsset}, reflect::TypeUuid, prelude::{Component}};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Deserializer, de, };

use crate::util::{ron_options, Roughly};

use super::lua::ScriptVar;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Version(u64, u64, u64);
impl Default for Version {
    fn default() -> Self { Version(0, 1, 0) }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum VersionBound {
    #[default]
    Any,
    AtLeast(u64),
    Between(u64, u64),
    Exactly(u64),
    Outside(u64, u64),
}
impl VersionBound {
    pub fn is_valid(self, i: u64) -> bool {
        match self {
            VersionBound::Any           => true,
            VersionBound::AtLeast(n)    => i >= n,
            VersionBound::Between(a, b) => a <= i && i <= b,
            VersionBound::Exactly(n)    => i == n,
            VersionBound::Outside(a, b) => i < a && i > b,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum VersionDependency {
    #[default]
    Any,
    Versioned(VersionBound, VersionBound, VersionBound),
}
impl VersionDependency {
    pub fn is_valid(&self, version: Version) -> bool {
        match self {
            VersionDependency::Any => true,
            VersionDependency::Versioned(maj, min, fix) => maj.is_valid(version.0) && min.is_valid(version.1) && fix.is_valid(version.2),
        }
    } 
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModError {
    CircularDependency {
        circular:    String,
        parent:      Option<String>,
    },
    IncompatibleVersion {
        name:        String,
        version:     Version,
        allowed:     VersionDependency,
        required_by: Option<String>,
    },
    MissingDependency {
        name:        String,
        required_by: Option<String>,
    },
}

#[derive(Clone, Component, Debug, PartialEq)]
pub struct LoadEntry {
    pub name: String,
    pub root: String,
}

#[derive(Clone, Component, Debug, Default, Deserialize, PartialEq, Serialize, TypeUuid)]
#[uuid = "6bc87c34-6bf8-4639-8735-84020a293762"]
pub struct Module {
    #[serde(default)]
    pub version:         Version,
    pub priority:        f64,
    #[serde(default)]
    pub dependencies:    HashMap<String, VersionDependency>,
    // #[serde(default)]
    // pub banned:          HashMap<String, VersionDependency>,
    #[serde(default)]
    pub lines:           HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub startup_scripts: Vec<String>,
    #[serde(default)]
    pub permissions:     HashSet<String>,
}
impl Module {
    pub fn is_version_valid(&self, version: VersionDependency) -> bool {
        version.is_valid(self.version)
    }

    /// Sorts a map of all loaded modules, and returns a Vec of a Vec of mod names.
    /// The outer Vec represents waves of mods that can be loaded in parallel, 
    /// and the inner vec are those mod names, sorted by mod priority.
    /// 
    /// todo: Allow mods to move into later waves depending on priority order. Current impl had issues with moving into waves where mods that depend on a high priority one are.
    pub fn sorted_load_order<'a, 'b>(mods: &'b HashMap<String, &'a Module>) -> Result<Vec<Vec<&'a String>>, ModError> where 'b: 'a {
        if mods.is_empty() {
            return Ok(Vec::new());
        }

        #[derive(Clone, Debug, PartialEq)]
        struct Wave<'a> {
            index:    usize,
            name:     &'a String,
            priority: f64,
            requires: Vec<&'a String>,
        }
        impl<'a> Eq for Wave<'a> {}
        impl<'a> Ord for Wave<'a> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let i = self.index.cmp(&other.index);
                if i == Ordering::Equal {
                    let p = Roughly(self.priority).cmp(&Roughly(other.priority));
                    if p == Ordering::Equal {
                        self.name.cmp(other.name)
                    } else { p }
                } else { i }
            }
        }
        impl<'a> PartialOrd for Wave<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        #[derive(Clone, Debug, PartialEq, PartialOrd)]
        enum WaveProgress<'a> { Done(Wave<'a>), Loading }
        let mut waves = HashMap::<String, WaveProgress>::new();

        fn visit<'a>(
            waves:   &mut HashMap<String, WaveProgress<'a>>,
            mods:    &'a HashMap<String, &Module>,
            name:    &'a String,
            parent:  Option<&String>,
            version: Option<VersionDependency>,
        ) -> Result<usize, ModError> {
            match waves.get(name) {
                Some(WaveProgress::Done(i)) => Ok(i.index),
                Some(WaveProgress::Loading)  => Err(ModError::CircularDependency { circular: name.clone(), parent: parent.cloned() }),
                None => {
                    let module = *mods.get(name)
                        .ok_or_else(|| ModError::MissingDependency { name: name.clone(), required_by: parent.cloned() })?;
                    
                    if let Some(version) = version {
                        if !module.is_version_valid(version) {
                            return Err(ModError::IncompatibleVersion { name: name.clone(), version: module.version, allowed: version, required_by: parent.cloned() })
                        }
                    }
                    let mut wave = 0usize;
                    waves.insert(name.clone(), WaveProgress::Loading);
                    for (dep_name, dep_version) in module.dependencies.iter() {
                        wave = wave.max(visit(waves, mods, dep_name, Some(name), Some(*dep_version))?);
                        if let WaveProgress::Done(w) = waves.get_mut(dep_name).unwrap() {
                            w.requires.push(name);
                        } else { unreachable!() }
                    }
                    waves.insert(name.clone(), WaveProgress::Done(Wave {
                        name:     &name,
                        index:    wave + 1,
                        priority: module.priority,
                        requires: Vec::new(),
                    }));
                    Ok(wave + 1)
                },
            }
        }
        for name in mods.keys() {
            visit(&mut waves, mods, name, None, None)?;
        }

        let mut waves_im: IndexMap<usize, Vec<Wave>> = waves.into_iter()
            .fold(IndexMap::new(), |mut acc, (_, wp)| {
                let wave = match wp {
                    WaveProgress::Done(w) => w,
                    _ => { unreachable!() }
                };
                let priorities = acc.entry(wave.index).or_insert_with(Vec::new);
                priorities.push(wave);
                acc
            });
        waves_im.sort_keys();
        for vec in waves_im.values_mut() {
            vec.sort();
        }
        
        Ok(waves_im.into_iter()
            .map(|(_, v)| v.into_iter().map(|w| w.name).collect())
            .collect())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModuleLoader;

impl AssetLoader for ModuleLoader {
    fn extensions(&self) -> &[&str] { &["mod.ron"] }

    fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
            Box::pin(async move {
                let module: Module = ron_options().from_bytes(bytes)?;
                load_context.set_default_asset(LoadedAsset::new(module));
                Ok(())
            })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ModEntry {
    pub file:     String,
    #[serde(default)]
    pub settings: HashMap<String, ScriptVar>,
    #[serde(skip)]
    pub name:     String,
}
impl<'de> Deserialize<'de> for ModEntry {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct ModEntryDe {
            pub file:     String,
            #[serde(default)]
            pub settings: HashMap<String, ScriptVar>,
        }
        let m: ModEntryDe = Deserialize::deserialize(d)?;

        let path = Path::new(&m.file);
        if !path.is_file() && path.extension().map(|e| e.eq_ignore_ascii_case("mod.ron")).unwrap_or(false) {
            return Err(de::Error::custom("ModEntry `file` is not a valid .mod.ron file"))
        }
        let name = Path::new(&m.file)
            .file_name()
            .map(|s| s.to_string_lossy().strip_suffix(".mod.ron").unwrap().to_string())
            .ok_or_else(|| {
                de::Error::custom("ModEntry `file` does not point to a file")
            })?;
        Ok(ModEntry { file: m.file, settings: m.settings, name })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ModList {
    pub entries: Vec<ModEntry>,
}

#[cfg(test)]
mod tests {
    use std::{collections::{HashMap}, default::default};

    use crate::{data::module::{VersionBound, Version, ModError}, util::collections::Singleton};

    use super::{VersionDependency, Module};

    #[test]
    fn version_bound_is_valid() {
        assert!(VersionDependency::Any.is_valid(Version(0, 0, 0)));
        assert!(VersionDependency::Any.is_valid(Version(1, 3, 2)));

        {
            let vs = "0.0.0";
            let vd = VersionDependency::Versioned(VersionBound::Exactly(0), VersionBound::Exactly(0), VersionBound::Exactly(0));

            assert!( vd.is_valid(Version(0, 0, 0)), "{}", vs);
            assert!(!vd.is_valid(Version(1, 3, 0)), "{}", vs);
            assert!(!vd.is_valid(Version(0, 0, 1)), "{}", vs);
        }
        {
            let vs = "0.0.*";
            let vd = VersionDependency::Versioned(VersionBound::Exactly(0), VersionBound::Exactly(0), VersionBound::Any);

            assert!( vd.is_valid(Version(0, 0, 0)), "{}", vs);
            assert!( vd.is_valid(Version(0, 0, 1)), "{}", vs);
            assert!( vd.is_valid(Version(0, 0, 5)), "{}", vs);
            assert!(!vd.is_valid(Version(0, 5, 1)), "{}", vs);
            assert!(!vd.is_valid(Version(1, 3, 0)), "{}", vs);
        }
        {
            let vs = "^1.^6.*";
            let vd = VersionDependency::Versioned(VersionBound::AtLeast(1), VersionBound::AtLeast(6), VersionBound::Any);

            assert!(!vd.is_valid(Version(0, 0, 0)), "{}", vs);
            assert!(!vd.is_valid(Version(0, 5, 1)), "{}", vs);
            assert!(!vd.is_valid(Version(1, 3, 0)), "{}", vs);
            assert!( vd.is_valid(Version(1, 6, 0)), "{}", vs);
            assert!( vd.is_valid(Version(1, 6, 3)), "{}", vs);
            assert!( vd.is_valid(Version(2, 8, 3)), "{}", vs);
        }
        {
            let vs = "2.3-5.*";
            let vd = VersionDependency::Versioned(VersionBound::Exactly(2), VersionBound::Between(3, 5), VersionBound::Any);

            assert!(!vd.is_valid(Version(0, 0, 0)), "{}", vs);
            assert!(!vd.is_valid(Version(1, 6, 7)), "{}", vs);
            assert!( vd.is_valid(Version(2, 3, 0)), "{}", vs);
            assert!( vd.is_valid(Version(2, 5, 7)), "{}", vs);
            assert!(!vd.is_valid(Version(2, 6, 7)), "{}", vs);
            assert!(!vd.is_valid(Version(3, 4, 7)), "{}", vs);
        }
    }

    #[test]
    fn sorted_mod_list() {
        let core_name = "core".to_string();
        let core = Module {
            version: Version(0, 16, 4),
           priority: -99999999.,
           ..default()
        };
        let editor_name = "editor".to_string();
        let editor = Module {
            dependencies: HashMap::singleton(("core".to_string(), VersionDependency::Any)),
            priority: -100.,
            ..default()
        };
        let unofficial_patch_name = "unofficial_patch".to_string();
        let unofficial_patch = Module {
            dependencies: HashMap::singleton(("core".to_string(), VersionDependency::Any)),
            priority: -10.,
            ..default()
        };
        let caked_up_cthulhu_name = "caked_up_cthulhu".to_string();
        let caked_up_cthulhu = Module {
            dependencies: HashMap::singleton(("core".to_string(), VersionDependency::Any)),
            priority: 666.,
            ..default()
        };
        let lore_friendly_fix_name = "lore_friendly_fix".to_string();
        let lore_friendly_fix = Module {
            dependencies: {
                let mut hm = HashMap::new();
                hm.insert("core".to_string(), VersionDependency::Versioned(VersionBound::Exactly(0), VersionBound::AtLeast(3), VersionBound::Any));
                hm.insert("unofficial_patch".to_string(), VersionDependency::Any);
                hm.insert("caked_up_cthulhu".to_string(), VersionDependency::Any);
                hm
            },
            priority: -1000.,
            ..default()
        };
        let crash_simulator = Module {
            dependencies: HashMap::singleton(("crash_simulator".to_string(), VersionDependency::Any)),
            ..default()
        };

        {
            let mut mods: HashMap<String, &Module> = HashMap::new();
            assert_eq!(Ok(vec![]), Module::sorted_load_order(&mods));

            mods.insert(core_name.clone(), &core);
            assert_eq!(
                Ok(vec![vec![&core_name]]),
                Module::sorted_load_order(&mods)
            );
            
            mods.insert(editor_name.clone(), &editor);
            assert_eq!(
                Ok(vec![vec![&core_name], vec![&editor_name]]),
                Module::sorted_load_order(&mods)
            );
            
            mods.insert(lore_friendly_fix_name.clone(), &lore_friendly_fix);
            mods.insert(unofficial_patch_name.clone(), &unofficial_patch);
            assert_eq!(
                Err(ModError::MissingDependency { name: "caked_up_cthulhu".to_string(), required_by: Some("lore_friendly_fix".to_string()) }),
                Module::sorted_load_order(&mods)
            );
            
            mods.insert(caked_up_cthulhu_name.clone(), &caked_up_cthulhu);
            assert_eq!(
                Ok(vec![vec![&core_name], vec![&editor_name, &unofficial_patch_name, &caked_up_cthulhu_name], vec![&lore_friendly_fix_name]]),
                Module::sorted_load_order(&mods)
            );

            mods.insert("crash_simulator".into(), &crash_simulator);
            assert_eq!(
                Err(ModError::CircularDependency { circular: "crash_simulator".into(), parent: Some("crash_simulator".into()) }),
                Module::sorted_load_order(&mods)
            );
        }
    }

}
