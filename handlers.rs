use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateEmbed, CreateEmbedFooter, CreateActionRow, CreateButton, GetMessages, CreateInteractionResponseFollowup};
use std::sync::Arc;
use log::info;
use tokio::time::{sleep, Duration};

use crate::shared_state::SharedState;
use crate::utils::{send_commands_to_game, substitute_steam_id_in_commands, is_special_command, calculate_discounted_price};
use crate::config::VipRole;

pub struct Handler {
    shared_state: Arc<SharedState>,
}

impl Handler {
    pub fn new(shared_state: Arc<SharedState>) -> Self {
        Handler { shared_state }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Bot {} is connected!", ready.user.name);
        info!("{} is connected!", ready.user.name);
        
        let shared_state = self.shared_state.clone();
        tokio::spawn(async move {
            crate::maintenance::start_maintenance_schedule(shared_state).await;
        });
        
        let shared_state = self.shared_state.clone();
        tokio::spawn(async move {
            process_command_queue(shared_state).await;
        });
        
        let shared_state = self.shared_state.clone();
        tokio::spawn(async move {
            auto_destroy_items_type1(shared_state).await;
        });
        
        let shared_state = self.shared_state.clone();
        tokio::spawn(async move {
            auto_destroy_items_type2(shared_state).await;
        });
        
        println!("All background tasks started successfully!");
    }
    
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        
        if msg.content == "!destroy" {
            crate::commands::handle_destroy_command(&ctx, &msg).await;
        }
        
        if msg.content.starts_with("!register ") {
            let args: Vec<&str> = msg.content.split_whitespace().collect();
            if args.len() >= 2 {
                let steam_id = args[1];
                let discord_id = msg.author.id.to_string();
                
                match self.shared_state.db.add_or_update_player(&discord_id, steam_id, 0) {
                    Ok(_) => {
                        let _ = msg.reply(&ctx.http, format!("✅ ลงทะเบียนสำเร็จ! Steam ID: {}", steam_id)).await;
                    },
                    Err(e) => {
                        let _ = msg.reply(&ctx.http, format!("❌ ลงทะเบียนไม่สำเร็จ: {:?}", e)).await;
                    }
                }
            } else {
                let _ = msg.reply(&ctx.http, "❌ รูปแบบคำสั่ง: !register <steam_id>").await;
            }
        }
        
        if msg.content == "!coin" {
            let discord_id = msg.author.id.to_string();
            match self.shared_state.db.get_player_by_discord_id(&discord_id) {
                Ok(Some(player)) => {
                    let _ = msg.reply(&ctx.http, format!("💰 คุณมี {} coins", player.coin)).await;
                },
                Ok(none) => {
                    let _ = msg.reply(&ctx.http, "❌ คุณยังไม่ได้ลงทะเบียน ใช้คำสั่ง !register <steam_id>").await;
                },
                Err(e) => {
                    let _ = msg.reply(&ctx.http, format!("❌ เกิดข้อผิดพลาด: {:?}", e)).await;
                }
            }
        }
        
