#![allow(unused_imports, non_snake_case, non_camel_case_types, dead_code)]
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
use strum::{EnumIter, IntoEnumIterator};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone)]
struct Module<'a> {
    name: Cow<'a, str>,
    interner: Rc<RefCell<Interner<'a>>>,
}

impl<'a> Module<'a> {
    #[doc(hidden)]
    pub fn new(name: Cow<'a, str>, interner: Rc<RefCell<Interner<'a>>>) -> Self {
        Self { name, interner }
    }
    #[doc(hidden)]
    pub fn builder() -> ModuleBuilder<'a> {
        ModuleBuilder::new()
    }
    pub fn get_name(&self) -> &Cow<'a, str> {
        &self.name
    }
    #[allow(unreachable_code)]
    fn add_type(&mut self, _type: FunkTy<'a>) -> anyhow::Result<()> {
        // Inspect the interner for the presented `r#type: FunkTy<'a>`
        // metadata. If an associated `InternerEntry` is found, we
        // yeet an error back to the caller.
        bail!("`add_type` Not implemented");
        todo!("Define an `InternerEntry` that can be stored and later retrieved");
        todo!("Encode `r#type`'s metadata as a bytestream.");
        todo!("Commit the new metadata into the Interner");
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
struct ModuleBuilder<'a> {
    name: Option<Cow<'a, str>>,
    interner: Option<Rc<RefCell<Interner<'a>>>>,
}

#[doc(hidden)]
impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            interner: None,
        }
    }
    fn build(self) -> Module<'a> {
        let Self { name, interner } = self;
        let name = name.unwrap_or(Cow::from("default"));
        let interner = {
            if let Some(thing) = interner {
                thing
            } else {
                Rc::new(RefCell::new(Interner::new()))
            }
        };
        Module::new(name, interner)
    }
    fn name<T: Into<Cow<'a, str>>>(self, new_name: T) -> Self {
        Self {
            name: Some(new_name.into()),
            interner: self.interner,
        }
    }
    fn interner(self, new_interner: Rc<RefCell<Interner<'a>>>) -> Self {
        Self {
            name: self.name,
            interner: Some(new_interner),
        }
    }
}

#[derive(TypedBuilder, Debug)]
struct Namespace<'a> {
    #[builder]
    interner: Rc<RefCell<Interner<'a>>>,
    #[builder]
    modules: Vec<&'a mut Module<'a>>,
}

impl<'a> Namespace<'a> {
    fn register_module(&mut self, new_module: &'a mut Module<'a>) -> anyhow::Result<()> {
        if let Some(_found) = self
            .modules
            .iter()
            .find(|module| module.get_name() == new_module.get_name())
        {
            bail!("Module registration occurred twice!");
        } else {
            self.modules.push(new_module);
            Ok(())
        }
    }
    fn try_commit(
        &mut self,
        commits: &Vec<(Module<'a>, Vec<FunkData<'a>>)>,
    ) -> anyhow::Result<()> {
        // For each of the modules we want to verify the type submission
        // as a unique entry. If a single type submission cannot be completed,
        // yeet the error back to the caller.
        // 
        // If the type has an external property or link outside of the current 
        // module, check the interner for name resolution.
        let it: Interner<'_> = self.interner.into_inner();
        for commit in commits {
            let (module, submissions): &(Module, Vec<FunkData>) = commit;
            for funkdata in submissions {
                let module_name = Some(module.get_name());
                let type_name = Some(funkdata.get_name().unwrap());
                if it.is_name_available(module_name, type_name, None) {
                    bail!("Do this later");
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Interner<'interner> {
    pub metadata: MetaMap<'interner>,
}

pub type MetaMap<'a> = BTreeMap<
    (
        /* module_name = */ Option<Cow<'a, str>>,
        /*    identity = */ Option<Cow<'a, str>>,
        /*  assignment = */ Option<Cow<'a, str>>,
    ),
    FunkData<'a>,
>;

#[doc(hidden)]
#[derive(Default)]
struct Key<'a> {
    r#mod: Option<Cow<'a, str>>,
    identity: Option<Cow<'a, str>>,
    assignment: Option<Cow<'a, str>>,
}

#[doc(hidden)]
macro_rules! key {
    ($($type_field:ident = $type_value:expr),* $(,)?) => {
        Key {
            $(
                $type_field: Some(Cow::Borrowed($type_value)),
            )*
            ..<_>::default()
        }
    }
}

impl<'interner> Interner<'interner> {
    pub fn new() -> Self {
        Interner {
            metadata: MetaMap::new(),
        }
    }
    pub fn is_name_available(
        &self,
        module_name: Option<&str>,
        identity: Option<&str>,
        field: Option<&str>,
    ) -> bool {
        
        // Property and link names shouldn't be disambiguated
        // on a given type.
        // todo!("Check the interner for name availability on a module level, a module::identity level, and a module::identity.field level");
        match (module_name, identity, field) {
            (Some(module), Some(ident), Some(link_or_prop)) => {
                let k = key!{
                    r#mod = module,
                    identity = ident,
                    assignment = link_or_prop,
                };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            (Some(module), Some(ident), None) => {
                let k = key!{
                    r#mod = module,
                    identity = ident,
                };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            (Some(module), None, None) => {
                let k = key!{ r#mod = module };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            _ => {
                panic!("Verify the arguments passed to is_name_available");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FunkData<'interner> {
    primitive(funkstd),
    custom(FunkTy<'interner>),
}

trait Named {
    fn get_name(&self) -> Option<&str> {
        <_>::default()
    }    
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

impl Named for funkstd {
    fn get_name(&self) -> Option<&str> {
        match self {
            Self::bool => Some("bool"),
            Self::int8 => Some("int8"),
            Self::int16 => Some("int16"),
            Self::int32 => Some("int32"),
            Self::int64 => Some("int64"),
            Self::int128 => Some("int128"),
            Self::uint8 => Some("uint8"),
            Self::uint16 => Some("uint16"),
            Self::uint32 => Some("uint32"),
            Self::uint64 => Some("uint64"),
            Self::uint128 => Some("uint128"),
            Self::str => Some("str"),
            Self::bool => Some("bool"),
        }
    }
}

pub type FunkPropMap<'interner> = BTreeMap<
    Cow<'interner, str>,
    (
        /* kind */ funkstd,
        /* required: */ bool,
        /* is_multi: */ bool,
    ),
>;

pub type FunkLinkMap<'interner> = BTreeMap<
    Cow<'interner, str>,
    (
        /* kind */ Rc<FunkTy<'interner>>,
        /* required: */ bool,
        /* is_multi: */ bool,
    ),
>;

#[derive(Debug, Clone, Default)]
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
        self.properties
            .insert(typekey, (property, required, is_multi));
        self
    }

    fn add_multi_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, multiproperty) = prop;
        let required = false;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties
            .insert(typekey, (multiproperty, required, is_multi));
        self
    }

    fn add_required_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, property) = prop;
        let required = true;
        let is_multi = false;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties
            .insert(typekey, (property, required, is_multi));
        self
    }

    fn add_required_multi_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, multiproperty) = prop;
        let required = true;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties
            .insert(typekey, (multiproperty, required, is_multi));
        self
    }

    fn add_multi_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, multilink) = link;
        let required = false;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }

    fn add_link<'interner, T: Into<Cow<'a, str>>>(mut self, link: (T, Rc<FunkTy<'a>>)) -> Self {
        let (linkkey, link) = link;
        let required = false;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (link, required, is_multi));
        self
    }

    fn add_required_multi_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, multilink) = link;
        let required = true;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }

    fn add_required_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, link) = link;
        let required = true;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (link, required, is_multi));
        self
    }
}

