use crate::auth::{get_me, handle_auth_code_redirect, login, ID_TOKEN_KEY};
use crate::event_bus::{EventBus, EventBusInput};
use crate::landing::Landing;
use crate::loading::Loading;
use crate::planner::{Planner, RacePlannerProvider};
use endurance_racing_planner_common::GoogleOpenIdClaims;
use gloo_console::error;
use gloo_storage::{LocalStorage, Storage};
use jwt_compact::UntrustedToken;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Window};
use yew::prelude::*;
use yew_agent::Bridged;
use yew_mdc::components::{
    menu::Corner,
    top_app_bar::{section::Align, Section, TopAppBar},
    Drawer, DrawerContent, IconButton, Menu, MenuItem, TextField,
};
use yew_router::prelude::*;

#[macro_use]
extern crate dotenv_codegen;

mod auth;
mod bindings;
mod duration_serde;
mod event_bus;
mod http;
mod landing;
mod loading;
mod md_text_field;
mod overview;
mod planner;
mod roster;
mod schedule;

#[derive(Routable, Clone, Eq, PartialEq, Copy)]
enum AppRoutes {
    #[at("/")]
    Landing,
    #[at("/planner/*")]
    Planner,
}

impl Display for AppRoutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppRoutes::Landing => f.write_str("My Plans"),
            AppRoutes::Planner => f.write_str("Planner"),
        }
    }
}

