use std::collections::HashMap;
use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

pub struct SourceRegistry {
    providers: HashMap<String, Box<dyn SourceProvider>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn SourceProvider>) {
        let key = provider.id().to_string();
        self.providers.insert(key, provider);
    }

    pub fn get(&self, id: &str) -> Option<&dyn SourceProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    pub fn list_sources(&self) -> Vec<Source> {
        self.providers
            .values()
            .map(|p| Source {
                id: p.id().to_string(),
                name: p.name().to_string(),
                kind: p.kind(),
                base_url: String::new(),
                config: serde_json::Value::Null,
                is_active: true,
            })
            .collect()
    }

    pub async fn search(
        &self,
        query: &str,
        source_ids: &[String],
        limit: usize,
    ) -> Result<SearchResult> {
        let start = std::time::Instant::now();

        let mut futures = Vec::new();
        for id in source_ids {
            if let Some(provider) = self.providers.get(id) {
                futures.push(provider.search(query, limit));
            }
        }

        // If no specific sources requested, search all
        if source_ids.is_empty() {
            for provider in self.providers.values() {
                futures.push(provider.search(query, limit));
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

        Ok(SearchResult {
            query: query.to_string(),
            items,
            total_results: total,
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub async fn fetch_latest(&self, source_ids: &[String], limit: usize) -> Result<Vec<ContentItem>> {
        let mut futures = Vec::new();

        if source_ids.is_empty() {
            for provider in self.providers.values() {
                futures.push(provider.fetch_latest(limit));
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
