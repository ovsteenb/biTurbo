use crate::error::{BiError, BiResult};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use parking_lot::Mutex;
use std::sync::Arc;

pub const DEFAULT_DIM: usize = 384;
pub const DEFAULT_MODEL: &str = "BGE-small-en-v1.5";

pub struct Embedder {
    inner: Arc<Mutex<TextEmbedding>>,
    pub dim: usize,
}

impl Embedder {
    pub fn new(model_name: &str) -> BiResult<Self> {
        let model = match model_name {
            "BGE-small-en-v1.5" | "BGE-small-en" => EmbeddingModel::BGESmallENV15,
            "BGE-base-en-v1.5" | "BGE-base-en" => EmbeddingModel::BGEBaseENV15,
            "BGE-large-en-v1.5" | "BGE-large-en" => EmbeddingModel::BGELargeENV15,
            "all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            other => {
                return Err(BiError::Embed(format!(
                    "unsupported model {other}; supported: BGE-small-en-v1.5, BGE-base-en-v1.5, BGE-large-en-v1.5, all-MiniLM-L6-v2"
                )))
            }
        };

        let opts = InitOptions::new(model)
            .with_show_download_progress(false)
            .with_cache_dir(
                dirs::cache_dir()
                    .ok_or_else(|| BiError::Embed("no cache dir".into()))?
                    .join("biturbo/models"),
            );

        let inner = TextEmbedding::try_new(opts)
            .map_err(|e| BiError::Embed(format!("init: {e}")))?;

        let dim = match model_name {
            "BGE-small-en-v1.5" | "BGE-small-en" | "all-MiniLM-L6-v2" => 384,
            "BGE-base-en-v1.5" | "BGE-base-en" => 768,
            "BGE-large-en-v1.5" | "BGE-large-en" => 1024,
            _ => 384,
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            dim,
        })
    }

    pub fn embed(&self, text: &str) -> BiResult<Vec<f32>> {
        let mut inner = self.inner.lock();
        let docs: Vec<&str> = vec![text];
        let mut results = inner
            .embed(docs, None)
            .map_err(|e| BiError::Embed(format!("embed: {e}")))?;
        if results.is_empty() {
            return Err(BiError::Embed("no embedding returned".into()));
        }
        Ok(results.remove(0))
    }

    pub fn embed_batch(&self, texts: &[&str]) -> BiResult<Vec<Vec<f32>>> {
        let mut inner = self.inner.lock();
        let results = inner
            .embed(texts.to_vec(), None)
            .map_err(|e| BiError::Embed(format!("embed_batch: {e}")))?;
        Ok(results)
    }
}
