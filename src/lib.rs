#![allow(unused_imports)]
use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fs::{self, File};
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use typed_builder::TypedBuilder;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone)]
struct Module<'a> {
    name: Cow<'a, str>,
    arena: Rc<RefCell<Arena<'a>>>,
}

impl<'a> Module<'a> {
    #[doc(hidden)]
    pub fn new(name: Cow<'a, str>, arena: Rc<RefCell<Arena<'a>>>) -> Self {
        Self { name, arena }
    }
    #[doc(hidden)]
    pub fn builder() -> ModuleBuilder<'a> {
        ModuleBuilder::new()
    }
    pub fn get_name(&self) -> Cow<'a, str> {
        self.name.clone()
    }
    #[allow(unreachable_code)]
    fn add_type(&mut self, r#type: FunkTy<'a>) -> anyhow::Result<()> {
        // Inspect the arena for the presented `r#type: FunkTy<'a>` 
        // metadata. If an associated `ArenaEntry` is found, we 
        // yeet an error back to the caller.
        bail!("Not implemented");
        todo!("Define an `ArenaEntry` that can be stored and later retrieved");
        todo!("Encode `r#type`'s metadata as a bytestream.");
        todo!("Commit the new metadata into the Arena");
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
struct ModuleBuilder<'a> {
    name: Option<Cow<'a, str>>,
    arena: Option<Rc<RefCell<Arena<'a>>>>,
}

#[doc(hidden)]
impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            arena: None,
        }
    }
    fn build(mut self) -> Module<'a> {
        let Self { name, arena } = self;
        let name = name.unwrap_or(Cow::from("default"));
        let arena = {
            if let Some(thing) = arena {
                thing
            } else {
                Rc::new(RefCell::new(Arena::new()))
            }
        };
        Module::new(name, arena)
    }
    fn name<T: Into<Cow<'a, str>>>(mut self, new_name: T) -> Self {
        Self {
            name: Some(new_name.into()),
            arena: self.arena,
        }
    }
    fn arena(mut self, new_arena: Rc<RefCell<Arena<'a>>>) -> Self {
        Self {
            name: self.name,
            arena: Some(new_arena),
        }
    }
}

#[derive(TypedBuilder, Debug)]
struct Namespace<'a> {
    #[builder]
    arena: Rc<RefCell<Arena<'a>>>,
    #[builder]
    modules: Vec<&'a mut Module<'a>>,
}

impl<'a> Namespace<'a> {
    fn register_module(&mut self, new_module: &'a mut Module<'a>) -> anyhow::Result<()> {
        if let Some(found) = self
            .modules
            .iter()
            .filter(|ref module| module.get_name() == new_module.get_name())
            .next()
        {
            bail!("Module registration occurred twice!");
        } else {
            self.modules.push(new_module);
            Ok(())
        }
    }
    fn try_commit(&mut self, commits: &Vec<(Module<'a>, Vec<FunkData<'a>>)>) -> anyhow::Result<()> {
        bail!("Not implemented");
    }
}

#[derive(Debug, Clone)]
struct Arena<'arena> {
    metadata: BTreeMap<(
        /* module_name = */ Cow<'arena, str>,
        /* identity = */ Cow<'arena, str>,
        /* assignment = */ Cow<'arena, str>), FunkData<'arena>>,
}

impl<'arena> Arena<'arena> {
    pub fn new() -> Self {
        Arena {
            metadata: BTreeMap::new(),
        }
    }
    pub fn is_name_available(&self, module_name: &str, identity: &str, field: &str) -> bool {
        // Property and link names shouldn't be disambiguated
        // on a given type.
        true  
    }
}

#[derive(Debug, Clone)]
pub enum FunkData<'arena> {
    primitive(funkstd),
    custom(FunkTy<'arena>),
}

#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
pub enum funkstd {
    r#bool,
    int8,
    int16,
    int32,
    int64,
    int128,
    r#str,
    uint8,
    uint16,
    uint32,
    uint64,
    uint128,
}

