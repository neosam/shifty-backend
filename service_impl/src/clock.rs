use service::clock::ClockService;
use time::OffsetDateTime;

pub struct ClockServiceImpl;
impl ClockService for ClockServiceImpl {
    fn time_now(&self) -> time::Time {
        OffsetDateTime::now_utc().time()
    }
    fn date_now(&self) -> time::Date {
        OffsetDateTime::now_utc().date()
    }
    fn date_time_now(&self) -> time::PrimitiveDateTime {
        let now = OffsetDateTime::now_utc();
        time::PrimitiveDateTime::new(now.date(), now.time())
    }
}
