use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::{BorrowMut, Cow};
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fs::{self, File};
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::rc::Weak;
use std::thread;
use strum::{EnumIter, IntoEnumIterator};
use typed_builder::TypedBuilder;

use crate::MetaMap;
use crate::module::{Module, ModuleBuilder};


// 'a approaches 'static
//
#[derive(TypedBuilder, Debug)]
pub(crate) struct Namespace<'a> {
    #[builder]
    metadata: MetaMap<'a>,
    #[builder]
    modules: Vec<Cow<'a, Module<'a>>>,
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
            self.modules.push(Cow::Borrowed(new_module));
            Box::leak(Box::new(self.interner.borrow_mut().take())).commit_module(&new_module);
        }
        Ok(())
    }
    fn try_commit(&mut self, commits: &'a Vec<(Module, Vec<FunkData<'a>>)>) -> anyhow::Result<()> {
        use std::ops::Deref;
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

            if !self
                .interner
                .borrow()
                .is_name_available(Some(module_name.as_ref()), None, None)
            {
                errors.push(anyhow!(
                    "Module {0}: already defined!",
                    module_name.as_ref()
                ));
                continue;
            }

            for funkdata in submissions {
                let identity = funkdata.get_name().unwrap().to_owned();
                if !self.interner.borrow().is_name_available(
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
                    &self
                        .interner
                        .borrow_mut()
                        .to_owned()
                        .get_mut()
                        .commit_member_level(
                            /* module_name = */ Cow::Borrowed(module_name.as_ref()),
                            /* identity = */ Cow::Borrowed(identity.as_ref()),
                            /* entry = */ FunkData::nil,
                        )?;
                }
            }
        }
        Ok(())
    }
}
