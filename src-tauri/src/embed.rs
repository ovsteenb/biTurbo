use crate::error::{BiError, BiResult};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use hex;
use lru::LruCache;
use once_cell::sync::Lazy;
use ort::execution_providers::CPUExecutionProvider;
use parking_lot::{Mutex, RwLock};
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing;

static EMBED_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub const DEFAULT_DIM: usize = 384;
pub const DEFAULT_MODEL: &str = "BGE-small-en-v1.5";
const QUERY_CACHE_CAP: usize = 256;
const IDLE_RELEASE: Duration = Duration::from_secs(2 * 60);
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
        let mut out = self.embed_batch(&[text])?;
        Ok(out.remove(0))
    }

    pub fn embed_batch(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        let mut cached: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        let mut missing: Vec<(usize, &str)> = Vec::new();
        {
            let mut cache = self.query_cache.lock();
            for (i, text) in texts.iter().enumerate() {
                let key = cache_key(text);
                if let Some(v) = cache.get(&key) {
                    cached[i] = Some(v.clone());
                } else {
                    missing.push((i, *text));
                }
            }
        }

        if !missing.is_empty() {
            let missing_texts: Vec<&str> = missing.iter().map(|(_, t)| *t).collect();
            let mut computed = Vec::with_capacity(missing.len());
            self.embed_batch_uncached_stream(&missing_texts, |_chunk, results| {
                computed.extend(results);
                Ok(())
            })?;

            let mut cache = self.query_cache.lock();
            for ((idx, text), v) in missing.iter().zip(computed) {
                let key = cache_key(text);
                cache.put(key, v.clone());
                cached[*idx] = Some(v);
            }
        }

        Ok(cached.into_iter().map(|o| o.unwrap()).collect())
    }

    /// Uncached bulk embedding for large batches (e.g., project ingest).
    /// Skips the LRU cache to avoid pollution and streams results in small
    /// batches so callers can drop embeddings as soon as they are consumed.
    pub fn embed_batch_uncached(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        self.embed_batch_uncached_stream(texts, |_chunk, results| {
            out.extend(results);
            Ok(())
        })?;
        Ok(out)
    }

    pub fn embed_batch_uncached_stream<F>(&self, texts: &[&str], mut on_batch: F) -> BiResult<()>
    where
        F: FnMut(&[&str], Vec<Vec<f32>>) -> BiResult<()>,
    {
        self.ensure_model()?;
        let _guard = EMBED_LOCK.lock();
        let guard = self.model.read();
        let model = guard
            .as_ref()
            .ok_or_else(|| BiError::Embed("model released mid-embed".into()))?;

        for chunk in texts.chunks(EMBED_BATCH) {
            let results = model
                .embed(chunk.to_vec(), Some(EMBED_BATCH))
                .map_err(|e| {
                    tracing::error!(
                        "embed: ONNX inference failed for {} texts: {e}",
                        chunk.len()
                    );
                    BiError::Embed(format!("embed_batch_uncached: {e}"))
                })?;
            on_batch(chunk, results)?;
        }

        Ok(())
    }

    pub fn release_if_idle(&self) {
        let idle_for = self.last_used.lock().elapsed();
        if idle_for < IDLE_RELEASE {
            return;
        }
        let _guard = EMBED_LOCK.lock();
        *self.model.write() = None;
        *self.last_used.lock() = Instant::now();
    }

    /// Force immediate model release — call after heavy workloads like ingest
    /// to free ONNX session memory and threads immediately.
    pub fn force_release(&self) {
        let _guard = EMBED_LOCK.lock();
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
    // Disable the ONNX CPU memory arena — this is the #1 cause of RAM bloat.
    // Without this, ONNX Runtime pre-allocates and holds a large arena across
    // every inference call, growing to multiple GB during batch embedding.
    // CPUExecutionProvider::default() has use_arena=false, so this disables it.
    let cpu_ep = CPUExecutionProvider::default().build();

    let opts = InitOptions::new(model_enum)
        .with_execution_providers(vec![cpu_ep])
        .with_show_download_progress(false)
        .with_cache_dir(
            dirs::cache_dir()
                .ok_or_else(|| BiError::Embed("no cache dir".into()))?
                .join("biturbo/models"),
        );
    TextEmbedding::try_new(opts).map_err(|e| BiError::Embed(format!("init: {e}")))
}

fn cache_key(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}
