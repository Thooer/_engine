use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ron::de::from_str;

use super::{AssetError, AssetManager, AssetManagerExt, FileSystem, MeshAsset};

impl<F: FileSystem> AssetManagerExt<F> for AssetManager<F> {
    fn new(fs: F) -> Self {
        Self {
            fs,
            meshes: HashMap::new(),
        }
    }

    fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> Result<Arc<MeshAsset>, AssetError> {
        let path = path.as_ref();
        if let Some(m) = self.meshes.get(path) {
            return Ok(m.clone());
        }

        let text = self.fs.read_to_string(path).map_err(AssetError::Io)?;
        let mesh: MeshAsset =
            from_str(&text).map_err(|e| AssetError::ParseMeshRon(e.into()))?;
        let arc = Arc::new(mesh);
        self.meshes.insert(path.to_path_buf(), arc.clone());
        Ok(arc)
    }
}

