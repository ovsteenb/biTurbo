use crate::error::{BiError, BiResult};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const DEFAULT_DIM: usize = 384;
pub const DEFAULT_MODEL: &str = "BGE-small-en-v1.5";
const QUERY_CACHE_CAP: usize = 256;
const IDLE_RELEASE: Duration = Duration::from_secs(60 * 60);

pub struct Embedder {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    model_name: &'static str,
    pub dim: usize,
    query_cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    last_used: Arc<Mutex<Instant>>,
}

impl Embedder {
    pub fn new(model_name: &str) -> BiResult<Self> {
        let (model_enum, dim) = match model_name {
            "BGE-small-en-v1.5" | "BGE-small-en" => (EmbeddingModel::BGESmallENV15, 384),
            "BGE-base-en-v1.5" | "BGE-base-en" => (EmbeddingModel::BGEBaseENV15, 768),
            "BGE-large-en-v1.5" | "BGE-large-en" => (EmbeddingModel::BGELargeENV15, 1024),
            "all-MiniLM-L6-v2" => (EmbeddingModel::AllMiniLML6V2, 384),
            other => {
                return Err(BiError::Embed(format!(
                    "unsupported model {other}; supported: BGE-small-en-v1.5, BGE-base-en-v1.5, BGE-large-en-v1.5, all-MiniLM-L6-v2"
                )))
            }
        };

        let opts = InitOptions::new(model_enum)
            .with_show_download_progress(false)
            .with_cache_dir(
                dirs::cache_dir()
                    .ok_or_else(|| BiError::Embed("no cache dir".into()))?
                    .join("biturbo/models"),
            );

        let model = TextEmbedding::try_new(opts)
            .map_err(|e| BiError::Embed(format!("init: {e}")))?;

        let now = Instant::now();
        Ok(Self {
            model: Arc::new(Mutex::new(Some(model))),
            model_name: match model_name {
                "BGE-small-en-v1.5" | "BGE-small-en" => "BGE-small-en-v1.5",
                "BGE-base-en-v1.5" | "BGE-base-en" => "BGE-base-en-v1.5",
                "BGE-large-en-v1.5" | "BGE-large-en" => "BGE-large-en-v1.5",
                _ => "BGE-small-en-v1.5",
            },
            dim,
            query_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(QUERY_CACHE_CAP).unwrap(),
            ))),
            last_used: Arc::new(Mutex::new(now)),
        })
    }

    fn model(&self) -> BiResult<()> {
        let mut guard = self.model.lock();
        if guard.is_none() {
            let model_enum = match self.model_name {
                "BGE-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
                "BGE-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
                "BGE-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
                _ => EmbeddingModel::BGESmallENV15,
            };
            let opts = InitOptions::new(model_enum)
                .with_show_download_progress(false)
                .with_cache_dir(
                    dirs::cache_dir()
                        .ok_or_else(|| BiError::Embed("no cache dir".into()))?
                        .join("biturbo/models"),
                );
            let m = TextEmbedding::try_new(opts)
                .map_err(|e| BiError::Embed(format!("reload: {e}")))?;
            *guard = Some(m);
        }
        *self.last_used.lock() = Instant::now();
        Ok(())
    }

    pub fn embed(&self, text: &str) -> BiResult<Vec<f32>> {
        if let Some(v) = self.query_cache.lock().get(text) {
            return Ok(v.clone());
        }
        self.model()?;
        let owned = text.to_string();
        let mut guard = self.model.lock();
        let model = guard.as_mut().expect("model just initialized");
        let docs: Vec<&str> = vec![&owned];
        let mut results = model
            .embed(docs, None)
            .map_err(|e| BiError::Embed(format!("embed: {e}")))?;
        if results.is_empty() {
            return Err(BiError::Embed("no embedding returned".into()));
        }
        let v = results.remove(0);
        self.query_cache.lock().put(owned, v.clone());
        Ok(v)
    }

    pub fn embed_batch(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        self.model()?;
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
            let missing: Vec<String> = to_compute.iter().map(|&i| texts[i].to_string()).collect();
            let missing_refs: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
            let mut guard = self.model.lock();
            let model = guard.as_mut().expect("model just initialized");
            let mut new_results = model
                .embed(missing_refs, None)
                .map_err(|e| BiError::Embed(format!("embed_batch: {e}")))?;
            for (i, idx) in to_compute.iter().enumerate() {
                let v = new_results.remove(0);
                cached[*idx] = Some(v.clone());
                self.query_cache.lock().put(missing[i].clone(), v);
            }
        }
        Ok(cached.into_iter().map(|o| o.unwrap()).collect())
    }

    pub fn release_if_idle(&self) {
        let mut last = self.last_used.lock();
        if last.elapsed() < IDLE_RELEASE {
            return;
        }
        if self.model.lock().is_some() {
            *self.model.lock() = None;
        }
        *last = Instant::now();
    }

    pub fn cache_len(&self) -> usize {
        self.query_cache.lock().len()
    }
}
