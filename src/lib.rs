use serde_json::json;
use tg_flows::{listen_to_update, Telegram, Update, UpdateKind};
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use store_flows::{get, set};
use flowsnet_platform_sdk::logger;
use std::env;
use std::path::{Path, PathBuf};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let placeholder_text = std::env::var("placeholder").unwrap_or("Typing ...".to_string());
    log::info!("Before read");
    let base_path = Path::new(file!()).parent().unwrap().to_owned();
    let relative_path = Path::new("./prompts/system_prompt.md");
    let path = base_path.join(relative_path).canonicalize().unwrap();

    if path.exists() {
        log::info!("File exists!");
    } else {
        log::info!("File does not exist!");
    }
    const SYSTEM_PROMPT: &[u8] = include_bytes!("../prompts/system_prompt.md");
    let system_prompt = std::fs::read_to_string("prompts/system_prompt.md")?.trim().to_string();// failed here
    let help_mesg = std::env::var("help_mesg").unwrap_or("I am your assistant on Telegram. Ask me any question! To start a new conversation, type the /restart command.".to_string());
    log::info!("Start");
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
            set(&chat_id.to_string(), json!(true), None);
            log::info!("Started converstion for {}", chat_id);

        } else if text.eq_ignore_ascii_case("/restart") {
            _ = tele.send_message(chat_id, "Ok, I am starting a new conversation.");
            set(&chat_id.to_string(), json!(true), None);
            log::info!("Restarted converstion for {}", chat_id);

        } else {
            let placeholder = tele
                .send_message(chat_id, placeholder_text)
                .expect("Error occurs when sending Message to Telegram");

            let restart = match get(&chat_id.to_string()) {
                Some(v) => v.as_bool().unwrap_or_default(),
                None => false,
            };
            if restart {
                log::info!("Detected restart = true");
                set(&chat_id.to_string(), json!(false), None);
                co.restart = true;
            }
            // TODO
            
            // if text is start with command, fill text(remove the command part) into command-specific template
                // get parsed action
                // call operation using parsed action
            // else reply with help, don't call openai
            
            match openai.chat_completion(&chat_id.to_string(), &text, &co).await {
                Ok(r) => {
                    _ = tele.edit_message_text(chat_id, placeholder.id, r.choice);
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
