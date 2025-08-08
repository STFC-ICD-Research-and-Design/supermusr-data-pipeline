mod control_box;
mod display_errors;
mod section;
mod topbar;

pub(crate) use control_box::{InputBoxWithLabel, SubmitBox};
pub(crate) use display_errors::DisplayErrors;
use leptos::{logging, tachys::renderer::dom::Element};
pub(crate) use section::Section;
pub(crate) use topbar::TopBar;

//use leptos::{IntoView, component, prelude::*, view};

pub(crate) fn build_classes_string(main: &'static str, mut classes: Vec<&'static str>) -> String {
    classes.push(main);
    classes
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn toggle_closed(element: Option<Element>) {
    if let Err(e) = element
        .expect("Parent element should exist, this should never fail.")
        .class_list()
        .toggle("closed")
    {
        if let Some(js) = e.as_string() {
            logging::warn!("JsValue: {js}");
        } else {
            logging::warn!("Cannot display JsValue error");
        }
    }
}
