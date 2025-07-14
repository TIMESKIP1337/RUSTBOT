mod config;
mod database;
mod commands;
mod maintenance;
mod shared_state;
mod utils;
mod handlers;

use std::env;
use std::sync::Arc;
use serenity::prelude::*;
use dotenv::dotenv;
use log::info;

use crate::config::Config;
use crate::database::Database;
use crate::shared_state::SharedState;
use crate::handlers::Handler;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Starting Discord bot...");
    
    dotenv().ok();
    
    println!("Loading environment variables...");
    
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in environment");
    
    println!("Token loaded successfully (length: {})", token.len());
    
    let _guild_id = env::var("GUILD_ID")
        .expect("Expected GUILD_ID in environment")
        .parse::<u64>()
        .expect("GUILD_ID must be a valid u64");
    
    println!("Guild ID loaded successfully");
    

    println!("Initializing database...");
    let db = Database::new().expect("Failed to initialize database");
    db.create_tables().expect("Failed to create tables");
    
    db.add_authorized_user("").ok(); // เพิ่ม authorized user ตรงนี้ เป็น discord id
    
    println!("Loading configuration...");
    let config = match Config::load() {
        Ok(cfg) => {
            println!("Configuration loaded successfully");
            cfg
        },
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please check your botshop.json file");
            return;
        }
    };
    
    println!("Creating shared state...");
    let shared_state = Arc::new(SharedState::new(db, config));
    println!("Shared state created");

    let intents = GatewayIntents::GUILDS 
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    
    println!("Building Discord client...");
    
    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler::new(shared_state.clone()))
        .await {
        Ok(client) => {
            println!("Client built successfully");
            client
        },
        Err(e) => {
            eprintln!("Failed to create Discord client: {}", e);
            eprintln!("Error details: {:?}", e);
            eprintln!("Please check:");
            eprintln!("1. Your DISCORD_TOKEN in .env file");
            eprintln!("2. Bot has proper intents enabled in Discord Developer Portal");
            eprintln!("3. Network connection");
            return;
        }
    };

    info!("Starting bot connection...");
    println!("Attempting to connect to Discord...");
    
    match client.start().await {
        Ok(_) => {
            println!("Bot shut down gracefully");
        },
        Err(why) => {
            eprintln!("Client error: {:?}", why);
            eprintln!("\nCommon issues:");
            eprintln!("1. Invalid token - check DISCORD_TOKEN in .env");
            eprintln!("2. Missing intents - enable MESSAGE CONTENT INTENT in Discord Developer Portal");
            eprintln!("3. No internet connection");
            eprintln!("4. Discord API is down");
            eprintln!("5. Bot doesn't have access to the guild");
        }
    }
}