use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::source::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub struct SourceRegistry {
    providers: HashMap<String, Box<dyn SourceProvider>>,
    disabled: Mutex<HashSet<String>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            disabled: Mutex::new(HashSet::new()),
        }
    }

    pub fn register(&mut self, provider: Box<dyn SourceProvider>) {
        let key = provider.id().to_string();
        self.providers.insert(key, provider);
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) {
        let mut disabled = self.disabled.lock().unwrap();
        if enabled {
            disabled.remove(id);
        } else {
            disabled.insert(id.to_string());
        }
    }

    pub fn list_sources(&self) -> Vec<Source> {
        let disabled = self.disabled.lock().unwrap();
        self.providers
            .values()
            .map(|p| Source {
                id: p.id().to_string(),
                name: p.name().to_string(),
                kind: p.kind(),
                base_url: p.base_url().to_string(),
                is_active: !disabled.contains(p.id()),
            })
            .collect()
    }

    pub async fn search(
        &self,
        query: &str,
        source_ids: &[String],
        limit: usize,
        page: usize,
    ) -> Result<SearchResult> {
        let start = std::time::Instant::now();

        // Snapshot disabled set so MutexGuard is dropped before .await
        let disabled_ids = self.disabled.lock().unwrap().clone();

        let mut futures = Vec::new();
        // When specific source_ids are given, search only those
        for id in source_ids {
            if let Some(provider) = self.providers.get(id) {
                futures.push(provider.search(query, limit, page));
            }
        }

        // When no specific sources requested, search all enabled sources
        if source_ids.is_empty() {
            for provider in self.providers.values() {
                if !disabled_ids.contains(provider.id()) {
                    futures.push(provider.search(query, limit, page));
                }
            }
        }

        let results = futures::future::join_all(futures).await;

        let mut items = Vec::new();
        for res in results {
            match res {
                Ok(r) => items.extend(r),
                Err(e) => eprintln!("Source search error: {}", e),
            }
        }

        // Sort by relevance score (higher first), items without score go last
        items.sort_by(|a, b| {
            let sa = a.relevance_score.unwrap_or(0.0);
            let sb = b.relevance_score.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let total = items.len();
        let has_more = items.len() >= limit;

        Ok(SearchResult {
            query: query.to_string(),
            items,
            total_results: total,
            elapsed_ms: start.elapsed().as_millis() as u64,
            page: SearchPage { page, has_more },
        })
    }

    pub async fn fetch_latest(
        &self,
        source_ids: &[String],
        limit: usize,
    ) -> Result<Vec<ContentItem>> {
        // Snapshot disabled set so MutexGuard is dropped before .await
        let disabled_ids = self.disabled.lock().unwrap().clone();
        let mut futures = Vec::new();

        if source_ids.is_empty() {
            for provider in self.providers.values() {
                if !disabled_ids.contains(provider.id()) {
                    futures.push(provider.fetch_latest(limit));
                }
            }
        } else {
            for id in source_ids {
                if let Some(provider) = self.providers.get(id) {
                    futures.push(provider.fetch_latest(limit));
                }
            }
        }

        let results = futures::future::join_all(futures).await;

        let mut items = Vec::new();
        for res in results {
            match res {
                Ok(r) => items.extend(r),
                Err(e) => eprintln!("Source fetch error: {}", e),
            }
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct StaticSource {
        id: &'static str,
    }

    #[async_trait]
    impl SourceProvider for StaticSource {
        fn id(&self) -> &str {
            self.id
        }

        fn name(&self) -> &str {
            self.id
        }

        fn kind(&self) -> SourceKind {
            SourceKind::Api
        }

        fn base_url(&self) -> &str {
            "https://example.invalid"
        }

        async fn search(
            &self,
            _query: &str,
            _limit: usize,
            _page: usize,
        ) -> Result<Vec<ContentItem>> {
            Ok(vec![ContentItem {
                source_id: self.id.to_string(),
                title: format!("{} result", self.id),
                url: format!("https://example.invalid/{}", self.id),
                summary: None,
                author: None,
                published_at: None,
                image_url: None,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                extra: None,
                relevance_score: Some(1.0),
            }])
        }

        async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn disabled_source_is_excluded_from_all_source_search() {
        let mut registry = SourceRegistry::new();
        registry.register(Box::new(StaticSource { id: "bilibili" }));
        registry.register(Box::new(StaticSource { id: "other" }));

        registry.set_enabled("bilibili", false);
        let sources = registry.list_sources();
        let bili_source = sources
            .iter()
            .find(|source| source.id == "bilibili")
            .expect("Bilibili source should be listed");
        assert!(!bili_source.is_active);

        let result = registry.search("music", &[], 10, 1).await.unwrap();

        assert!(result.items.iter().all(|item| item.source_id != "bilibili"));
        assert!(result.items.iter().any(|item| item.source_id == "other"));
    }
}
