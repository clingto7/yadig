use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

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

    pub async fn fetch_latest(&self, source_ids: &[String], limit: usize) -> Result<Vec<ContentItem>> {
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
