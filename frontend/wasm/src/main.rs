use stdweb::js;
use std::str::FromStr;

fn main() {
    let value = js!({
        return window.initial_counter_state;
    });
    let arg = match value {
        stdweb::Value::String(s) => {
            let initial_counter_state=u32::from_str(&s).expect("parsing provided u32 failed");
            app::Arg { initial_counter_state }
        }
        _ => panic!("unexpected type from handlebar template, FIXME"),
    };
    yew::start_app_with_props::<app::State>(arg);
}
