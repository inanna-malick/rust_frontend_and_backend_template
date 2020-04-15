#![recursion_limit = "512"]

use my_types::{IncrementReq, IncrementResp};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender, format::Json};

macro_rules! println {
    ($($tt:tt)*) => {{
        let msg = format!($($tt)*);
        js! { @(no_return) console.log(@{ msg }) }
    }}
}

#[derive(Debug)]
pub struct State {
    counter_state: u32,
    link: ComponentLink<Self>,
    fetch_service: FetchService,
    increment_task: Option<FetchTask>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Msg {
    TriggerIncrement(IncrementReq),
    OnIncrementResp(IncrementResp),
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Properties)]
pub struct Arg {
    pub initial_counter_state: u32,
}

impl Component for State {
    type Message = Msg;
    type Properties = Arg;

    fn create(arg: Self::Properties, link: ComponentLink<Self>) -> Self {
        State {
            counter_state: arg.initial_counter_state,
            fetch_service: FetchService::new(),
            increment_task: None,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TriggerIncrement(req) => {
                let request = Request::post("/increment/counter")
                    .header("Content-Type", "application/json")
                    .body(Json(&req))
                    .expect("push node request");

                let callback = self.link.callback(
                    move |response: Response<Json<Result<IncrementResp, anyhow::Error>>>| {
                        let (meta, Json(res)) = response.into_parts();
                        let body = res.expect("failure parsing resp body");
                        if meta.status.is_success() {
                            Msg::OnIncrementResp(body)
                        } else {
                            panic!("non-200 resp, {:?}", meta.status)
                        }
                    },
                );

                let task = self.fetch_service.fetch(request, callback).expect("creating task failed");
                self.increment_task = Some(task);

                // no need to redraw
                false
            }
            Msg::OnIncrementResp(resp) => {
                self.counter_state = resp.new_counter_state;
                self.increment_task = None;

                // state updated, trigger redraw
                true
            }
        }
    }

    fn view(&self) -> Html {
        html! {
        <div class="wrapper">
            <button onclick=self.link.callback( |_| Msg::TriggerIncrement(IncrementReq{increment_counter_by: 1}) )>
            {format!("counter state is {:?}, click to increment by 1", self.counter_state)}
            </button>
        </div>
        }
    }
}
