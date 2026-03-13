use dao::BasicDao;
use rest::RestStateDef;
use service::extra_hours::ExtraHoursService;
use service::permission::Authentication;
use service::sales_person::SalesPersonService;
use service::special_days::SpecialDayService;

use crate::integration_test::TestSetup;

#[tokio::test]
async fn test_seed_on_empty_database() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let auth = Authentication::Full;

    // Seed data
    rest::dev::seed_dev_data_impl(rest_state).await.unwrap();

    // Verify sales persons
    let sales_persons = rest_state
        .sales_person_service()
        .get_all(auth.clone(), None)
        .await
        .unwrap();
    assert_eq!(sales_persons.len(), 5);

    let names: Vec<&str> = sales_persons.iter().map(|sp| sp.name.as_ref()).collect();
    assert!(names.contains(&"Anna Müller"));
    assert!(names.contains(&"Max Schmidt"));
    assert!(names.contains(&"Lisa Weber"));
    assert!(names.contains(&"Tom Bauer"));
    assert!(names.contains(&"Sarah Fischer"));

    // Verify inactive person
    let lisa = sales_persons.iter().find(|sp| sp.name.as_ref() == "Lisa Weber").unwrap();
    assert!(lisa.inactive);

    // Verify extra hours
    let now = time::OffsetDateTime::now_utc();
    let today = now.date();
    let (year, week, _) = today.to_iso_week_date();
    let extra_hours = rest_state
        .extra_hours_service()
        .find_by_week(year as u32, week, auth.clone(), None)
        .await
        .unwrap();
    assert_eq!(extra_hours.len(), 3);

    // Verify special days (KW 14)
    let special_days = rest_state
        .special_day_service()
        .get_by_week(year as u32, 14, auth.clone())
        .await
        .unwrap();
    assert_eq!(special_days.len(), 2); // Karfreitag + Ostermontag
}

#[tokio::test]
async fn test_clear_after_seed() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let auth = Authentication::Full;

    // Seed then clear
    rest::dev::seed_dev_data_impl(rest_state).await.unwrap();
    rest_state.basic_dao().clear_all().await.unwrap();

    // Verify everything is empty
    let sales_persons = rest_state
        .sales_person_service()
        .get_all(auth.clone(), None)
        .await
        .unwrap();
    assert_eq!(sales_persons.len(), 0);
}

#[tokio::test]
async fn test_clear_on_empty_database() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;

    // Clear on empty should succeed
    rest_state.basic_dao().clear_all().await.unwrap();
}

#[tokio::test]
async fn test_seed_twice_is_additive() {
    let test_setup = TestSetup::new().await;
    let rest_state = &test_setup.rest_state;
    let auth = Authentication::Full;

    // Seed twice
    rest::dev::seed_dev_data_impl(rest_state).await.unwrap();
    rest::dev::seed_dev_data_impl(rest_state).await.unwrap();

    // Should have 10 sales persons (5 + 5)
    let sales_persons = rest_state
        .sales_person_service()
        .get_all(auth.clone(), None)
        .await
        .unwrap();
    assert_eq!(sales_persons.len(), 10);
}
