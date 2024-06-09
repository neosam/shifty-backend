use crate::{api, error::ShiftyError, state::Shiftplan};

pub async fn load_shift_plan() -> Result<Shiftplan, ShiftyError> {
    let slot_tos = api::get_slots().await?;
    Ok(Shiftplan {
        week: 0,
        year: 0,
        slots: slot_tos.iter().map(|slot_to| slot_to.into()).collect(),
    })
}