        if msg.content == "!updateshop" {
            if self.shared_state.db.is_authorized(&msg.author.id.to_string()).unwrap_or(false) {
                let _ = msg.reply(&ctx.http, "🔄 กำลังอัปเดตร้านค้า...").await;
                
                for shop in &self.shared_state.config.shop_data {
                    if let Ok(channel_id) = shop.channel.parse::<u64>() {
                        let channel_id = ChannelId::new(channel_id);
                        
                        let messages = channel_id
                            .messages(&ctx.http, GetMessages::new().limit(100))
                            .await
                            .unwrap_or_default();
                        
                        for message in messages {
                            let _ = message.delete(&ctx.http).await;
                            sleep(Duration::from_millis(100)).await;
                        }
                        
                        for item in &shop.items {
                            let embed = CreateEmbed::new()
                                .title(&item.name)
                                .color(0xFF00FF)
                                .field("💰 ราคา", format!("{} COIN", item.price), true)
                                .thumbnail("https://cdn.discordapp.com/attachments/1347264410087067709/1364553843316363304/raw.png")
                                .footer(CreateEmbedFooter::new("© powered by TimeSkip"));
                            
                            let mut components = vec![];
                            let mut buttons = vec![];
                            
                            for button in &item.buttons {
                                buttons.push(
                                    CreateButton::new(&button.trigger)
                                        .label(&button.text)
                                        .style(ButtonStyle::Danger)
                                );
                                
                                if buttons.len() == 5 {
                                    components.push(CreateActionRow::Buttons(buttons.clone()));
                                    buttons.clear();
                                }
                            }
                            
                            if !buttons.is_empty() {
                                components.push(CreateActionRow::Buttons(buttons));
                            }
                            
                            let message = CreateMessage::new()
                                .embed(embed)
                                .components(components);
                            
                            let _ = channel_id.send_message(&ctx.http, message).await;
                            
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
                
                let _ = msg.reply(&ctx.http, "✅ อัปเดตร้านค้าเรียบร้อย!").await;
            } else {
                let _ = msg.reply(&ctx.http, "❌ คุณไม่มีสิทธิ์ใช้คำสั่งนี้").await;
            }
        }
        
        if msg.content.starts_with("!addcoin ") {
            if self.shared_state.db.is_authorized(&msg.author.id.to_string()).unwrap_or(false) {
                let args: Vec<&str> = msg.content.split_whitespace().collect();
                if args.len() >= 3 {
                    let user_mention = args[1];
                    if let Some(user_id) = user_mention.strip_prefix("<@").and_then(|s| s.strip_suffix(">")) {
                        let user_id = user_id.trim_start_matches('!');
                        if let Ok(amount) = args[2].parse::<i32>() {
                            match self.shared_state.db.get_player_by_discord_id(user_id) {
                                Ok(Some(mut player)) => {
                                    player.coin += amount;
                                    if let Err(e) = self.shared_state.db.update_coin(user_id, player.coin) {
                                        let _ = msg.reply(&ctx.http, format!("❌ ไม่สามารถอัปเดต coin: {:?}", e)).await;
                                    } else {
                                        let _ = msg.reply(&ctx.http, format!("✅ เพิ่ม {} coins ให้ <@{}> สำเร็จ! (รวม: {} coins)", amount, user_id, player.coin)).await;
                                    }
                                },
                                Ok(none) => {
                                    let _ = msg.reply(&ctx.http, "❌ ผู้ใช้ยังไม่ได้ลงทะเบียน").await;
                                },
                                Err(e) => {
                                    let _ = msg.reply(&ctx.http, format!("❌ เกิดข้อผิดพลาด: {:?}", e)).await;
                                }
                            }
                        } else {
                            let _ = msg.reply(&ctx.http, "❌ จำนวน coin ไม่ถูกต้อง").await;
                        }
                    } else {
                        let _ = msg.reply(&ctx.http, "❌ รูปแบบคำสั่ง: !addcoin @user amount").await;
                    }
                } else {
                    let _ = msg.reply(&ctx.http, "❌ รูปแบบคำสั่ง: !addcoin @user amount").await;
                }
            } else {
                let _ = msg.reply(&ctx.http, "❌ คุณไม่มีสิทธิ์ใช้คำสั่งนี้").await;
            }
        }
        
        if msg.content == "!help" {
            let help_message = "
**📋 คำสั่งที่ใช้ได้:**

**สำหรับผู้เล่น:**
`!register <steam_id>` - ลงทะเบียนเพื่อใช้งาน bot
`!coin` - เช็คจำนวน coin ที่มี
`!destroy` - แสดงปุ่มลบไอเทม

**สำหรับ Admin:**
`!updateshop` - อัปเดตร้านค้าทั้งหมด
`!addcoin @user amount` - เพิ่ม coin ให้ผู้เล่น

**วิธีซื้อของ:**
1. ลงทะเบียนด้วย `!register <steam_id>`
2. ตรวจสอบ coin ด้วย `!coin`
3. ไปที่ช่องร้านค้าและกดปุ่มซื้อ
            ";
            let _ = msg.reply(&ctx.http, help_message).await;
        }
    }
    
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Component(component) = interaction {
            let custom_id = &component.data.custom_id;
            
            if !self.shared_state.is_bot_active().await {
                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("❌ ระบบ BOTSHOP หยุดทำงานชั่วคราวเนื่องจาก SERVER กำลังจะ RESTART")
                        .ephemeral(true)
                );
                let _ = component.create_response(&ctx.http, response).await;
                return;
            }
            
            if custom_id == "destroy_type1" || custom_id == "destroy_type2" {
                self.handle_destroy_command(&ctx, &component, custom_id).await;
                return;
            }
            
            self.handle_shop_interaction(&ctx, &component, custom_id).await;
        }
    }
}