fn switch(switch: &AppRoutes) -> Html {
    match switch {
        AppRoutes::Planner => {
            return html! {
                <RacePlannerProvider>
                    <Planner />
                </RacePlannerProvider>
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
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub user_info: Option<UserInfo>,
    pub nav_sidebar_open: bool,
    pub page_title: Option<String>,
}

pub enum AppStateAction {
    SetUser(Option<UserInfo>),
    SetSidebarOpen(bool),
    SetPageTitle(String),
}

impl Reducible for AppState {
    type Action = AppStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let clone = self.as_ref().clone();
        match action {
            AppStateAction::SetUser(user) => Self {
                user_info: user,
                ..clone
            },
            AppStateAction::SetSidebarOpen(value) => Self {
                nav_sidebar_open: value,
                ..clone
            },
            AppStateAction::SetPageTitle(title) => Self {
                page_title: Some(title),
                ..clone
            },
        }
        .into()
    }
}

pub type AppStateContext = UseReducerHandle<AppState>;

#[function_component(App)]
pub fn app() -> Html {
    let app_state_context = use_reducer(|| AppState {
        user_info: None,
        nav_sidebar_open: false,
        page_title: None,
    });
    let is_loading = use_state_eq(|| false);
    let nav_sidebar_open = app_state_context.nav_sidebar_open;
    let my_plans_onclick = {
        let app_state_context = app_state_context.clone();
        Callback::from(move |_| {
            app_state_context
                .dispatch(AppStateAction::SetPageTitle(AppRoutes::Landing.to_string()));
            app_state_context.dispatch(AppStateAction::SetSidebarOpen(false));
        })
    };

    let _handle_auth_redirect = {
        let app_state_context = app_state_context.clone();
        let is_loading = is_loading.clone();
        use_effect_with_deps(
            move |_| {
                if app_state_context.user_info.is_none() {
                    spawn_local(async move {
                        let token_result: gloo_storage::Result<String> =
                            LocalStorage::get(ID_TOKEN_KEY);
                        if let Ok(id_token) = token_result {
                            is_loading.set(true);
                            match get_me().await {
                                Ok(user) => {
                                    let parsed_token = UntrustedToken::new(&id_token).unwrap();
                                    let claims = parsed_token
                                        .deserialize_claims_unchecked::<GoogleOpenIdClaims>()
                                        .unwrap();
                                    is_loading.set(false);
                                    app_state_context.dispatch(AppStateAction::SetUser(Some(
                                        UserInfo {
                                            name: user.name,
                                            email: user.email,
                                            picture: claims.custom.picture,
                                        },
                                    )))
                                }
                                Err(_) => login(),
                            }
                        } else {
                            is_loading.set(true);
                            match handle_auth_code_redirect().await {
                                Ok(user) => {
                                    is_loading.set(false);
                                    if let Some(user) = user {
                                        app_state_context
                                            .dispatch(AppStateAction::SetUser(Some(user)));
                                        let window: Window =
                                            window().expect("no global `window` object exists");
                                        let location: web_sys::Location = window.location();
                                        location
                                            .set_hash("")
                                            .expect("url hash/fragment could not be reset");
                                    }
                                }
                                Err(e) => {
                                    is_loading.set(false);
                                    error!(e.to_string().as_str())
                                }
                            }
                        }
                    });
                };

                || ()
            },
            (),
        );
    };

    html! {
        <BrowserRouter>
            <ContextProvider<AppStateContext> context={app_state_context}>
                <Drawer id="nav-sidebar" dismissible={true} open={nav_sidebar_open}>
                    <DrawerContent>
                        <nav class="mdc-deprecated-list">
                            <div onclick={ my_plans_onclick }>
                                <Link<AppRoutes> to={AppRoutes::Landing} classes="mdc-deprecated-list-item">
                                    <span class="mdc-deprecated-list-item__ripple"></span>
                                    <i class="material-icons mdc-deprecated-list-item__graphic" aria-hidden="true">{ "view_list" }</i>
                                    <span class="mdc-deprecated-list-item__text">{ "My Plans" }</span>
                                </Link<AppRoutes>>
                            </div>
                        </nav>
                    </DrawerContent>
                </Drawer>
                <div class="wrapper flex-container flex-column mdc-drawer-app-content">
                    <Header />
                    {
                        if *is_loading  {
                            html! { <Loading /> }
                        } else {
                            html! { <Switch<AppRoutes> render={Switch::render(switch)} /> }
                        }
                    }
                </div>
            </ContextProvider<AppStateContext>>
        </BrowserRouter>
    }
}

struct HeaderState {
    profile_menu_open: bool,
    is_editing_title: bool,
    page_title: String,
}

#[function_component(Header)]
pub fn header() -> Html {
    let app_state_context = use_context::<AppStateContext>().unwrap();
    let current_route: AppRoutes = use_route().unwrap();
    let state = use_state(|| HeaderState {
        profile_menu_open: false,
        is_editing_title: false,
        page_title: app_state_context
            .page_title
            .clone()
            .unwrap_or_else(|| current_route.to_string()),
    });
    let event_bus = use_mut_ref(|| EventBus::bridge(Callback::noop()));

    let profile_picture_onclick = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(HeaderState {
                profile_menu_open: !state.profile_menu_open,
                page_title: state.page_title.clone(),
                ..*state
            })
        })
    };

    let profile_menu_onclose = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(HeaderState {
                profile_menu_open: false,
                page_title: state.page_title.clone(),
                ..*state
            })
        })
    };

    let top_bar_menu_onclick = {
        let app_state_context = app_state_context.clone();
        Callback::from(move |_| {
            app_state_context.dispatch(AppStateAction::SetSidebarOpen(
                !app_state_context.nav_sidebar_open,
            ))
        })
    };

    if current_route == AppRoutes::Landing && app_state_context.user_info.is_none() {
        return html! {};
    } else {
        let profile_picture_section = if let Some(user) = &app_state_context.user_info {
            html! {
                <Section align={Align::End}>
                    <div class="mdc-menu-surface--anchor">
                        <img id="profile-picture" class="mdc-top-app-bar__action-item" src={user.picture.clone()} onclick={profile_picture_onclick} />
                        <Menu open={state.profile_menu_open} onclose={profile_menu_onclose} corner={Corner::BottomLeft} fixed_position={true}>
                            <MenuItem text={user.name.clone()} />
                            <MenuItem text={user.email.clone()} />
                        </Menu>
                    </div>
                </Section>
            }
        } else {
            html! {}
        };

        let page_title = app_state_context
            .page_title
            .clone()
            .unwrap_or_else(|| current_route.to_string());
        let page_title_html = {
            if state.is_editing_title {
                let page_title = state.page_title.clone();

                let title_change = {
                    let state = state.clone();
                    Callback::from(move |value| {
                        state.set(HeaderState {
                            is_editing_title: true,
                            page_title: value,
                            ..*state
                        })
                    })
                };
                html! {
                    <span class="mdc-top-app-bar__title">
                        <TextField value={ page_title.clone() } onchange={ title_change } classes={ "header-text-field" } />
                    </span>
                }
            } else {
                html! {
                    <span class="mdc-top-app-bar__title">{ page_title.clone() }</span>
                }
            }
        };

        let edit_button = if current_route == AppRoutes::Planner {
            if state.is_editing_title {
                let done_button_click = {
                    let app_state = app_state_context.clone();
                    let state = state.clone();
                    Callback::from(move |_| {
                        state.set(HeaderState {
                            is_editing_title: false,
                            page_title: state.page_title.clone(),
                            ..*state
                        });
                        (*event_bus.borrow_mut())
                            .send(EventBusInput::PutPlannerTitle(state.page_title.clone()));
                        app_state.dispatch(AppStateAction::SetPageTitle(state.page_title.clone()))
                    })
                };
                let cancel_button_click = {
                    let state = state.clone();
                    Callback::from(move |_| {
                        state.set(HeaderState {
                            is_editing_title: false,
                            page_title: page_title.clone(),
                            ..*state
                        })
                    })
                };
                html! {
                    <>
                        <IconButton classes="material-icons mdc-top-app-bar__navigation-icon" onclick={done_button_click}>{ "done" }</IconButton>
                        <IconButton classes="material-icons mdc-top-app-bar__navigation-icon" onclick={cancel_button_click}>{ "cancel" }</IconButton>
                    </>
                }
            } else {
                let edit_button_click = {
                    let state = state.clone();
                    Callback::from(move |_| {
                        state.set(HeaderState {
                            is_editing_title: true,
                            page_title: page_title.clone(),
                            ..*state
                        })
                    })
                };
                html! {
                    <IconButton classes="material-icons mdc-top-app-bar__navigation-icon" onclick={edit_button_click}>{ "edit" }</IconButton>
                }
            }
        } else {
            html! {}
        };

        return html! {
            <TopAppBar id="app-header">
                <Section>
                    <IconButton classes="material-icons mdc-top-app-bar__navigation-icon" onclick={top_bar_menu_onclick}>{ "menu" }</IconButton>
                    { page_title_html }
                    { edit_button }
                </Section>
                { profile_picture_section }
            </TopAppBar>
        };
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    yew::start_app::<App>();
}
