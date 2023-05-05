use endurance_racing_planner_common::User;
use uuid::Uuid;
use yew::prelude::*;
use yew_mdc::components::{
    button::{Button, Style},
    Dialog, TextField,
};

use crate::http::plans::{get_shared_users_for_plan, share_plan};

#[derive(Properties, PartialEq)]
pub struct SharePlanProps {
    pub plan_id: Uuid,
}

#[function_component(SharePlan)]
pub fn share_plan_component(props: &SharePlanProps) -> Html {
    let share_dialog_open = use_state_eq(|| false);
    let emails_to_share = use_state_eq(Vec::<String>::new);
    let current_email = use_state_eq(String::new);
    let existing_shared_users = use_state_eq(Vec::<String>::new);
    let plan_id = props.plan_id;

    let set_existing_emails = {
        let existing_shared_users = existing_shared_users.clone();
        Callback::from(move |shared_users: Vec<User>| {
            let existing_emails = shared_users
                .iter()
                .map(|u| u.email.clone())
                .collect::<Vec<_>>();
            existing_shared_users.set(existing_emails)
        })
    };

    let share_button = {
        let share_button_click = {
            let share_dialog_open = share_dialog_open.clone();
            Callback::once(move |_| {
                get_shared_users_for_plan(plan_id, set_existing_emails);
                share_dialog_open.set(true)
            })
        };
        html! {
            <Button id={"share-plan-button"} text={"Share"} style={Style::Raised} onclick={share_button_click} />
        }
    };

    let send_button_click = {
        let share_dialog_open = share_dialog_open.clone();
        let emails_to_share = emails_to_share.clone();
        Callback::from(move |_| {
            let emails = (*emails_to_share).clone();
            share_plan(plan_id, emails);
            share_dialog_open.set(false)
        })
    };

    let handle_email_change = {
        let current_email = current_email.clone();
        Callback::from(move |value| current_email.set(value))
    };
    let handle_email_enter = {
        let emails_to_share = emails_to_share.clone();
        let current_email = current_email.clone();
        Callback::from(move |keyboard_event: KeyboardEvent| {
            if keyboard_event.key() == "Enter" {
                let mut emails = (*emails_to_share).clone();
                let email = (*current_email).clone();
                emails.push(email);
                emails_to_share.set(emails);
                current_email.set(String::new());
            }
        })
    };
    let email_value = (*current_email).clone();

    let handle_share_onclosed = {
        let share_dialog_open = share_dialog_open.clone();
        Callback::from(move |_| share_dialog_open.set(false))
    };

    let display_emails = existing_shared_users
        .iter()
        .cloned()
        .chain(emails_to_share.iter().cloned());

    html! {
        <>
            <Dialog title={"Share Plan"} open={*share_dialog_open} onclosed={handle_share_onclosed}>
                <div id="share-plan-modal" class="mdc-dialog__content">
                    <p>{ "Share plan with the following users: "}</p>
                    <TextField classes={"mdc-text-field--filled"} value={email_value} nolabel={true} hint={"test@foo.com, j.bond@mi6.uk"} onchange={handle_email_change} onkeydown={handle_email_enter} />
                    <div class="email-chip-set">
                        {
                            display_emails
                                .map(|email| html! { <span class="email-chip mdc-theme--secondary-bg mdc-theme--on-secondary">{email}</span> })
                                .collect::<Vec<_>>()
                        }
                    </div>
                </div>
                <div class="mdc-dialog__actions">
                    <Button text={"Send"} style={Style::Raised} onclick={send_button_click} disabled={emails_to_share.is_empty()} />
                </div>
            </Dialog>
            { share_button }
        </>
    }
}
