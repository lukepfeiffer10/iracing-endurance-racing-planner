use std::fmt::{Display, Formatter};
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::prelude::*;
use yew_mdc::components::{top_app_bar::{TopAppBar, Section, section::Align}, IconButton};
use crate::planner::Planner;
use crate::landing::{Landing};

mod bindings;
mod md_text_field;
mod event_bus;
mod duration_serde;
mod overview;
mod roster;
mod schedule;
mod landing;
mod planner;

#[derive(Routable,Clone,Eq,PartialEq,Copy)]
enum AppRoutes {
    #[at("/")]
    Landing,
    #[at("/planner/:s")]
    Planner,
}

impl Display for AppRoutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppRoutes::Landing => f.write_str("My Plans"),
            AppRoutes::Planner => f.write_str("Planner")
        }
    }
}

fn switch(switch: &AppRoutes) -> Html {
    match switch {
        AppRoutes::Planner => {
            return html! {
                <Planner />
            }
        }
        AppRoutes::Landing => {
            return html! {
                <Landing />
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub email: String,
    pub picture: String,
    pub name: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInfoMessage {
    pub user_info: Option<UserInfo>
}

impl Reducible for UserInfoMessage {
    type Action = Option<UserInfo>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Self {
            user_info: action
        }.into()
    }
}

pub type UserInfoContext = UseReducerHandle<UserInfoMessage>;

#[function_component(App)]
pub fn app() -> Html {
    let user_info_context = use_reducer(|| UserInfoMessage { user_info: None });
    html! {
        <BrowserRouter>
            <ContextProvider<UserInfoContext> context={user_info_context}>
                <div class="wrapper flex-container flex-column">
                    <Header />
                    <Switch<AppRoutes> render={Switch::render(switch)} />
                </div>
            </ContextProvider<UserInfoContext>>
        </BrowserRouter>
    }
}

#[function_component(Header)]
pub fn header() -> Html {
    let user_info_context = use_context::<UserInfoContext>().unwrap();
    let current_route: AppRoutes = use_route().unwrap();
    if current_route == AppRoutes::Landing && user_info_context.user_info.is_none() {
        return html! {}
    }
    else {
        let user_section = if let Some(user) = &user_info_context.user_info {
            html! {                
                <Section align={Align::End}>                    
                    <img id="profile-picture" src={user.picture.clone()} />
                    <span>{ user.email.clone() } { user.name.clone() }</span>
                </Section>
            }
        } else {
            html! {}
        };
        
        return html! {
            <TopAppBar id="app-header">
                <Section>
                    <IconButton classes="material-icons mdc-top-app-bar__navigation-icon">{ "menu" }</IconButton>
                    <span class="mdc-top-app-bar__title">{ current_route }</span>
                </Section>
                { user_section }
            </TopAppBar>
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    yew::start_app::<App>();
}