use crate::error::{BiError, BiResult};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const DEFAULT_DIM: usize = 384;
pub const DEFAULT_MODEL: &str = "BGE-small-en-v1.5";
const QUERY_CACHE_CAP: usize = 256;
const IDLE_RELEASE: Duration = Duration::from_secs(5 * 60);
/// Explicit batch size for uncached bulk embeddings. Bounded to keep ONNX
/// arena memory low (32 texts × 512 tokens ≈ few hundred MB vs multi-GB).
const EMBED_BATCH: usize = 32;

pub struct Embedder {
    /// RwLock (not Mutex): `TextEmbedding::embed` takes `&self` and the
    /// underlying ONNX session is thread-safe, so concurrent embeds only
    /// need a read lock. The write lock is for lazy (re)init / release.
    model: Arc<RwLock<Option<TextEmbedding>>>,
    model_name: &'static str,
    pub dim: usize,
    query_cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    last_used: Arc<Mutex<Instant>>,
}

impl Embedder {
    pub fn new(model_name: &str) -> BiResult<Self> {
        let (model_enum, canonical, dim) = resolve_model(model_name)?;
        let model = load_model(model_enum)?;
        Ok(Self {
            model: Arc::new(RwLock::new(Some(model))),
            model_name: canonical,
            dim,
            query_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(QUERY_CACHE_CAP).unwrap(),
            ))),
            last_used: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Ensure the model is loaded; cheap read-lock check on the hot path.
    fn ensure_model(&self) -> BiResult<()> {
        if self.model.read().is_none() {
            let mut guard = self.model.write();
            if guard.is_none() {
                let (model_enum, _, _) = resolve_model(self.model_name)?;
                *guard = Some(load_model(model_enum)?);
            }
        }
        *self.last_used.lock() = Instant::now();
        Ok(())
    }

    pub fn embed(&self, text: &str) -> BiResult<Vec<f32>> {
        if let Some(v) = self.query_cache.lock().get(text) {
            return Ok(v.clone());
        }
        self.ensure_model()?;
        let guard = self.model.read();
        let model = guard
            .as_ref()
            .ok_or_else(|| BiError::Embed("model released mid-embed".into()))?;
        let mut results = model
            .embed(vec![text], None)
            .map_err(|e| BiError::Embed(format!("embed: {e}")))?;
        drop(guard);
        if results.is_empty() {
            return Err(BiError::Embed("no embedding returned".into()));
        }
        let v = results.remove(0);
        self.query_cache.lock().put(text.to_string(), v.clone());
        Ok(v)
    }

    pub fn embed_batch(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        self.ensure_model()?;
        let mut to_compute: Vec<usize> = Vec::new();
        let mut cached: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        {
            let mut cache = self.query_cache.lock();
            for (i, t) in texts.iter().enumerate() {
                if let Some(v) = cache.get(*t) {
                    cached[i] = Some(v.clone());
                } else {
                    to_compute.push(i);
                }
            }
        }
        if !to_compute.is_empty() {
            let missing: Vec<&str> = to_compute.iter().map(|&i| texts[i]).collect();
            let guard = self.model.read();
            let model = guard
                .as_ref()
                .ok_or_else(|| BiError::Embed("model released mid-embed".into()))?;
            let new_results = model
                .embed(missing, None)
                .map_err(|e| BiError::Embed(format!("embed_batch: {e}")))?;
            drop(guard);
            let mut cache = self.query_cache.lock();
            for (idx, v) in to_compute.iter().zip(new_results) {
                cache.put(texts[*idx].to_string(), v.clone());
                cached[*idx] = Some(v);
            }
        }
        Ok(cached.into_iter().map(|o| o.unwrap()).collect())
    }

    /// Uncached bulk embedding for large batches (e.g., project ingest).
    /// Skips the LRU cache to avoid pollution and uses an explicit small batch
    /// size to bound ONNX arena memory. Processes in chunks of EMBED_BATCH.
    /// Sub-batches are processed in parallel using rayon to utilize all cores.
    pub fn embed_batch_uncached(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        self.ensure_model()?;
        let guard = self.model.read();
        let model = guard
            .as_ref()
            .ok_or_else(|| BiError::Embed("model released mid-embed".into()))?;

        let model_ref = &*model;
        let all_results: Vec<Vec<Vec<f32>>> = texts
            .par_chunks(EMBED_BATCH)
            .map(|chunk: &[&str]| {
                let results = model_ref
                    .embed(chunk.to_vec(), Some(EMBED_BATCH))
                    .map_err(|e| BiError::Embed(format!("embed_batch_uncached: {e}")))?;
                Ok(results)
            })
            .collect::<Result<Vec<_>, BiError>>()?;

        drop(guard);

        // Flatten the results in order
        let mut flattened = Vec::with_capacity(texts.len());
        for mut chunk_results in all_results {
            flattened.append(&mut chunk_results);
        }
        Ok(flattened)
    }

    pub fn release_if_idle(&self) {
        if self.last_used.lock().elapsed() < IDLE_RELEASE {
            return;
        }
        *self.model.write() = None;
        *self.last_used.lock() = Instant::now();
    }

    /// Force immediate model release — call after heavy workloads like ingest
    /// to free ONNX session memory and threads immediately.
    pub fn force_release(&self) {
        *self.model.write() = None;
        *self.last_used.lock() = Instant::now();
    }

    pub fn cache_len(&self) -> usize {
        self.query_cache.lock().len()
    }
}

fn resolve_model(name: &str) -> BiResult<(EmbeddingModel, &'static str, usize)> {
    Ok(match name {
        "BGE-small-en-v1.5" | "BGE-small-en" => (EmbeddingModel::BGESmallENV15, "BGE-small-en-v1.5", 384),
        "BGE-base-en-v1.5" | "BGE-base-en" => (EmbeddingModel::BGEBaseENV15, "BGE-base-en-v1.5", 768),
        "BGE-large-en-v1.5" | "BGE-large-en" => (EmbeddingModel::BGELargeENV15, "BGE-large-en-v1.5", 1024),
        "all-MiniLM-L6-v2" => (EmbeddingModel::AllMiniLML6V2, "all-MiniLM-L6-v2", 384),
        other => {
            return Err(BiError::Embed(format!(
                "unsupported model {other}; supported: BGE-small-en-v1.5, BGE-base-en-v1.5, BGE-large-en-v1.5, all-MiniLM-L6-v2"
            )))
        }
    })
}

fn load_model(model_enum: EmbeddingModel) -> BiResult<TextEmbedding> {
    // Force CPU-only execution — disable CoreML/Metal GPU to prevent
    // high GPU usage and thermal throttling on Apple Silicon.
    std::env::set_var("ORT_DISABLE_CORE_ML", "1");
    std::env::set_var("ORT_DNNL_DISABLE", "1");

    let opts = InitOptions::new(model_enum)
        .with_show_download_progress(false)
        .with_cache_dir(
            dirs::cache_dir()
                .ok_or_else(|| BiError::Embed("no cache dir".into()))?
                .join("biturbo/models"),
        );
    TextEmbedding::try_new(opts).map_err(|e| BiError::Embed(format!("init: {e}")))
}
