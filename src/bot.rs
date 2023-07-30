use serde_json::json;
use tg_flows::{listen_to_update, Telegram, Update, UpdateKind};
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use store_flows::{get, set};
use flowsnet_platform_sdk::logger;

use crate::llm;
use llm::{get_prompt_from_bytes,form_prompt_new_idea,form_prompt_update_idea};

const STATE_KEY_STR: &str = "state";
const RESTART_KEY_STR: &str = "is_restart";


#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let placeholder_text = std::env::var("placeholder").unwrap_or("Typing ...".to_string());

    const SYSTEM_PROMPT_BYTES: &[u8] = include_bytes!("../prompts/system_prompt.md");
    let system_prompt : String = get_prompt_from_bytes(SYSTEM_PROMPT_BYTES);
    const HELP_MESG_BYTES: &[u8] = include_bytes!("../prompts/help_mesg.md");
    let help_mesg : String = get_prompt_from_bytes(HELP_MESG_BYTES);
    log::info!("Bot initialized successfully");
    listen_to_update(&telegram_token, |update| {
        let tele = Telegram::new(telegram_token.to_string());
        log::info!("Received update from {}",telegram_token.to_string());
        handler(tele, &placeholder_text, &system_prompt, &help_mesg, update)
    }).await;

    Ok(())
}

async fn handler(tele: Telegram, placeholder_text: &str, system_prompt: &str, help_mesg: &str, update: Update) {
    if let UpdateKind::Message(msg) = update.kind {
        let chat_id = msg.chat.id;
        log::info!("Received message from {}", chat_id);

        let mut openai = OpenAIFlows::new();
        openai.set_retry_times(3);
        let mut co = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: false,
            system_prompt: Some(system_prompt),
        };

        let text = msg.text().unwrap_or("");
        if text.eq_ignore_ascii_case("/help") {
            _ = tele.send_message(chat_id, help_mesg);

        } else if text.eq_ignore_ascii_case("/start") {
            _ = tele.send_message(chat_id, help_mesg);
            let mut user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
            user_data[RESTART_KEY_STR] = json!(true);
            set(&chat_id.to_string(), user_data, None);
            log::info!("Started converstion for {}", chat_id);

        } else if text.eq_ignore_ascii_case("/restart") {
            _ = tele.send_message(chat_id, "Ok, I am starting a new conversation.");
            let mut user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
            user_data[RESTART_KEY_STR] = json!(true);
            set(&chat_id.to_string(), user_data, None);
            log::info!("Restarted converstion for {}", chat_id);
        } else if text.starts_with("/check") {
            let user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
            let current_state = user_data[STATE_KEY_STR].as_str().unwrap_or("");
            _ = tele.send_message(chat_id, format!("Your ideas so far: \n{}",current_state.clone()));
        
        } else {
            let placeholder = tele
                .send_message(chat_id, placeholder_text)
                .expect("Error occurs when sending Message to Telegram");

                let user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
                let restart = user_data[RESTART_KEY_STR].as_bool().unwrap_or(false);
                let current_state = user_data[STATE_KEY_STR].as_str().unwrap_or("");

            if restart {
                log::info!("Detected restart = true");
                let mut user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
                user_data[RESTART_KEY_STR] = json!(false);
                set(&chat_id.to_string(), user_data, None);
                co.restart = true;
            }
            
            
            let text_ref = if text.starts_with("/new") {
                let command_text = &text[4..];
                form_prompt_new_idea(command_text,current_state)
            } else if text.starts_with("/update") {
                let command_text = &text[7..];
                form_prompt_update_idea(command_text,current_state)
            } else {
                text.to_string()
            };
            match openai.chat_completion(&chat_id.to_string(), &text_ref, &co).await {
                Ok(r) => {
                    let mut result = r.choice;
                    if let Some(suffix) = result.strip_prefix("Result:") {
                        if suffix.starts_with('\n') {
                            result = (&suffix[1..]).to_string();
                        }
                        let mut user_data = get(&chat_id.to_string()).unwrap_or(json!({}));
                        user_data[STATE_KEY_STR] = json!(result.clone());
                        set(&chat_id.to_string(), user_data, None);
                        _ = tele.edit_message_text(chat_id, placeholder.id, result);
                    } else {
                        _ = tele.edit_message_text(chat_id, placeholder.id, result);
                    }
                }
                Err(e) => {
                    _ = tele.edit_message_text(chat_id, placeholder.id, "Sorry, an error has occured. Please try again later!");
                    log::error!("OpenAI returns error: {}", e);
                }
            }
        }
    }
    else {
        log::error!("handler failed to run");
    }
}
