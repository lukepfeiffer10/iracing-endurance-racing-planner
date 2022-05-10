use crate::auth::login;
use crate::http::plans::get_plans;
use crate::planner::PlannerRoutes;
use crate::{AppStateContext, UserInfo};
use endurance_racing_planner_common::RacePlannerDto;
use uuid::Uuid;
use yew::context::ContextHandle;
use yew::prelude::*;
use yew::{Component, Html};
use yew_mdc::components::list::item::LeadingType;
use yew_mdc::components::{Card, List, ListItem, PrimaryAction};
use yew_router::prelude::*;

pub struct Landing {
    google_login_image: String,
    user: Option<UserInfo>,
    _app_state_context_handle: ContextHandle<AppStateContext>,
    my_plans: Vec<RacePlannerDto>,
}

#[derive(Clone)]
pub enum MouseEventType {
    Over,
    Out,
    Down,
    Up,
}

pub enum LandingMsg {
    OnMouseEvent(MouseEventType),
    OnLoginClick,
    OnAppStateContextUpdate(AppStateContext),
    UpdatePlans(Vec<RacePlannerDto>),
    OpenPlan(Uuid),
}

fn render_plan(plan: &RacePlannerDto, ctx: &Context<Landing>) -> Html {
    let id = plan.id;
    let onclick = ctx.link().callback(move |_| LandingMsg::OpenPlan(id));
    html! {
        <ListItem
            text={vec![plan.title.clone()]}
            leading_item={Some(LeadingType::Icon("event".into()))}
            onclick={onclick} />
    }
}

impl Component for Landing {
    type Message = LandingMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_state_context, context_listener) = ctx
            .link()
            .context::<AppStateContext>(ctx.link().callback(LandingMsg::OnAppStateContextUpdate))
            .expect("No App State Context Provided");

        if app_state_context.user_info.is_some() {
            get_plans(ctx.link().callback(LandingMsg::UpdatePlans));
        }

        Self {
            google_login_image: "btn_google_signin_light_normal_web.png".to_string(),
            user: app_state_context.user_info.clone(),
            _app_state_context_handle: context_listener,
            my_plans: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LandingMsg::OnMouseEvent(event_type) => {
                self.google_login_image = match event_type {
                    MouseEventType::Over => "btn_google_signin_light_focus_web.png".to_string(),
                    MouseEventType::Out | MouseEventType::Up => {
                        "btn_google_signin_light_normal_web.png".to_string()
                    }
                    MouseEventType::Down => "btn_google_signin_light_pressed_web.png".to_string(),
                };
                true
            }
            LandingMsg::OnLoginClick => {
                login();
                false
            }
            LandingMsg::OnAppStateContextUpdate(app_state_context) => {
                if self.user != app_state_context.user_info {
                    self.user = app_state_context.user_info.clone();
                    get_plans(ctx.link().callback(LandingMsg::UpdatePlans));
                    true
                } else {
                    false
                }
            }
            LandingMsg::UpdatePlans(my_plans) => {
                self.my_plans = my_plans;
                true
            }
            LandingMsg::OpenPlan(plan_id) => {
                ctx.link()
                    .history()
                    .unwrap()
                    .push(PlannerRoutes::Overview { id: plan_id });
                false
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let mouse_events = |event_type: MouseEventType| {
            link.callback(move |_event: MouseEvent| LandingMsg::OnMouseEvent(event_type.clone()))
        };
        let on_mouse_over = mouse_events(MouseEventType::Over);
        let on_mouse_out = mouse_events(MouseEventType::Out);
        let on_mouse_down = mouse_events(MouseEventType::Down);
        let on_mouse_up = mouse_events(MouseEventType::Up);
        let on_login_click = link.callback(|_| LandingMsg::OnLoginClick);

        match &self.user {
            Some(_) => {
                let new_plan_click = {
                    let history = link.history().unwrap();
                    Callback::from(move |_| {
                        history.push(PlannerRoutes::Overview { id: Uuid::nil() })
                    })
                };
                return html! {
                    <div class="content">
                        <Card classes="plan-card">
                            <PrimaryAction onclick={ new_plan_click }>
                                <i class="material-icons">{ "add" }</i>
                                <span>{ "New Plan" }</span>
                            </PrimaryAction>
                        </Card>
                        <List>
                            {
                                self.my_plans
                                    .iter()
                                    .map(|plan| render_plan(plan, ctx))
                                    .collect::<Vec<_>>()
                            }
                        </List>
                    </div>
                };
            }
            None => {
                return html! {
                    <div id="login-content" class="flex-container flex-column">
                        <div id="login-card" class="mdc-card">
                            <div class="mdc-card-wrapper__text-section">
                                <div class="card-title">{ "Login" }</div>
                            </div>
                            <img src={format!("images/{}", self.google_login_image)} alt="Sign in with Google" width="191" height="46"
                                onmouseover={on_mouse_over}
                                onmouseout={on_mouse_out}
                                onmousedown={on_mouse_down}
                                onmouseup={on_mouse_up}
                                onclick={on_login_click} />
                        </div>
                    </div>
                }
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
}
