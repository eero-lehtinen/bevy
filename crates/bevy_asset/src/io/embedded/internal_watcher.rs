use crate::io::AssetSourceEvent;
use alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc};
use bevy_ecs::{
    resource::Resource,
    world::{Mut, World},
};
use bevy_platform::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{EmbeddedAssetRegistry, EmbeddedWatcher};

/// Bypasses the [`AssetServer`] and allows us to hot-reload embedded strings directly with an
/// arbitrary update function.
#[derive(Resource)]
pub struct InternalStringWatcher {
    registry: EmbeddedAssetRegistry,
    update_functions: HashMap<PathBuf, Arc<AssetUpdateFn>>,
    receiver: crossbeam_channel::Receiver<AssetSourceEvent>,
    _watcher: EmbeddedWatcher,
}

/// Takes world, asset path, and asset string value as arguments.
type AssetUpdateFn = dyn Fn(&mut World, String, String) + Send + Sync;

impl Default for InternalStringWatcher {
    fn default() -> Self {
        let registry = EmbeddedAssetRegistry::default();
        let (sender, receiver) = crossbeam_channel::unbounded();
        let watcher = EmbeddedWatcher::new(
            registry.dir.clone(),
            registry.root_paths.clone(),
            sender,
            core::time::Duration::from_millis(300),
        );
        Self {
            registry,
            update_functions: Default::default(),
            receiver,
            _watcher: watcher,
        }
    }
}

impl InternalStringWatcher {
    pub fn watch_path(
        &mut self,
        full_path: PathBuf,
        asset_path: &Path,
        updater: Box<AssetUpdateFn>,
    ) {
        // The value can be empty for now as we are only watching for future changes.
        self.registry.insert_asset(full_path, asset_path, &[]);
        self.update_functions
            .insert(asset_path.to_owned(), updater.into());
    }

    fn update(&self, world: &mut World) {
        let iter = self.receiver.try_iter();
        for event in iter {
            if let AssetSourceEvent::ModifiedAsset(path) = event {
                if let Some(update_fn) = self.update_functions.get(&path).cloned() {
                    if let Some(data) = self.registry.dir.get_asset(&path) {
                        update_fn(
                            world,
                            data.path().to_string_lossy().into_owned(),
                            data.value_to_string(),
                        );
                    }
                }
            }
        }
    }
}

/// Run the update function for all modified internal string assets.
pub fn update(world: &mut World) {
    world.resource_scope(|world, watcher: Mut<InternalStringWatcher>| {
        watcher.update(world);
    });
}