pub type FunkPropMap<'arena> = BTreeMap::<
    Cow<'arena, str>, 
    (/* kind */ funkstd, /* required: */ bool, /* is_multi: */ bool)>;

pub type FunkLinkMap<'arena> = BTreeMap::<
    Cow<'arena, str>, 
    (/* kind */ Rc<FunkTy<'arena>>, /* required: */ bool, /* is_multi: */ bool)>;

#[derive(Debug, Clone)]
pub struct FunkTy<'a> {
    pub type_name: Option<Cow<'a, str>>,
    pub properties: FunkPropMap<'a>,
    pub links: FunkLinkMap<'a>,
}

impl<'a> FunkTy<'a> {
    pub fn r#type<T: Into<Cow<'a, str>>>(name: T) -> FunkTy<'a> {
        let mut this = FunkTy::default();
        this.type_name = Some(name.into());
        this
    }

    fn add_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, property) = prop;
        let required = false;
        let is_multi = false;
        let typekey: Cow<'a, str> = typekey.into();
        &mut self.properties.insert(typekey, (property, required, is_multi));
        self 
    }

    fn add_multi_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, multiproperty) = prop;
        let required = false;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        &mut self.properties.insert(typekey, (multiproperty, required, is_multi));
        self 
    }

    fn add_required_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, property) = prop;
        let required = true;
        let is_multi = false;
        let typekey: Cow<'a, str> = typekey.into();
        &mut self.properties.insert(typekey, (property, required, is_multi));
        self 
    }


    fn add_required_multi_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, multiproperty) = prop;
        let required = true;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        &mut self.properties.insert(typekey, (multiproperty, required, is_multi));
        self 
    }

    fn add_multi_link<'arena, T: Into<Cow<'a, str>>>(mut self, link: (T, Rc<FunkTy<'a>>)) -> Self {
        let (linkkey, multilink) = link;
        let required = false;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        &mut self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }

    fn add_link<'arena, T: Into<Cow<'a, str>>>(mut self, link: (T, Rc<FunkTy<'a>>)) -> Self {
        let (linkkey, link) = link;
        let required = false;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        &mut self.links.insert(linkkey, (link, required, is_multi));
        self
    }

    fn add_required_multi_link<'arena, T: Into<Cow<'a, str>>>(mut self, link: (T, Rc<FunkTy<'a>>)) -> Self {
        let (linkkey, multilink) = link;
        let required = true;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        &mut self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }

       
       
    fn add_required_link<'arena, T: Into<Cow<'a, str>>>(mut self, link: (T, Rc<FunkTy<'a>>)) -> Self {
        let (linkkey, link) = link;
        let required = true;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        &mut self.links.insert(linkkey, (link, required, is_multi));
        self
    }

       
}

impl<'a> Default for FunkTy<'a> {
    fn default() -> Self {
        Self { 
            type_name: None,
            properties: BTreeMap::new(),
            links: BTreeMap::new(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_disk_persisted() -> anyhow::Result<()> {
        let path = "tmp.funk";
        let mut db = FunkDb::open(&path).expect("it works in testing");
        db.save()?;
        assert!(fs::remove_file(db.path).is_ok());
        Ok(())
    }

    #[test]
    fn create_db_schema_and_apply_it() -> anyhow::Result<()> {
        use std::cell::{RefCell, RefMut};
        use std::rc::Rc;

        let test_schema = r#"
            module default {
                type F____Given {
                    required expires: int32;
                    significance: str,
                }
                type ReasonForLiving {
                    required online: bool;
                    multi f____: F____Given;
                }
            }"#;
        let arena = Rc::new(RefCell::new(Arena::new()));
        let mut ns = Namespace::builder()
            .arena(Rc::clone(&arena))
            .modules(vec![])
            .build();

        let mut funk_std = Module::builder()
            .name("std")
            .arena(Rc::clone(&arena))
            .build();
        let mut mod_funk_std = funk_std.clone();
        &mut ns.register_module(&mut mod_funk_std)?;
        
        let mut default = Module::builder()
            .name("default")
            .arena(Rc::clone(&arena))
            .build();
        let mut mod_default = default.clone();
        &mut ns.register_module(&mut mod_default)?;

        // First we need to commit all of the builtins
        let builtins: Vec<FunkData> = funkstd::iter()
            .map(|ty| FunkData::primitive(ty))
            .collect();
        let builtins = vec![(funk_std, builtins)];
        &mut ns.try_commit(&builtins)?;

        let F____Given = Rc::new(FunkTy::r#type("F____Given")
            .add_required_property(("expires", funkstd::int32))
            .add_property(("significance", funkstd::r#str)));
        
        let ReasonForLiving = FunkTy::r#type("ReasonForLiving")
            .add_required_property(("online", funkstd::r#bool))
            .add_multi_link(("f____", Rc::clone(&F____Given)));

        let commits = vec![
            (
                default, 
                vec![
                    FunkData::custom(Rc::into_inner(F____Given).unwrap()), 
                    FunkData::custom(ReasonForLiving)
                ]
            )
        ];
        &mut ns.try_commit(&commits)?;
        Ok(())
    }

    #[ignore]
    #[test]
    fn persist_schema_transaction_to_disk() {
        todo!("Commit a database transaction to a disk-persisted db");
    }

    #[ignore]
    #[test]
    fn verify_schema_post_transaction() {
        todo!("Retrieve schema from database file, verify it corresponds to sdl transaction");
    }

    #[ignore]
    #[test]
    fn parse_schema_into_memory_model() {
        todo!("Make the simplest schema in an external file, serialize it, apply it.");
    }

    #[ignore]
    #[test]
    fn insert_query() {
        todo!("Create new insert query and apply it");
    }
}

#[allow(dead_code)]
pub struct FunkDb {
    path: PathBuf,
    stream: Option<UnixStream>,
    file: File,
}

impl FunkDb {
    pub fn new<F: IntoRawFd>(path: PathBuf, fileno: Option<F>, file: File) -> Self {
        let stream = match fileno {
            Some(f) => {
                let fd = f.into_raw_fd();
                Some(unsafe { <UnixStream as FromRawFd>::from_raw_fd(fd) })
            }
            None => None,
        };
        Self { path, stream, file }
    }
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = PathBuf::from(path.as_ref());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        Ok(Self::new(path, Option::<UnixStream>::None, file))
    }
    #[allow(dead_code)]
    fn new_server(
        &mut self,
        server_path: impl AsRef<Path>,
        db_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let _sfd = FunkDbServer::bind(server_path, db_path)?;
        todo!("Not implemented");
    }
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.stream.is_some() {
            bail!("Not implemented!");
        }
        self.file.sync_all()?;
        Ok(())
    }
}

pub struct FunkDbServer {}

impl FunkDbServer {
    /// Returns the result of the bind op
    /// which, assuming the socket path wasn't already taken, should be Ok(i32).
    ///
    /// With the unwrapped return value, the caller can assume
    /// that there is a unix domain socket at [`path`] which
    /// is a [`UnixListener`].
    ///
    /// The listener will be used to accept client connections to the database
    /// so that prepared statements can be executed, queries against the
    /// database can be ran, and transactions to update the schema can be made.
    ///
    /// Note that [`bind`]'s argument, [`path`], is distinct from the actual
    /// database file.
    #[allow(dead_code, unused_variables)]
    pub fn bind(server_path: impl AsRef<Path>, db_path: impl AsRef<Path>) -> anyhow::Result<RawFd> {
        let path = server_path.as_ref().to_string_lossy();
        let stream = db_path.as_ref().to_string_lossy();
        let server = UnixListener::bind(db_path)?.set_nonblocking(true);

        bail!("This is not yet implemented");
    }
}