impl Handler {
    async fn handle_destroy_command(&self, ctx: &Context, component: &ComponentInteraction, command_type: &str) {
        let _ = component.defer_ephemeral(&ctx.http).await;
        
        let _guard = self.shared_state.destroy_lock.lock().await;
        
        let commands = match command_type {
            "destroy_type1" => &self.shared_state.config.destroy_commands_type1,
            "destroy_type2" => &self.shared_state.config.destroy_commands_type2,
            _ => return,
        };
        
        let announce_start = match command_type {
            "destroy_type1" => "#Announce บอทกำลังลบชุด/เสื้อผ้า บอทหยุดชั่วคราว!",
            "destroy_type2" => "#Announce บอทกำลังลบวัสดุก่อสร้าง/เศษไม้ บอทหยุดชั่วคราว!",
            _ => return,
        };
        
        let announce_end = match command_type {
            "destroy_type1" => "#Announce บอทลบชุด/เสื้อผ้าสำเร็จ สามารถใช้บอทต่อได้!",
            "destroy_type2" => "#Announce บอทลบวัสดุก่อสร้าง/เศษไม้สำเร็จ สามารถใช้บอทต่อได้!",
            _ => return,
        };
        
        send_commands_to_game(vec![announce_start.to_string()], "destroy").await;
        sleep(Duration::from_secs(1)).await;
        send_commands_to_game(commands.clone(), "destroy").await;
        send_commands_to_game(vec![announce_end.to_string()], "destroy").await;
        
        let content = CreateInteractionResponseFollowup::new()
            .content("✅ ดำเนินการเรียบร้อย")
            .ephemeral(true);
        let _ = component.create_followup(&ctx.http, content).await;
    }
    
    async fn handle_shop_interaction(&self, ctx: &Context, component: &ComponentInteraction, custom_id: &str) {
        if let Ok(_) = self.shared_state.destroy_lock.try_lock() {
        } else {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ บอทกำลังลบขยะ กรุณาลองใหม่อีกครั้งภายหลัง")
                    .ephemeral(true)
            );
            let _ = component.create_response(&ctx.http, response).await;
            return;
        }
        
        let (item, button) = {
            let mut found = None;
            for shop in &self.shared_state.config.shop_data {
                for item in &shop.items {
                    for button in &item.buttons {
                        if button.trigger == custom_id {
                            found = Some((item.clone(), button.clone()));
                            break;
                        }
                    }
                    if found.is_some() { break; }
                }
                if found.is_some() { break; }
            }
            
            match found {
                Some(f) => f,
                none => return,
            }
        };
        
        let _ = component.defer_ephemeral(&ctx.http).await;
        
        let user_id = component.user.id.to_string();
        
        let player = match self.shared_state.db.get_player_by_discord_id(&user_id).unwrap() {
            Some(p) => p,
            none => {
                let content = CreateInteractionResponseFollowup::new()
                    .content("ไม่พบข้อมูลผู้เล่นของคุณในระบบ! กรุณาลงทะเบียนด้วย !register <steam_id>")
                    .ephemeral(true);
                let _ = component.create_followup(&ctx.http, content).await;
                return;
            }
        };
        
        if component.channel_id.get() != self.shared_state.config.exempt_channel_id {
            if let Some(remaining) = self.shared_state.check_cooldown(&user_id, custom_id) {
                let content = CreateInteractionResponseFollowup::new()
                    .content(format!("สินค้าดังกล่าวอยู่ในช่วง cooldown กรุณารออีก {} วินาที", remaining.as_secs()))
                    .ephemeral(true);
                let _ = component.create_followup(&ctx.http, content).await;
                return;
            }
        }
        
        let vip_info = self.get_vip_tier(&component.member.as_ref().unwrap().roles);
        let discount = vip_info.as_ref().map(|v| v.discount).unwrap_or(0.0);
        let (_original_price, discounted_price, _discount_percent) = 
            calculate_discounted_price(item.price, button.quantity, discount);
        
        if player.coin < discounted_price as i32 {
            let content = CreateInteractionResponseFollowup::new()
                .content(format!("คุณมี coin ไม่พอ (ต้องการ {}, มี {})", discounted_price, player.coin))
                .ephemeral(true);
            let _ = component.create_followup(&ctx.http, content).await;
            return;
        }
        
        if !self.shared_state.db.remove_coin(&user_id, discounted_price as i32).unwrap_or(false) {
            let content = CreateInteractionResponseFollowup::new()
                .content("เกิดข้อผิดพลาดในการหัก coin!")
                .ephemeral(true);
            let _ = component.create_followup(&ctx.http, content).await;
            return;
        }
        
        let mut commands_with_steam = substitute_steam_id_in_commands(&button.commands, &player.steam_id);
        
