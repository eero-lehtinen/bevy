use crate::io::{memory::Value, AssetSourceEvent};
use alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc};
use bevy_ecs::{resource::Resource, world::World};
use bevy_platform::collections::HashMap;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};

use super::{EmbeddedAssetRegistry, EmbeddedWatcher};

type AssetUpdateFn = dyn Fn(&mut World, String, String) + Send + Sync;

#[derive(Resource)]
pub struct InternalAssetWatcher {
    registry: EmbeddedAssetRegistry,
    receiver: crossbeam_channel::Receiver<AssetSourceEvent>,
    asset_updaters: Arc<RwLock<HashMap<PathBuf, Arc<AssetUpdateFn>>>>,
    _watcher: EmbeddedWatcher,
}

impl Default for InternalAssetWatcher {
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
            receiver,
            asset_updaters: Default::default(),
            _watcher: watcher,
        }
    }
}

impl InternalAssetWatcher {
    pub fn get_changed(&self) -> Option<(String, String, Arc<AssetUpdateFn>)> {
        let iter = self.receiver.try_iter();
        for event in iter {
            if let AssetSourceEvent::ModifiedAsset(path) = event {
                if let Some(updater) = self.asset_updaters.read().get(&path).cloned() {
                    if let Some(data) = self.registry.dir.get_asset(&path) {
                        return Some((
                            data.path().to_string_lossy().into_owned(),
                            data.value_to_string(),
                            updater,
                        ));
                    }
                }
            }
        }
        None
    }

    pub fn insert(
        &self,
        full_path: PathBuf,
        asset_path: &Path,
        value: impl Into<Value>,
        updater: Box<AssetUpdateFn>,
    ) {
        self.registry.insert_asset(full_path, asset_path, value);
        self.asset_updaters
            .write()
            .insert(asset_path.to_owned(), updater.into());
    }
}

pub fn update(world: &mut World) {
    loop {
        let Some((path, content, updater)) = world.resource::<InternalAssetWatcher>().get_changed()
        else {
            break;
        };
        updater(world, path, content);
    }
}
