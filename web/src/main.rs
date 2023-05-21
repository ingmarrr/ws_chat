use leptos::ev::{MouseEvent, SubmitEvent};
use leptos::*;
use leptos::html::Input;
use leptos_meta::*;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "ws_chat=info, tower=debug, web=info",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    mount_to_body(|cx| view! { cx, <Chat/> })
}

async fn join_chat(name: String) -> anyhow::Result<()> {
    Ok(())
}

#[component]
pub fn Chat(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::new());
    let (msg, set_msg) = create_signal(cx, String::new());
    let (msgs, set_msgs) = create_signal(cx, Vec::<String>::new());
    let (joined, set_joined) = create_signal(cx, false);

    let join = create_action(cx, |name: &String| {
        tracing::info!("{name}");
        join_chat(name.clone())
    });


    view! { cx,

        <Stylesheet id="leptos" href="/tailwind.css"/>
        <Title text="Ws Chat"/>

        <div>
            <ChatForm/>
        </div>
        // <div>
        //     <JoinForm set_joined=set_joined/>
        //     <SendForm joined=joined/>
        // </div>
    }
}


#[component]
pub fn ChatForm(
    cx: Scope,
) -> impl IntoView {
    let (name, set_name)     = create_signal(cx, String::new());
    let (msg, set_msg)       = create_signal(cx, String::new());
    let (joined, set_joined) = create_signal(cx, false);
    let (msgs, set_msgs)     = create_signal(cx, Vec::<Msg>::new());

    let name_inel: NodeRef<Input> = create_node_ref(cx);
    let msg_inel : NodeRef<Input> = create_node_ref(cx);

    let on_join = move |ev: SubmitEvent| {
        ev.prevent_default();
        let val = name_inel()
            .unwrap()
            .value();
        set_joined(true);
        set_name(val)
    };
    let on_msg  = move |ev: SubmitEvent| {
        ev.prevent_default();
        let val = msg_inel()
            .unwrap()
            .value();
        tracing::info!("Pushing message");
        set_msg(val.clone());
        set_msgs.update(move |msgs| msgs.push(Msg {
            msg: val,
            id: msgs.len(),
        }))
    };

    let msg_box = move || {
        if joined.get() {
            view! { cx, 
                <div>
                    <form on:submit=on_msg>
                    <input type="text"
                        value=msg
                        node_ref=msg_inel
                        prop:value=msg
                    />
                    <input type="submit" value="Send"/>
                    </form>
                </div>
            }
        } else {
            view! { cx,
                <div><p>"Join to view messages"</p></div>
            }
        }
    };

    view! { cx,

        <form on:submit=on_join>
            <input type="text"
                value=name
                node_ref=name_inel
                prop:value=name
            />
            <input type="submit" value="Submit"/>
        </form>
        
        {msg_box}

        <For
            each=msgs
            key=|msg| msg.id
            view=move |cx, msg: Msg| {
                view! { cx, <p> {msg} </p> }
            }
        />
    }
}


#[component]
pub fn JoinForm(
    cx: Scope, 
    set_joined: WriteSignal<bool>,
) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::new());
    let input_el: NodeRef<Input> = create_node_ref(cx);
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let val = input_el()
            .expect("<input> to exist.")
            .value();
        set_joined(true);
        set_name(val);
    };

    view! { cx,
        <form on:submit=on_submit>
            <input type="text"
                value=name
                node_ref=input_el
            />
            <input type="submit" value="Submit"/>
        </form>
    }
}

#[component]
pub fn SendForm(
    cx: Scope, 
    joined: ReadSignal<bool>,
) -> impl IntoView {
    let (name, set_name) = create_signal(cx, String::new());
    let input_el: NodeRef<Input> = create_node_ref(cx);
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let val = input_el()
            .expect("<input> to exist.")
            .value();
        set_name(val);
    };

    if !joined.get() {
        return view! { cx, <div><p>"Join to view messages"</p></div>};
    }

    let (mut soc, resp) = tungstenite::connect("ws://127.0.0.1:3001")
        .expect("Cant connect");

    // let res = soc.write(tungstenite::Message::Text(val)).unwrap();
    // let res = soc.write_message(tungstenite::Message::Text(val))
    // loop {
    //     let msg = soc.read_message().expect("Error reading message.");

    // }

    // let msges = move || {
    //     if joined.get() {
    //         view! { cx, 
    //         <div>
    //             <form on:submit=on_submit>
    //             <input type="text"
    //                 value=name
    //                 node_ref=input_el
    //             />
    //             <input type="submit" value="Send"/>
    //         </form>
    //             </div>}
    //     } else {
    //         view! { cx,
    //             <div><p>"Join to view messages"</p></div>
    //         }
    //     }
    // };

    view! { cx,
        <div>
            // {msges}
            <form on:submit=on_submit>
                <input type="text"
                    value=name
                    node_ref=input_el
                />
                <input type="submit" value="Send"/>
            </form>
        </div>
    }
}


#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Msg {
    pub msg: String,
    pub id: usize,
}

impl IntoView for Msg {
    fn into_view(self, cx: Scope) -> View {
        view! { cx,
            <p> {self.id} " >> " {self.msg}</p>
        }.into_view(cx)
    }
}


