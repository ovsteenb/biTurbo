use crate::error::{BiError, BiResult};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing;
use turbovec::IdMapIndex;

/// All mutable index state behind one lock — `add`/`search`/`remove` each
/// need every map anyway, so a single acquisition beats four.
struct Inner {
    index: IdMapIndex,
    uid_to_extid: HashMap<String, u64>,
    extid_to_uid: HashMap<u64, String>,
    next_extid: u64,
}

pub struct ProjectIndex {
    pub project_id: String,
    pub dim: usize,
    pub bit_width: usize,
    inner: Mutex<Inner>,
    file_path: PathBuf,
    dirty: AtomicBool,
    last_change: Mutex<Instant>,
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
                let map: HashMap<String, u64> = serde_json::from_slice(&bytes).unwrap_or_default();
                let e2u: HashMap<u64, String> = map.iter().map(|(u, e)| (*e, u.clone())).collect();
                let n = map.values().copied().max().unwrap_or(0);
                (map, e2u, n + 1)
            } else {
                (HashMap::new(), HashMap::new(), 1)
            };
            (idx, u2e, e2u, n)
        } else {
            let idx =
                IdMapIndex::new(dim, bit_width).map_err(|e| BiError::Index(format!("new: {e}")))?;
            (idx, HashMap::new(), HashMap::new(), 1)
        };

        Ok(Self {
            project_id: project_id.to_string(),
            dim,
            bit_width,
            inner: Mutex::new(Inner {
                index,
                uid_to_extid,
                extid_to_uid,
                next_extid,
            }),
            file_path,
            dirty: AtomicBool::new(false),
            last_change: Mutex::new(Instant::now()),
        })
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn add(&self, uid: &str, vector: &[f32]) -> BiResult<()> {
        assert_eq!(vector.len(), self.dim, "vector dim mismatch");
        let mut inner = self.inner.lock();
        inner.add_one(uid, vector)?;
        drop(inner);
        self.mark_dirty();
        Ok(())
    }

    /// Add many (uid, vector) pairs under a single lock acquisition.
    /// New uids are appended in one `add_with_ids` call so turbovec can
    /// process them as a contiguous block.
    pub fn add_batch(&self, items: &[(String, Vec<f32>)]) -> BiResult<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut inner = self.inner.lock();
        let mut flat: Vec<f32> = Vec::with_capacity(items.len() * self.dim);
        let mut ids: Vec<u64> = Vec::with_capacity(items.len());
        let mut seen: HashSet<&str> = HashSet::new();
        for (uid, vector) in items {
            assert_eq!(vector.len(), self.dim, "vector dim mismatch");
            if !seen.insert(uid) {
                tracing::warn!(
                    "index: duplicate uid '{}' skipped in add_batch ({} items, ext-ids so far {})",
                    uid,
                    items.len(),
                    ids.len()
                );
                continue; // duplicate uid in same batch — skip
            }
            let extid = match inner.uid_to_extid.get(uid) {
                Some(&id) => {
                    let _ = inner.index.remove(id);
                    inner.extid_to_uid.remove(&id);
                    id
                }
                None => {
                    let id = inner.next_extid;
                    inner.next_extid += 1;
                    id
                }
            };
            inner.uid_to_extid.insert(uid.clone(), extid);
            inner.extid_to_uid.insert(extid, uid.clone());
            ids.push(extid);
            flat.extend_from_slice(vector);
        }
        inner
            .index
            .add_with_ids(&flat, &ids)
            .map_err(|e| BiError::Index(format!("add_batch: {e}")))?;
        drop(inner);
        self.mark_dirty();
        Ok(())
    }

    pub fn remove(&self, uid: &str) -> BiResult<bool> {
        let mut inner = self.inner.lock();
        if let Some(extid) = inner.uid_to_extid.remove(uid) {
            inner.extid_to_uid.remove(&extid);
            let removed = inner.index.remove(extid);
            drop(inner);
            self.mark_dirty();
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
        *self.last_change.lock() = Instant::now();
    }

    /// Persist the index to disk if it's been dirty for at least `min_idle` and
    /// the last change was more than `min_idle` ago, or if `force` is true.
    /// Returns true if a write actually happened.
    pub fn maybe_flush(&self, min_idle: Duration, force: bool) -> BiResult<bool> {
        if !self.dirty.load(Ordering::Acquire) {
            return Ok(false);
        }
        if !force && self.last_change.lock().elapsed() < min_idle {
            return Ok(false);
        }
        self.persist_now()
    }

    /// Force a persist regardless of dirty/idle state.
    pub fn flush(&self) -> BiResult<bool> {
        self.persist_now()
    }

    fn persist_now(&self) -> BiResult<bool> {
        let inner = self.inner.lock();
        // Write to temp files then rename, so a crash mid-write never
        // corrupts the on-disk index.
        let tmp_index = self.file_path.with_extension("tvim.tmp");
        inner
            .index
            .write(&tmp_index)
            .map_err(|e| BiError::Index(format!("write: {e}")))?;
        let meta = meta_path_for(&self.file_path);
        let tmp_meta = meta.with_extension("json.tmp");
        {
            let file = std::fs::File::create(&tmp_meta)?;
            let mut w = std::io::BufWriter::new(file);
            serde_json::to_writer(&mut w, &inner.uid_to_extid)?;
            w.flush()?;
        }
        drop(inner);
        std::fs::rename(&tmp_index, &self.file_path)?;
        std::fs::rename(&tmp_meta, &meta)?;
        self.dirty.store(false, Ordering::Release);
        Ok(true)
    }

    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        allowlist_uids: Option<&[String]>,
    ) -> BiResult<Vec<SearchHit>> {
        assert_eq!(query.len(), self.dim, "query dim mismatch");
        let inner = self.inner.lock();

        let allowlist_extids: Option<Vec<u64>> = allowlist_uids.map(|uids| {
            uids.iter()
                .filter_map(|u| inner.uid_to_extid.get(u).copied())
                .collect()
        });

        let (scores, ids) = match allowlist_extids.as_ref() {
            Some(v) => inner
                .index
                .search_with_allowlist(query, k, Some(v.as_slice())),
            None => inner.index.search(query, k),
        };

        Ok(ids
            .into_iter()
            .zip(scores)
            .filter_map(|(id, score)| {
                inner.extid_to_uid.get(&id).map(|uid| SearchHit {
                    uid: uid.clone(),
                    score,
                    ext_id: id,
                })
            })
            .collect())
    }

    /// Search the index, then return only hits whose uid is in `filter_uids`.
    /// Cheaper than `search_with_allowlist` when the candidate set is small
    /// relative to the index.
    pub fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter_uids: &[String],
    ) -> BiResult<Vec<SearchHit>> {
        assert_eq!(query.len(), self.dim, "query dim mismatch");
        let filter: HashSet<&str> = filter_uids.iter().map(|s| s.as_str()).collect();
        let inner = self.inner.lock();
        let (scores, ids) = inner.index.search(query, k);
        Ok(ids
            .into_iter()
            .zip(scores)
            .filter_map(|(id, score)| {
                inner
                    .extid_to_uid
                    .get(&id)
                    .filter(|uid| filter.contains(uid.as_str()))
                    .map(|uid| SearchHit {
                        uid: uid.clone(),
                        score,
                        ext_id: id,
                    })
            })
            .collect())
    }

    pub fn len(&self) -> usize {
        self.inner.lock().uid_to_extid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Inner {
    fn add_one(&mut self, uid: &str, vector: &[f32]) -> BiResult<()> {
        let extid = match self.uid_to_extid.get(uid) {
            Some(&id) => {
                let _ = self.index.remove(id);
                self.extid_to_uid.remove(&id);
                id
            }
            None => {
                let id = self.next_extid;
                self.next_extid += 1;
                id
            }
        };
        self.index
            .add_with_ids(vector, &[extid])
            .map_err(|e| BiError::Index(format!("add: {e}")))?;
        self.uid_to_extid.insert(uid.to_string(), extid);
        self.extid_to_uid.insert(extid, uid.to_string());
        Ok(())
    }
}

fn meta_path_for(tvim: &Path) -> PathBuf {
    let mut p = tvim.to_path_buf();
    p.set_extension("uidmap.json");
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_for(seed: f32, dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32 * 0.01 + seed).sin()).collect()
    }

    #[test]
    fn add_batch_search_persist_roundtrip() {
        let dir = std::env::temp_dir().join(format!("biturbo-test-{}", uuid::Uuid::new_v4()));
        let dim = 32;
        let idx = ProjectIndex::open_or_create("t", dim, 4, &dir).unwrap();

        let items: Vec<(String, Vec<f32>)> = (0..50)
            .map(|i| (format!("uid-{i}"), vec_for(i as f32, dim)))
            .collect();
        idx.add_batch(&items).unwrap();
        idx.add("uid-extra", &vec_for(99.0, dim)).unwrap();
        assert_eq!(idx.len(), 51);

        // Re-adding an existing uid replaces, not duplicates.
        idx.add_batch(&[("uid-0".to_string(), vec_for(0.0, dim))])
            .unwrap();
        assert_eq!(idx.len(), 51);

        // Search returns hits; bit_width=4 quantization makes exact NN
        // ordering unreliable on synthetic vectors, so we only assert non-empty.
        let hits = idx.search(&vec_for(7.0, dim), 5, None).unwrap();
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|h| h.uid.starts_with("uid-")));

        let filtered = idx
            .search_filtered(&vec_for(7.0, dim), 10, &["uid-7".to_string()])
            .unwrap();
        assert!(filtered.iter().all(|h| h.uid == "uid-7"));

        // Persist, reload, count preserved.
        assert!(idx.flush().unwrap());
        assert!(dir.join("t.tvim").exists());
        assert!(dir.join("t.uidmap.json").exists());
        let reloaded = ProjectIndex::open_or_create("t", dim, 4, &dir).unwrap();
        assert_eq!(reloaded.len(), 51);
        let hits = reloaded.search(&vec_for(7.0, dim), 5, None).unwrap();
        assert!(!hits.is_empty());

        assert!(reloaded.remove("uid-7").unwrap());
        assert_eq!(reloaded.len(), 50);

        std::fs::remove_dir_all(&dir).ok();
    }
}
