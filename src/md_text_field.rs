use yew::prelude::*;
use yew::Properties;
use boolinator::{Boolinator};

pub enum MaterialTextFieldMessage {
    ChangeValue(String)
}

#[derive(Properties, Clone, PartialEq)]
pub struct MaterialTextFieldProps {
    // The value to display in the text field
    pub value: String,
    // The label for the text field
    pub label: String,
    // The id for the control to coordinate the label and input
    pub id: String,
    // Whether the input should be rendered as disabled
    // Defaults to false
    #[prop_or(false)]
    pub disabled: bool,
    // The callback function to be notified of the changed value
    // Should not be set if text field is disabled
    #[prop_or_default]
    pub on_change: Callback<String>
}

pub struct MaterialTextField {
    link: ComponentLink<Self>,
    properties: MaterialTextFieldProps
}

impl Component for MaterialTextField {
    type Message = MaterialTextFieldMessage;
    type Properties = MaterialTextFieldProps;    

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            properties: props
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            MaterialTextFieldMessage::ChangeValue(value) => { 
                self.properties.on_change.emit(value);
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.value != self.properties.value {
            self.properties.value = props.value;
            return true;
        }
        false
    }

    fn view(&self) -> Html {
        let Self::Properties { id, value, label, disabled, .. } = self.properties.clone();
        let onchange = self.link.batch_callback(|data: ChangeData| {
            match data {
                ChangeData::Value(value) => Some(MaterialTextFieldMessage::ChangeValue(value)),
                _ => None
            }
        });
        html! {
            <label class=classes!("mdc-text-field", "mdc-text-field--filled", disabled.as_some("mdc-text-field--disabled"))>
                <span class="mdc-text-field__ripple"></span>
                <span class="mdc-floating-label" id=id.clone()>{ label }</span>
                <input class="mdc-text-field__input" type="text" disabled=disabled value=value onchange=onchange aria-labelledby=id.clone() />
                <span class="mdc-line-ripple"></span>
            </label>
        }
    }
}