use anyhow::Context;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub fn compute_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Realiza gravação atômica em disco utilizando arquivo temporário e rename do sistema de arquivos.
pub async fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp_path = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));

    tokio::fs::write(&tmp_path, content)
        .await
        .with_context(|| {
            format!(
                "Falha ao gravar arquivo temporário '{}'",
                tmp_path.display()
            )
        })?;

    tokio::fs::rename(&tmp_path, path)
        .await
        .with_context(|| format!("Falha ao renomear arquivo para '{}'", path.display()))?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub file_path: PathBuf,
    pub layout_path: PathBuf,
    pub content: Arc<RwLock<String>>,
    pub layout: Arc<RwLock<Option<String>>>,
    pub last_saved_hash: Arc<RwLock<u64>>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub async fn new(file_path: PathBuf) -> anyhow::Result<Self> {
        let layout_path = PathBuf::from(format!("{}.layout.json", file_path.display()));

        // Cria arquivo com template inicial se não existir
        if !file_path.exists() {
            let initial_dbml = "// Documento DBML\n\nTable users {\n  id int [pk]\n  name varchar\n  email varchar [unique]\n}\n\nTable posts {\n  id int [pk]\n  user_id int\n  title varchar\n}\n\nRef: posts.user_id > users.id\n";
            atomic_write(&file_path, initial_dbml).await?;
        }

        let content = tokio::fs::read_to_string(&file_path)
            .await
            .with_context(|| format!("Não foi possível ler o arquivo '{}'", file_path.display()))?;

        let hash = compute_hash(&content);

        let layout = if layout_path.exists() {
            tokio::fs::read_to_string(&layout_path).await.ok()
        } else {
            None
        };

        let (tx, _) = broadcast::channel(100);

        Ok(Self {
            file_path,
            layout_path,
            content: Arc::new(RwLock::new(content)),
            layout: Arc::new(RwLock::new(layout)),
            last_saved_hash: Arc::new(RwLock::new(hash)),
            tx,
        })
    }

    pub async fn save_content(&self, new_content: String) -> anyhow::Result<()> {
        let new_hash = compute_hash(&new_content);

        // Atualiza a hash em memória antes de gravar para que o watcher ignore o próprio eco
        {
            let mut hash_lock = self.last_saved_hash.write().await;
            *hash_lock = new_hash;
        }

        atomic_write(&self.file_path, &new_content).await?;

        {
            let mut content_lock = self.content.write().await;
            *content_lock = new_content;
        }

        Ok(())
    }

    pub async fn save_layout(&self, layout_json: String) -> anyhow::Result<()> {
        atomic_write(&self.layout_path, &layout_json).await?;
        let mut layout_lock = self.layout.write().await;
        *layout_lock = Some(layout_json);
        Ok(())
    }
}
