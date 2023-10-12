use crate::module::{Module, ModuleBuilder};
use crate::FunkData;
use crate::MetaMap;
use crate::Named;
use crate::{key, Key};
use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::{BorrowMut, Cow};
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::ops::Deref;
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::rc::Weak;
use std::thread;
use strum::{EnumIter, IntoEnumIterator};
use typed_builder::TypedBuilder;

// 'a approaches 'static
//
#[derive(TypedBuilder)]
pub(crate) struct Namespace<'a> {
    #[builder]
    pub(crate) metadata: MetaMap<'a>,
    #[builder]
    pub(crate) modules: Vec<Module<'a>>,
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
            self.modules.push(*new_module);
            self.commit_module(&new_module);
        }
        Ok(())
    }
    fn try_commit(&mut self, commits: &'a Vec<(Module, Vec<FunkData<'a>>)>) -> anyhow::Result<()> {
        // For each of the modules we want to verify the type submission
        // as a unique entry. If a single type submission cannot be completed,
        // yeet the error back to the caller.
        //
        // If the type has an external property or link outside of the current
        // module, check the interner for name resolution.
        let mut errors = vec![];

        for commit in commits {
            let (module, submissions): &(Module, Vec<FunkData>) = commit;

            // First we need to verify that a given module has not been taken
            // before introspecting the module-level types, abstract types,
            // triggers, mutations, traits, or even properties and links on those types.
            let module_name = module.get_name().to_owned();

            if !self.is_name_available(Some(module_name.as_ref()), None, None) {
                errors.push(anyhow!(
                    "Module {0}: already defined!",
                    module_name.as_ref()
                ));
                continue;
            }

            for funkdata in submissions {
                let identity = funkdata.get_name().unwrap().to_owned();
                if !self.is_name_available(
                    Some(module_name.as_ref()),
                    Some(identity.as_ref()),
                    None,
                ) {
                    errors.push(anyhow!(
                        "Schema level name `{0}::{1}` was defined more than once.",
                        module_name.as_ref(),
                        identity
                    ));
                    continue;
                } else {
                    self.commit_member_level(
                        /* module_name = */ Cow::Borrowed(module_name.as_ref()),
                        /* identity = */ Cow::Borrowed(identity.as_ref()),
                        /* entry = */ FunkData::nil,
                    )?;
                }
            }
        }
        Ok(())
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
                let k = key! {
                    r#mod = module,
                    identity = ident,
                    assignment = link_or_prop,
                };
                self.metadata
                    .get(&(k.r#mod, k.identity, k.assignment))
                    .is_none()
            }
            (Some(module), Some(ident), None) => {
                let k = key! {
                    r#mod = module,
                    identity = ident,
                };
                self.metadata
                    .get(&(k.r#mod, k.identity, k.assignment))
                    .is_none()
            }
            (Some(module), None, None) => {
                let k = key! { r#mod = module };
                self.metadata
                    .get(&(k.r#mod, k.identity, k.assignment))
                    .is_none()
            }
            _ => {
                panic!("Verify the arguments passed to is_name_available");
            }
        }
    }

    fn commit_module(&mut self, module: &Module<'a>) {
        let r#mod = FunkData::nil;
        assert!(self
            .metadata
            .insert((Some(module.module_name.to_owned()), None, None), r#mod)
            .is_none());
    }

    // [`commit_member_level`] allocates the name on the module-level identity
    // in such a way that it guarantees future attempts to bind this name
    // to a repeated type name, a link, property, or backlink cannot happen
    // without presenting the collision attempt to the caller.
    fn commit_member_level(
        &mut self,
        module_name: Cow<'a, str>,
        identity: Cow<'a, str>,
        entry: FunkData<'a>,
    ) -> anyhow::Result<()> {
        // When commiting a new custom type identity, `entry` should be `nil`.
        let memkey = (Some(module_name), Some(identity), None);
        match self.metadata.insert(memkey, entry) {
            Some(collision) => Err(anyhow!(
                "We should not reach this point if `try_commit` logic is correct"
            )),
            None => Ok(()),
        }
    }

    fn commit_member(&mut self, funkey: Option<Key<'a>>, entry: FunkData<'a>) {
        match entry {
            FunkData::primitive(funk_std) => {
                let r#mod = Some(Cow::Owned("std".to_string()));
                let member = Some(Cow::Owned(funk_std.get_name().unwrap().to_string()));
                assert!(self.metadata.insert((r#mod, member, None), entry).is_none());
            }
            FunkData::custom(ref funk_ty) => {
                assert!(funkey.is_some());
                let funkey = funkey.unwrap();
                let r#mod = funkey.r#mod;
                let member = funkey.identity;
                assert!(self
                    .metadata
                    .insert((funk_ty.type_name.clone(), None, None), entry)
                    .is_none());
            }
            FunkData::nil => {
                panic!("Incorrect usage of `commit_member`. Use `commit_module` instead.");
            }
        }
    }
}
