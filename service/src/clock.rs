use mockall::automock;

#[automock]
pub trait ClockService {
    fn time_now(&self) -> time::Time;
    fn date_now(&self) -> time::Date;
    fn date_time_now(&self) -> time::PrimitiveDateTime;
}