        for cmd in &mut commands_with_steam {
            if !is_special_command(cmd, &self.shared_state.config.special_commands) && !cmd.contains("Location") {
                cmd.push_str(&format!(" Location {}", player.steam_id));
            }
        }
        
        {
            let mut queue = self.shared_state.command_queue.lock().await;
            queue.push(commands_with_steam);
        }
        
        let _ = self.shared_state.db.log_purchase(&user_id, &player.steam_id, &item.name, discounted_price as i32);
        
        if component.channel_id.get() != self.shared_state.config.exempt_channel_id {
            self.shared_state.set_cooldown(&user_id, custom_id);
        }
        
        let content = CreateInteractionResponseFollowup::new()
            .content(format!("✅ ซื้อ {} สำเร็จ! หัก {} coins (เหลือ {} coins)", 
                item.name, discounted_price, player.coin - discounted_price as i32))
            .ephemeral(true);
        let _ = component.create_followup(&ctx.http, content).await;
        
        if let Ok(dm_channel) = component.user.create_dm_channel(&ctx.http).await {
            let embed = CreateEmbed::new()
                .title("⚡แจ้งเตือนการหัก Coin")
                .color(0x9900cc)
                .field("🛒 สินค้าที่ซื้อ", format!("**{}** x{}", item.name, button.quantity), false)
                .field("💷 ราคา", format!("**{}** coin", discounted_price), true)
                .field("💷 Coin คงเหลือ", format!("**{}** coin", player.coin - discounted_price as i32), false)
                .footer(CreateEmbedFooter::new("© powered by TimeSkip"));
            
            let message = CreateMessage::new().embed(embed);
            let _ = dm_channel.send_message(&ctx.http, message).await;
        }
    }
    
    fn get_vip_tier(&self, roles: &[RoleId]) -> Option<&VipRole> {
        let mut highest_tier: Option<&VipRole> = None;
        
        for role_id in roles {
            if let Some(vip_info) = self.shared_state.config.vip_roles.get(&role_id.get()) {
                if highest_tier.is_none() || vip_info.tier > highest_tier.unwrap().tier {
                    highest_tier = Some(vip_info);
                }
            }
        }
        
        highest_tier
    }
}

pub async fn process_command_queue(shared_state: Arc<SharedState>) {
    loop {
        sleep(Duration::from_millis(100)).await;
        
        let commands = {
            let mut queue = shared_state.command_queue.lock().await;
            if queue.is_empty() {
                continue;
            }
            queue.drain(..).collect::<Vec<_>>()
        };
        
        for command_set in commands {
            info!("Processing botshop commands from queue: {:?}", command_set);
            send_commands_to_game(command_set, "normal").await;
            sleep(Duration::from_millis(5)).await;
        }
    }
}

pub async fn auto_destroy_items_type1(shared_state: Arc<SharedState>) {
    loop {
        sleep(Duration::from_secs(1800)).await; 
        
        let _guard = shared_state.destroy_lock.lock().await;
        
        info!("👕 Starting automatic clothes/outfit destruction");
        
        send_commands_to_game(
            vec!["#Announce บอทกำลังลบชุด/เสื้อผ้าอัตโนมัติ บอทหยุดชั่วคราว!".to_string()],
            "destroy"
        ).await;
        
        sleep(Duration::from_secs(1)).await;
        
        send_commands_to_game(
            shared_state.config.destroy_commands_type1.clone(),
            "destroy"
        ).await;
        
        send_commands_to_game(
            vec!["#Announce บอทลบชุด/เสื้อผ้าอัตโนมัติสำเร็จ สามารถใช้บอทต่อได้!".to_string()],
            "destroy"
        ).await;
        
        info!("✅ Automatic clothes/outfit destruction completed");
    }
}

pub async fn auto_destroy_items_type2(shared_state: Arc<SharedState>) {
    loop {
        sleep(Duration::from_secs(7200)).await; 
        
        let _guard = shared_state.destroy_lock.lock().await;
        
        info!("🔨 Starting automatic construction materials destruction");
        
        send_commands_to_game(
            vec!["#Announce บอทกำลังลบวัสดุก่อสร้าง บอทหยุดชั่วคราว!".to_string()],
            "destroy"
        ).await;
        
        sleep(Duration::from_secs(1)).await;
        
        send_commands_to_game(
            shared_state.config.destroy_commands_type2.clone(),
            "destroy"
        ).await;
        
        send_commands_to_game(
            vec!["#Announce บอทลบวัสดุก่อสร้าง สามารถใช้บอทต่อได้!".to_string()],
            "destroy"
        ).await;
        
        info!("✅ Automatic construction materials destruction completed");
    }
}