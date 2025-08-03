use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    use crate::app::codee::string::FromToStringCodec;

    //let (id, _set_id) = leptos_use::use_cookie::<String, FromToStringCodec>("trace-viewer");
    view! {
        <div class = "topbar">
            <h1>"Trace Viewer"</h1>
            <div class = "menu">
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </div>
            <div class = "stats">
                //{move || id.get().map(|str| view!{ "Id: " {str} })}
            </div>
        </div>
    }
}
