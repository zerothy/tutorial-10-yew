use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen chat-bg">
                // User sidebar
                <div class="flex-none w-64 h-screen user-list overflow-hidden">
                    <div class="text-xl p-4 font-semibold border-b border-gray-200 flex items-center">
                        <span class="mr-2">{"👥"}</span>
                        <span>{"Users"}</span>
                    </div>
                    <div class="overflow-y-auto h-full">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex m-3 bg-white rounded-xl p-3 shadow-sm user-item">
                                        <div>
                                            <img class="w-12 h-12 rounded-full user-avatar" src={u.avatar.clone()} alt="avatar"/>
                                        </div>
                                        <div class="flex-grow p-2 ml-2">
                                            <div class="flex text-sm font-medium justify-between">
                                                <div>{u.name.clone()}</div>
                                            </div>
                                            <div class="text-xs text-gray-400 mt-1">
                                                {"Online"}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
                
                // Main chat area
                <div class="grow h-screen flex flex-col">
                    // Chat header
                    <div class="w-full h-16 chat-header flex items-center px-6 border-b border-gray-200">
                        <div class="text-xl font-semibold">{"💬 YewChat"}</div>
                        <div class="ml-3 text-sm text-gray-500">{"Let's chat!"}</div>
                    </div>
                    
                    // Messages container
                    <div class="w-full grow overflow-auto p-6 space-y-6">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap_or_else(|| {
                                    // Fallback for when user is not found
                                    &self.users[0]
                                });
                                
                                let is_current_user = false; // Replace with actual check when user context is available
                                
                                html!{
                                    <div class={if is_current_user { 
                                        "flex justify-end" 
                                    } else { 
                                        "flex" 
                                    }}>
                                        if !is_current_user {
                                            <img class="w-10 h-10 rounded-full user-avatar self-end mr-3" 
                                                 src={user.avatar.clone()} alt="avatar"/>
                                        }
                                        
                                        <div class={if is_current_user {
                                            "max-w-md bg-primary-light text-white rounded-2xl py-2 px-4 message-bubble"
                                        } else {
                                            "max-w-md bg-white rounded-2xl py-2 px-4 shadow-sm message-bubble"
                                        }}>
                                            if !is_current_user {
                                                <div class="font-medium text-sm mb-1">{m.from.clone()}</div>
                                            }
                                            
                                            if m.message.ends_with(".gif") {
                                                <img class="rounded-lg w-full" src={m.message.clone()}/>
                                            } else {
                                                <p class="text-sm">{m.message.clone()}</p>
                                            }
                                            
                                            <div class="text-xs text-right mt-1 message-time">
                                                {"Just now"}
                                            </div>
                                        </div>
                                        
                                        if is_current_user {
                                            <img class="w-10 h-10 rounded-full user-avatar self-end ml-3" 
                                                 src={user.avatar.clone()} alt="avatar"/>
                                        }
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Message input
                    <div class="w-full px-4 py-3 bg-white border-t border-gray-200 flex items-center">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Type a message..." 
                            class="block w-full py-3 px-4 bg-gray-50 rounded-full outline-none message-input" 
                            name="message" 
                            required=true 
                        />
                        <button 
                            onclick={submit} 
                            class="p-3 ml-3 bg-primary-dark hover:bg-primary-dark w-12 h-12 rounded-full flex justify-center items-center text-white send-button"
                        >
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white w-5 h-5">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
