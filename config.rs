use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub price: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub buttons: Vec<ShopButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopButton {
    pub text: String,
    pub trigger: String,
    pub commands: Vec<String>,
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopConfig {
    pub channel: String,
    pub items: Vec<ShopItem>,
}

#[derive(Debug, Clone)]
pub struct VipRole {
    pub tier: u8,
    pub discount: f32,
    pub name: String,
}

pub struct Config {
    pub shop_data: Vec<ShopConfig>,
    pub vip_roles: HashMap<u64, VipRole>,
    pub exempt_channel_id: u64,
    pub destroy_commands_type1: Vec<String>,
    pub destroy_commands_type2: Vec<String>,
    pub special_commands: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Loading shop data from botshop.json...");
        
        if !std::path::Path::new("botshop.json").exists() {
            return Err("botshop.json file not found!".into());
        }

        let shop_data_str = match std::fs::read_to_string("botshop.json") {
            Ok(data) => {
                println!("Shop data file read successfully");
                data
            },
            Err(e) => {
                return Err(format!("Failed to read botshop.json: {}", e).into());
            }
        };
        
        println!("Parsing JSON data...");
        
        let shop_data: Vec<ShopConfig> = match serde_json::from_str(&shop_data_str) {
            Ok(data) => {
                println!("JSON parsed successfully");
                data
            },
            Err(e) => {
                return Err(format!("Failed to parse botshop.json: {}", e).into());
            }
        };
        
        println!("Loaded {} shops", shop_data.len());
        
        let mut vip_roles = HashMap::new();
        vip_roles.insert(1375091477448888412, VipRole { tier: 1, discount: 0.0, name: "Silver".to_string() });
        vip_roles.insert(1345511219263569984, VipRole { tier: 2, discount: 0.30, name: "Gold".to_string() });
        vip_roles.insert(1375090778254217317, VipRole { tier: 3, discount: 0.50, name: "Platinum".to_string() });
        vip_roles.insert(1381346983649874030, VipRole { tier: 4, discount: 0.60, name: "Diamond".to_string() });
        
        Ok(Config {
            shop_data,
            vip_roles,
            exempt_channel_id: 1381383699320537209,
            destroy_commands_type1: vec![
                "#DestroyAllItemsWithinRadius Rag_Stripes 9999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Rags 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Peniswarmer_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wool_Gloves_01_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Inmate_Hoodie_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Inmate_shirt_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Scum_Shirt_Event_Black 99999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Scum_Shirt_Event_White 999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Scum_Shirt_Event_Orange 999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Inmate_pants 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Underpants_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Scum_Shirt_Supporter_Pack_Black_01 99999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius F_Undershirt_Bra_01 999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Scum_Shirt_Event_Black 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Boxer_Briefs_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius HighTop_Shoes 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Sock_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Danny_Trejo_Vest 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Danny_Trejo_Pants 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 1H_DannyMachete 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Danny_Trejo_Glove_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Danny_Trejo_Boots_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Beanie_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Beanie_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Parachute 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Mask_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius F_Bra_Supporter_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Undershirt_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius LuisMoncada_Jacket 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius LuisMoncada_Pants 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius LuisMoncada_Boots 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 2H_La_Hacha_Axe 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Raymond_Cruz_Boots 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Raymond_Cruz_Hat 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Raymond_Cruz_Pants 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Raymond_Cruz_Shirt 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 1H_RaymondCruz_Knife 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Undershirt_01 999999999999999999999".to_string(),
            ],
            destroy_commands_type2: vec![
                "#DestroyAllItemsWithinRadius Rope1 99999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Paper 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius PETBottle04 999999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Sock_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Stick 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Bundle_Wooden_Plank 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Log_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Log_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Long_wooden_stick 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Paper 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Plank 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Beanie_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Beanie_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Parachute 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Military_Mask_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius F_Bra_Supporter_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Undershirt_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Log_Small_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Log_Small_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wooden_Log_Small_03 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Metal_Scrap_02 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Metal_Scrap_03 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Metal_Scrap_01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Metal_Scrap_04 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Metal_Scrap_05 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius PETBottle01 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Brick 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 2H_Axe 9999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 2H_La_Hacha_Axe 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius 1H_RaymondCruz_Knife 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Inmate_Hoodie_01 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Wool_Gloves_01_01 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Undershirt_01 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Bolts_Package_Box 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Rope 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Nails_Package_Box 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius CementBag 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius GravelBag 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius SandBag 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Barbed_Wire 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Sledgehammer 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Nails 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius Bolts 999999999999999999999".to_string(),
                "#DestroyAllItemsWithinRadius EmptyBag 999999999999999999999".to_string(),
            ],
            special_commands: vec![
                "ChangeCurrencyBalance".to_string(),
                "ChangeFamePoints".to_string(),
            ],
        })
    }
}