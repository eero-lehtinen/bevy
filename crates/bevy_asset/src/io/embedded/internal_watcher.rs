use crate::io::{
    file::{get_base_path, FileWatcher},
    AssetSourceEvent,
};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    string::String,
    sync::Arc,
};
use bevy_ecs::{resource::Resource, world::World};
use bevy_platform::collections::HashMap;
use parking_lot::RwLock;
use std::{fs::read_to_string, path::PathBuf};

type AssetUpdateFn = dyn Fn(&mut World, String, Cow<'_, str>) + Send + Sync;

#[derive(Resource)]
pub struct InternalAssetWatcher {
    root: PathBuf,
    receiver: crossbeam_channel::Receiver<AssetSourceEvent>,
    _watcher: FileWatcher,
    asset_updaters: Arc<RwLock<HashMap<PathBuf, Arc<AssetUpdateFn>>>>,
}

impl Default for InternalAssetWatcher {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let root_path = get_base_path().canonicalize().unwrap();
        Self {
            root: root_path.clone(),
            receiver,
            _watcher: FileWatcher::new(root_path, sender, core::time::Duration::from_millis(300))
                .unwrap(),
            asset_updaters: Default::default(),
        }
    }
}

impl InternalAssetWatcher {
    pub fn get_changed(&self) -> Option<(PathBuf, Arc<AssetUpdateFn>)> {
        let iter = self.receiver.try_iter();
        for event in iter {
            if let AssetSourceEvent::ModifiedAsset(path) = event {
                if let Some(updater) = self.asset_updaters.read().get(&path).cloned() {
                    return Some((path, updater));
                }
            }
        }
        None
    }

    pub fn add(&self, path: PathBuf, updater: Box<AssetUpdateFn>) {
        let path = self.root.join(path).canonicalize().unwrap();
        let path = path.strip_prefix(&self.root).unwrap().to_owned();
        self.asset_updaters.write().insert(path, updater.into());
    }
}

pub fn update(world: &mut World) {
    loop {
        let Some((path, updater)) = world.resource::<InternalAssetWatcher>().get_changed() else {
            break;
        };
        if let Ok(contents) = read_to_string(&path) {
            updater(world, contents, path.to_string_lossy());
        }
    }
}
