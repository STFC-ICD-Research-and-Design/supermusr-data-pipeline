mod control_box;
mod section;
mod topbar;
mod display_errors;

pub(crate) use control_box::{ControlBoxWithLabel, InputBoxWithLabel, SubmitBox};
pub(crate) use section::{Panel, Section};
pub(crate) use topbar::TopBar;
pub(crate) use display_errors::DisplayErrors;

//use leptos::{IntoView, component, prelude::*, view};

pub(crate) fn build_classes_string(main: &'static str, mut classes: Vec<&'static str>) -> String {
    classes.push(main);
    classes
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}