use yew::prelude::*;
use yew::Properties;
use yew::events::Event;
use boolinator::{Boolinator};
use std::fmt::{Formatter, Display};
use web_sys::{EventTarget, HtmlInputElement};
use wasm_bindgen::JsCast;

pub enum MaterialTextFieldMessage {
    ChangeValue(String)
}

#[derive(Clone, PartialEq, Debug)]
pub enum MaterialTextFieldIconStyle {
    Leading,
    Trailing,
}

impl Display for MaterialTextFieldIconStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterialTextFieldIconStyle::Leading => write!(f, "leading"),
            MaterialTextFieldIconStyle::Trailing => write!(f, "trailing")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct MaterialTextFieldIcon {
    pub style: MaterialTextFieldIconStyle,
    pub icon: String,
    pub on_click: Option<Callback<MouseEvent>>,
    pub background_color: Option<String>,
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct MaterialTextFieldProps {
    // The value to display in the text field
    pub value: String,
    // The label for the text field. If left as None a label will not be displayed
    // Defaults to None
    #[prop_or_default]
    pub label: Option<String>,
    // The id for the control to coordinate the label and input
    #[prop_or_default]
    pub id: String,
    // Whether the input should be rendered as disabled
    // Defaults to false
    #[prop_or(false)]
    pub disabled: bool,
    // The callback function to be notified of the changed value
    // Should not be set if text field is disabled
    #[prop_or_default]
    pub on_change: Callback<String>,
    // Whether the text in the input should be positioned at the end (right for LTR and left for RTL)
    #[prop_or(false)]
    pub end_aligned: bool,
    // Option to set an icon inside the text field (leading or trailing)
    // Defaults to None
    #[prop_or_default]
    pub icon: Option<MaterialTextFieldIcon>,
}

pub struct MaterialTextField;

impl Component for MaterialTextField {
    type Message = MaterialTextFieldMessage;
    type Properties = MaterialTextFieldProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MaterialTextFieldMessage::ChangeValue(value) => {
                ctx.props().on_change.emit(value);
                false
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self::Properties { id, value, label, disabled, end_aligned, icon, .. } = ctx.props().clone();
        let onchange = ctx.link().batch_callback(|event: Event| {
            let target: Option<EventTarget> = event.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            input.map(|input| MaterialTextFieldMessage::ChangeValue(input.value()))
        });
        let classes = classes!("mdc-text-field", 
            "mdc-text-field--filled", 
            disabled.as_some("mdc-text-field--disabled"), 
            label.is_none().as_some("mdc-text-field--no-label"),
            end_aligned.as_some("mdc-text-field--end-aligned"),
            icon.clone().map(|value| format!("mdc-text-field--with-{}-icon", value.style))
        );
        html! {
            <label class={classes}>
                <span class="mdc-text-field__ripple"></span>
                { render_label(label, id.clone()) }
                { render_icon(icon) }
                <input class="mdc-text-field__input" type="text" disabled={disabled} value={value} onchange={onchange} aria-labelledby={id.clone()} />
                <span class="mdc-line-ripple"></span>
            </label>
        }
    }
}

fn render_label(label: Option<String>, id: String) -> Html {
    match label {
        None => html! {},
        Some(value) => html! {
            <span class="mdc-floating-label" id={id}>{ value }</span>
        }
    }
}

fn render_icon(icon: Option<MaterialTextFieldIcon>) -> Html {
    match icon {
        None => html! {},
        Some(value) => {
            let icon_classes = classes!(
                "material-icons",
                "mdc-text-field__icon",
                format!("mdc-text-field__icon--{}", value.style)
            );
            let style = match value.background_color {
                Some(background_color) => {
                    format!("background-color: {}", background_color)
                },
                None => "".to_string()
            };
            
            
            match value.on_click {
                Some(callback) => html! {
                    <i class={icon_classes} style={style} tabindex="0" role="button" onclick={callback}>{ value.icon }</i>
                },
                None => html! {
                    <i class={icon_classes} style={style}>{ value.icon }</i>
                }
            }            
        }
    }
}