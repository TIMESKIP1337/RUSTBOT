use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use dashmap::DashMap;
use std::time::{Duration, Instant};
use crate::config::Config;
use crate::database::Database;

pub struct SharedState {
    pub db: Arc<Database>,
    pub config: Arc<Config>,
    pub destroy_lock: Arc<Mutex<()>>,
    pub bot_active: Arc<RwLock<bool>>,
    pub item_cooldowns: Arc<DashMap<(String, String), Instant>>,
    pub command_queue: Arc<Mutex<Vec<Vec<String>>>>,
}

impl SharedState {
    pub fn new(db: Database, config: Config) -> Self {
        SharedState {
            db: Arc::new(db),
            config: Arc::new(config),
            destroy_lock: Arc::new(Mutex::new(())),
            bot_active: Arc::new(RwLock::new(true)),
            item_cooldowns: Arc::new(DashMap::new()),
            command_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn is_bot_active(&self) -> bool {
        *self.bot_active.read().await
    }
    
    pub async fn set_bot_active(&self, active: bool) {
        *self.bot_active.write().await = active;
    }
    
    pub fn check_cooldown(&self, user_id: &str, item_id: &str) -> Option<Duration> {
        let key = (user_id.to_string(), item_id.to_string());
        if let Some(last_use) = self.item_cooldowns.get(&key) {
            let elapsed = last_use.elapsed();
            if elapsed < Duration::from_secs(20) {
                return Some(Duration::from_secs(20) - elapsed);
            }
        }
        None
    }
    
    pub fn set_cooldown(&self, user_id: &str, item_id: &str) {
        let key = (user_id.to_string(), item_id.to_string());
        self.item_cooldowns.insert(key, Instant::now());
    }
}