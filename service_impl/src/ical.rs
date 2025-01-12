use std::sync::Arc;

use service::{block::Block, ical::IcalService, ServiceError};
use time::macros::format_description;

pub struct IcalServiceImpl;

impl IcalService for IcalServiceImpl {
    fn convert_blocks_to_ical_string(
        &self,
        blocks: Arc<[Block]>,
    ) -> Result<Arc<str>, ServiceError> {
        let datetime_format = format_description!("[year][month][day]T[hour][minute][second]");

        let mut ical_string = String::new();
        ical_string.push_str("BEGIN:VCALENDAR\n");
        ical_string.push_str("VERSION:2.0\n");
        ical_string.push_str("PRODID:-//shifty/handcal//NONSGML v1.0//EN\n");
        for block in blocks.iter() {
            ical_string.push_str("BEGIN:VEVENT\n");
            ical_string.push_str(&format!("UID:{}\n", block.block_identifier()));
            ical_string.push_str(&format!(
                "DTSTART:{}\n",
                block.datetime_from()?.format(&datetime_format)?
            ));
            ical_string.push_str(&format!(
                "DTEND:{}\n",
                block.datetime_to()?.format(&datetime_format)?
            ));
            ical_string.push_str(&format!("SUMMARY:{}\n", "Shift"));
            ical_string.push_str("END:VEVENT\n");
        }
        ical_string.push_str("END:VCALENDAR\n");

        Ok(ical_string.into())
    }
}
