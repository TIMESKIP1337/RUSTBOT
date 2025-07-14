use chrono::{NaiveTime, Local, Duration as ChronoDuration};
use chrono_tz::Asia::Bangkok;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use log::info;

use crate::shared_state::SharedState;
use crate::utils::send_commands_to_game;

pub async fn start_maintenance_schedule(shared_state: Arc<SharedState>) {
    let restart_times = vec![
        NaiveTime::from_hms_opt(23, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(3, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(7, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(11, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(15, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(19, 58, 0).unwrap(),
        NaiveTime::from_hms_opt(21, 58, 0).unwrap(),
    ];
    
    let resume_times = vec![
        NaiveTime::from_hms_opt(0, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(4, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(8, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(12, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(16, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(20, 6, 0).unwrap(),
        NaiveTime::from_hms_opt(22, 6, 0).unwrap(),
    ];
    
    loop {
        let now = Local::now().with_timezone(&Bangkok);
        let current_time = now.time();
        
        let mut next_stop = None;
        for &time in &restart_times {
            if time > current_time {
                next_stop = Some(time);
                break;
            }
        }
        if next_stop.is_none() {
            next_stop = Some(restart_times[0]);
        }
        
        let mut next_resume = None;
        for &time in &resume_times {
            if time > current_time {
                next_resume = Some(time);
                break;
            }
        }
        if next_resume.is_none() {
            next_resume = Some(resume_times[0]);
        }
        
        let next_stop = next_stop.unwrap();
        let next_resume = next_resume.unwrap();
        
        let duration_to_stop = if next_stop > current_time {
            next_stop - current_time
        } else {
            (next_stop + ChronoDuration::days(1)) - current_time
        };
        
        let duration_to_resume = if next_resume > current_time {
            next_resume - current_time
        } else {
            (next_resume + ChronoDuration::days(1)) - current_time
        };
        
        if duration_to_stop < duration_to_resume {
            let sleep_secs = duration_to_stop.num_seconds() as u64;
            info!("Bot will stop at {:?} (in {} seconds)", next_stop, sleep_secs);
            sleep(Duration::from_secs(sleep_secs)).await;
            
            shared_state.set_bot_active(false).await;
            info!("🛑 Bot stopped for server restart");
            
            send_commands_to_game(
                vec!["#Announce BOTSHOP หยุดทำงานชั่วคราว เนื่องจาก SERVER กำลังจะ RESTART ในอีก 2 นาที".to_string()],
                "normal"
            ).await;
            
            let now = Local::now().with_timezone(&Bangkok);
            let duration_to_resume = if next_resume > now.time() {
                next_resume - now.time()
            } else {
                (next_resume + ChronoDuration::days(1)) - now.time()
            };
            let sleep_secs = duration_to_resume.num_seconds() as u64;
            info!("Bot will resume at {:?} (in {} seconds)", next_resume, sleep_secs);
            sleep(Duration::from_secs(sleep_secs)).await;
            
            shared_state.set_bot_active(true).await;
            info!("✅ Bot resumed normal operation");
            
            send_commands_to_game(
                vec!["#Announce BOTSHOP กลับมาทำงานปกติแล้ว ลุยยยยย".to_string()],
                "normal"
            ).await;
        } else {
            let sleep_secs = duration_to_resume.num_seconds() as u64;
            info!("Waiting for next event (in {} seconds)", sleep_secs);
            sleep(Duration::from_secs(sleep_secs)).await;
        }
    }
}