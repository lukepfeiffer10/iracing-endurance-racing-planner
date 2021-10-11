﻿use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::worker::*;
use crate::overall_fuel_stint_config::OverallFuelStintConfigData;

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusInput {
    OverallFuelStintConfig(OverallFuelStintConfigData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusOutput {
    OverallFuelStintConfig(OverallFuelStintConfigData)
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = EventBusInput;
    type Output = EventBusOutput;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            EventBusInput::OverallFuelStintConfig(data) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::OverallFuelStintConfig(data.clone()));
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}