impl<'ugh> Named for FunkTy<'ugh> {
    fn get_name(&self) -> Option<&str> {
        let cow = self.type_name.unwrap();
        let name = cow.get(0..).unwrap();
        Some(name)        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_disk_persisted() -> anyhow::Result<()> {
        let path = "tmp.funk";
        let mut db = FunkDb::open(path).expect("it works in testing");
        db.save()?;
        assert!(fs::remove_file(db.path).is_ok());
        Ok(())
    }

    #[test]
    fn create_db_schema_and_apply_it() -> anyhow::Result<()> {
        use std::cell::{RefCell, RefMut};
        use std::rc::Rc;

        // The codegen associated with the schema migration
        // is meant to mirror this default module, assuming
        // that a parser has been fed a file (or user input)
        // containing the given sdl.
        //
        // Note: If this strongly resembles EdgeDB syntax, that's
        // because I'm writing FunkDB to be compatible with it.
        /*
            module default {
                type FunksGiven {
                    required expires: int32;
                    significance: str,
                }
                type ReasonForLiving {
                    required online: bool;
                    multi given: FunksGiven;
                }
            }
        */

        // Preparing the namespace, type interner, builtin types, and
        // creating a way for the interned data to be shared between
        // a module and the namespace.
        let interner = Rc::new(RefCell::new(Interner::new()));

        let mut ns = Namespace::builder()
            .interner(Rc::clone(&interner))
            .modules(vec![])
            .build();

        let funk_std = Module::builder()
            .name("std")
            .interner(Rc::clone(&interner))
            .build();

        let mut mod_funk_std = funk_std.clone();

        ns.register_module(&mut mod_funk_std)?;

        let builtins: Vec<FunkData> = funkstd::iter().map(FunkData::primitive).collect();
        let builtins = vec![(funk_std, builtins)];
        ns.try_commit(&builtins)?;

        // This is where we introduce a user-defined schema
        let default = Module::builder()
            .name("default")
            .interner(Rc::clone(&interner))
            .build();
        let mut mod_default = default.clone();

        ns.register_module(&mut mod_default)?;

        let FunksGiven = {
            let ty = FunkTy::r#type("FunksGiven")
                .add_required_property(("expires", funkstd::int32))
                .add_property(("significance", funkstd::r#str));
            Rc::new(ty)
        };

        let ReasonForLiving = FunkTy::r#type("ReasonForLiving")
            .add_required_property(("online", funkstd::r#bool))
            .add_multi_link(("funks", Rc::clone(&FunksGiven)));

        let FunksGiven = Rc::into_inner(FunksGiven).unwrap();

        let commits = vec![(
            default,
            vec![
                FunkData::custom(FunksGiven),
                FunkData::custom(ReasonForLiving),
            ],
        )];
        ns.try_commit(&commits)?;
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
        todo!("`new_server` Not implemented");
    }
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.stream.is_some() {
            bail!("`save` not implemented!");
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
