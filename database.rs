use rusqlite::{Connection, Result, params, OptionalExtension}; 
use std::sync::Mutex;

pub struct Player {
    pub discord_id: String,
    pub steam_id: String,
    pub coin: i32,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("bot_database.db")?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
    
    pub fn create_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                steam_id TEXT UNIQUE,
                discord_id TEXT UNIQUE,
                coin INTEGER DEFAULT 0,
                welcome TEXT,
                whitelist TEXT,
                registered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS purchase_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                discord_id TEXT,
                steam_id TEXT,
                item_name TEXT,
                price INTEGER
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS authorized_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT UNIQUE
            )",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn get_player_by_discord_id(&self, discord_id: &str) -> Result<Option<Player>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT discord_id, steam_id, coin FROM players WHERE discord_id = ?"
        )?;
        
        let player = stmt.query_row(params![discord_id], |row| {
            Ok(Player {
                discord_id: row.get(0)?,
                steam_id: row.get(1)?,
                coin: row.get(2)?,
            })
        }).optional()?;
        
        Ok(player)
    }
    
    pub fn add_or_update_player(&self, discord_id: &str, steam_id: &str, coin: i32) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        
        match conn.execute(
            "INSERT INTO players (discord_id, steam_id, coin) VALUES (?1, ?2, ?3)
             ON CONFLICT(discord_id) DO UPDATE SET steam_id = ?2, coin = ?3",
            params![discord_id, steam_id, coin],
        ) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    pub fn update_coin(&self, discord_id: &str, new_coin: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE players SET coin = ? WHERE discord_id = ?",
            params![new_coin, discord_id],
        )?;
        Ok(())
    }
    
    pub fn remove_coin(&self, discord_id: &str, amount: i32) -> Result<bool> {
        if let Some(player) = self.get_player_by_discord_id(discord_id)? {
            if player.coin < amount {
                return Ok(false);
            }
            let new_coin = player.coin - amount;
            self.update_coin(discord_id, new_coin)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn log_purchase(&self, discord_id: &str, steam_id: &str, item_name: &str, price: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO purchase_logs (discord_id, steam_id, item_name, price) VALUES (?, ?, ?, ?)",
            params![discord_id, steam_id, item_name, price],
        )?;
        Ok(())
    }
    
    pub fn is_authorized(&self, user_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM authorized_users WHERE user_id = ?",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
    
    pub fn add_authorized_user(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO authorized_users (user_id) VALUES (?)",
            params![user_id],
        )?;
        Ok(())
    }
}