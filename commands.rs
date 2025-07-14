use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::builder::*;

pub async fn handle_destroy_command(ctx: &Context, msg: &Message) {
    let embed = CreateEmbed::new()
        .title("🧹 ระบบลบชุด")
        .description("กดปุ่มด้านล่างเพื่อลบชุดต่างๆ ในเกม")
        .color(0x00ff00)
        .field(
            "ℹ️ ข้อมูล",
            "• ลบชุด/เสื้อผ้า: อัตโนมัติทุก 30 นาที\n• ลบวัสดุก่อสร้าง: อัตโนมัติทุก 120 นาที\n• คุณสามารถกดปุ่มเพื่อลบด้วยตนเองได้",
            false
        )
        .footer(CreateEmbedFooter::new("© powered by TimeSkip"));
    
    let components = vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("destroy_type1")
                .label("ลบชุด")
                .style(ButtonStyle::Success),
            CreateButton::new("destroy_type2")
                .label("ลบวัสดุ")
                .style(ButtonStyle::Danger),
        ])
    ];
    
    let _ = msg.channel_id.send_message(&ctx.http, CreateMessage::new()
        .embed(embed)
        .components(components)
    ).await;
}