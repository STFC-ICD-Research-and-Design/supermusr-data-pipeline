mod control_box;
mod topbar;
mod section;

pub(crate) use control_box::{ControlBoxWithLabel, InputBoxWithLabel, SubmitBox};
pub(crate) use topbar::TopBar;
pub(crate) use section::{Panel, Section};

//use leptos::{IntoView, component, prelude::*, view};

pub(crate) fn build_classes_string(main: &'static str, mut classes: Vec<&'static str>) -> String {
    classes.push(main);
    classes
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}