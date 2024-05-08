use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{services::{event_bus::EventBus, websocket::WebsocketService}, User};

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
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
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
                                    "https://api.dicebear.com/8.x/identicon/svg?seed={}",
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
                    //log::debug!("got input: {:?}", input.value());
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
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-gray-100 dark:bg-slate-950 border-r border-gray-300 dark:border-slate-700">
                    <div class="w-full">
                        <div class="text-sm p-3 font-semibold text-center">{"People"}</div>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex bg-white dark:bg-slate-900 p-2 border-y border-gray-300 dark:border-slate-700 items-center">
                                    <div>
                                        <img class="h-12 aspect-square rounded-full border border-gray-300 dark:border-slate-700" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-500 dark:text-slate-300">
                                            {"Online"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col dark:bg-slate-900">
                    <div class="w-full h-14 border-gray-300 dark:border-slate-700"><div class="text-xl font-semibold p-3 ps-6">{"#general"}</div></div>
                    <div class="w-full grow overflow-auto p-8 pb-0 border-b border-gray-300 dark:border-slate-700">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="mb-6 flex items-center">
                                        <img class="w-8 me-4 aspect-square rounded-full border border-gray-300 dark:border-slate-700" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="flex items-end bg-gray-100 dark:bg-slate-700 rounded-tl-xl rounded-tr-xl rounded-br-xl">
                                            <div class="p-3">
                                                <div class="text-xs text-gray-500 dark:text-slate-300 font-semibold">
                                                    {m.from.clone()}
                                                </div>
                                                <div class="text-sm text-gray-800 dark:text-white">
                                                    if m.message.ends_with(".gif") {
                                                        <img class="mt-3" src={m.message.clone()}/>
                                                    } else {
                                                        {m.message.clone()}
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }

                    </div>
                    <div class="w-full h-14 flex px-2 py-2 items-center bg-gray-100 border-gray-300 dark:border-slate-700 dark:bg-slate-950">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class="block w-full me-2 py-2 pl-4 bg-white dark:bg-slate-700 rounded-md outline-none" name="message" required=true />
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-md flex justify-center items-center color-white">
                            <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}