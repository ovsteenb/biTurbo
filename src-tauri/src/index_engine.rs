use crate::error::{BiError, BiResult};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use turbovec::IdMapIndex;

pub struct ProjectIndex {
    pub project_id: String,
    pub dim: usize,
    pub bit_width: usize,
    index: Mutex<IdMapIndex>,
    uid_to_extid: Mutex<HashMap<String, u64>>,
    extid_to_uid: Mutex<HashMap<u64, String>>,
    file_path: PathBuf,
    next_extid: Mutex<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub uid: String,
    pub score: f32,
    pub ext_id: u64,
}

impl ProjectIndex {
    pub fn open_or_create(
        project_id: &str,
        dim: usize,
        bit_width: usize,
        data_dir: &Path,
    ) -> BiResult<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let file_path = data_dir.join(format!("{project_id}.tvim"));

        let (index, uid_to_extid, extid_to_uid, next_extid) = if file_path.exists() {
            let idx = IdMapIndex::load(&file_path)
                .map_err(|e| BiError::Index(format!("load {file_path:?}: {e}")))?;
            let meta_path = meta_path_for(&file_path);
            let (u2e, e2u, n) = if meta_path.exists() {
                let bytes = std::fs::read(&meta_path)?;
                let map: HashMap<String, u64> =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                let e2u: HashMap<u64, String> =
                    map.iter().map(|(u, e)| (*e, u.clone())).collect();
                let n = map.values().copied().max().unwrap_or(0);
                (map, e2u, n + 1)
            } else {
                (HashMap::new(), HashMap::new(), 1)
            };
            (idx, u2e, e2u, n)
        } else {
            let idx = IdMapIndex::new(dim, bit_width)
                .map_err(|e| BiError::Index(format!("new: {e}")))?;
            (idx, HashMap::new(), HashMap::new(), 1)
        };

        Ok(Self {
            project_id: project_id.to_string(),
            dim,
            bit_width,
            index: Mutex::new(index),
            uid_to_extid: Mutex::new(uid_to_extid),
            extid_to_uid: Mutex::new(extid_to_uid),
            file_path,
            next_extid: Mutex::new(next_extid),
        })
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn add(&self, uid: &str, vector: &[f32]) -> BiResult<()> {
        assert_eq!(vector.len(), self.dim, "vector dim mismatch");
        let mut idx = self.index.lock();
        let mut u2e = self.uid_to_extid.lock();
        let mut e2u = self.extid_to_uid.lock();
        let mut next = self.next_extid.lock();

        if let Some(&extid) = u2e.get(uid) {
            let _ = idx.remove(extid);
            e2u.remove(&extid);
        }

        let extid = *next;
        *next += 1;

        idx.add_with_ids(vector, &[extid])
            .map_err(|e| BiError::Index(format!("add: {e}")))?;

        u2e.insert(uid.to_string(), extid);
        e2u.insert(extid, uid.to_string());

        self.persist(&idx, &u2e)?;
        Ok(())
    }

    pub fn remove(&self, uid: &str) -> BiResult<bool> {
        let mut idx = self.index.lock();
        let mut u2e = self.uid_to_extid.lock();
        let mut e2u = self.extid_to_uid.lock();

        if let Some(extid) = u2e.remove(uid) {
            e2u.remove(&extid);
            let removed = idx.remove(extid);
            self.persist(&idx, &u2e)?;
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        allowlist_uids: Option<&[String]>,
    ) -> BiResult<Vec<SearchHit>> {
        assert_eq!(query.len(), self.dim, "query dim mismatch");
        let idx = self.index.lock();
        let u2e = self.uid_to_extid.lock();
        let e2u = self.extid_to_uid.lock();

        let allowlist_extids: Option<Vec<u64>> = allowlist_uids.map(|uids| {
            uids.iter().filter_map(|u| u2e.get(u).copied()).collect()
        });

        let (scores, ids) = match allowlist_extids.as_ref() {
            Some(v) => idx.search_with_allowlist(query, k, Some(v.as_slice())),
            None => idx.search(query, k),
        };

        Ok(ids
            .into_iter()
            .zip(scores.into_iter())
            .filter_map(|(id, score)| {
                e2u.get(&id).map(|uid| SearchHit {
                    uid: uid.clone(),
                    score,
                    ext_id: id,
                })
            })
            .collect())
    }

    pub fn len(&self) -> usize {
        self.uid_to_extid.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn persist(&self, idx: &IdMapIndex, u2e: &HashMap<String, u64>) -> BiResult<()> {
        idx.write(&self.file_path)
            .map_err(|e| BiError::Index(format!("write: {e}")))?;
        let meta = meta_path_for(&self.file_path);
        let bytes = serde_json::to_vec(u2e)?;
        std::fs::write(&meta, bytes)?;
        Ok(())
    }
}

fn meta_path_for(tvim: &Path) -> PathBuf {
    let mut p = tvim.to_path_buf();
    p.set_extension("uidmap.json");
    p
}
