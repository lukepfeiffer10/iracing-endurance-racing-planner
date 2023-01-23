use uuid::Uuid;

#[derive(Debug)]
pub struct Driver {
    pub id: i32,
    pub plan_id: Uuid,
    pub name: String,
    pub color: String,
    pub utc_offset: i16,
    pub irating: i16,
    pub stint_preference: i16,
}

impl Driver {
    pub fn create(d: endurance_racing_planner_common::Driver, plan_id: Uuid) -> Self {
        Driver {
            id: 0,
            plan_id,
            name: d.name,
            color: d.color,
            utc_offset: d.utc_offset,
            irating: d.irating,
            stint_preference: d.stint_preference,
        }
    }
}

impl Into<endurance_racing_planner_common::Driver> for Driver {
    fn into(self) -> endurance_racing_planner_common::Driver {
        (&self).into()
    }
}

impl Into<endurance_racing_planner_common::Driver> for &Driver {
    fn into(self) -> endurance_racing_planner_common::Driver {
        endurance_racing_planner_common::Driver {
            id: self.id,
            name: self.name.clone(),
            total_stints: 0,
            fair_share: false,
            color: self.color.clone(),
            utc_offset: self.utc_offset,
            irating: self.irating,
            stint_preference: self.stint_preference,
        }
    }
}
