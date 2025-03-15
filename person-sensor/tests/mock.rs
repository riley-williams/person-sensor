mod common;
use common::MockPersonSensorBus;
use person_sensor::{PersonID, PersonSensorBuilder};

const NO_FACES: [u8; 39] = [
    0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x25, 0x37,
];
const ONE_FACE: [u8; 39] = [
    0x00, 0x00, 0x21, 0x00, 0x01, 0x63, 0x7c, 0x80, 0x95, 0xaa, 0x43, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x8b,
];
const TWO_FACES: [u8; 39] = [
    0x00, 0x00, 0x21, 0x00, 0x02, 0x63, 0x3e, 0x5e, 0x62, 0x9e, 0x4e, 0x00, 0x01, 0x5e, 0x79, 0x67,
    0x8e, 0x88, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0xb9, 0xf9,
];
const BAD_CHECKSUM: [u8; 39] = [
    0x00, 0x00, 0x21, 0x00, 0x02, 0x63, 0x3e, 0x5e, 0x62, 0x9e, 0x4e, 0x00, 0x01, 0x5e, 0x79, 0x67,
    0x8e, 0x88, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0xb9, 0xf9,
];

#[tokio::test]
async fn no_faces() {
    let i2c = MockPersonSensorBus::new(1, &NO_FACES);

    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();
    let detections = person_sensor.get_detections().await.unwrap();
    assert_eq!(detections.len(), 0);
}

#[tokio::test]
async fn one_face() {
    let i2c = MockPersonSensorBus::new(1, &ONE_FACE);

    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();
    let detections = person_sensor.get_detections().await.unwrap();
    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].box_confidence, 0x63);
    assert_eq!(detections[0].box_left, 0x7c);
    assert_eq!(detections[0].box_top, 0x80);
    assert_eq!(detections[0].box_right, 0x95);
    assert_eq!(detections[0].box_bottom, 0xaa);
    assert_eq!(detections[0].id_confidence, 0x43);
    assert_eq!(detections[0].id, Some(PersonID::new(0).unwrap()));
    assert!(detections[0].is_facing);
}

#[tokio::test]
async fn two_faces() {
    let i2c = MockPersonSensorBus::new(1, &TWO_FACES);
    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();
    let detections = person_sensor.get_detections().await.unwrap();
    assert_eq!(detections.len(), 2);

    // First face
    assert_eq!(detections[0].box_confidence, 0x63);
    assert_eq!(detections[0].box_left, 0x3e);
    assert_eq!(detections[0].box_top, 0x5e);
    assert_eq!(detections[0].box_right, 0x62);
    assert_eq!(detections[0].box_bottom, 0x9e);
    assert_eq!(detections[0].id_confidence, 0x4e);
    assert_eq!(detections[0].id, Some(PersonID::new(0).unwrap()));
    assert!(detections[0].is_facing);

    // Second face
    assert_eq!(detections[1].box_confidence, 0x5e);
    assert_eq!(detections[1].box_left, 0x79);
    assert_eq!(detections[1].box_top, 0x67);
    assert_eq!(detections[1].box_right, 0x8e);
    assert_eq!(detections[1].box_bottom, 0x88);
    assert_eq!(detections[1].id_confidence, 0x38);
    assert_eq!(detections[1].id, Some(PersonID::new_unchecked(0)));
    assert!(detections[1].is_facing);
}

#[tokio::test]
async fn bad_checksum_continuous() {
    let i2c = MockPersonSensorBus::new(1, &BAD_CHECKSUM);

    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();

    if let Err(person_sensor::ReadError::ChecksumMismatch) = person_sensor.get_detections().await {
    } else {
        panic!("Expected ChecksumMismatch error");
    }
}

#[tokio::test]
async fn bad_checksum_standby() {
    let i2c = MockPersonSensorBus::new(1, &BAD_CHECKSUM);

    let mut person_sensor = PersonSensorBuilder::new_standby(i2c, false)
        .build()
        .await
        .unwrap();

    if Err(person_sensor::ReadError::ChecksumMismatch) == person_sensor.capture_once().await {
    } else {
        panic!("Expected ChecksumMismatch error");
    }
}

#[tokio::test]
async fn bad_checksum_ignored_when_disabled() {
    let i2c = MockPersonSensorBus::new(1, &BAD_CHECKSUM);

    let mut person_sensor = PersonSensorBuilder::new_standby(i2c, false)
        .build()
        .await
        .unwrap();
    person_sensor.set_checksum_enabled(false);

    _ = person_sensor.capture_once().await.unwrap();
}

#[tokio::test]
async fn set_mode_on_init() {
    let i2c = MockPersonSensorBus::new(0, &NO_FACES);

    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();
    _ = person_sensor.get_detections().await.unwrap();

    let i2c = MockPersonSensorBus::new(1, &NO_FACES);

    let mut person_sensor = PersonSensorBuilder::new_standby(i2c, false)
        .build()
        .await
        .unwrap();
    _ = person_sensor.capture_once().await.unwrap();
}

#[tokio::test]
async fn switch_mode() {
    let i2c = MockPersonSensorBus::new(0, &NO_FACES);

    let mut person_sensor = PersonSensorBuilder::new_continuous(i2c, false)
        .build()
        .await
        .unwrap();
    _ = person_sensor.get_detections().await.unwrap();

    let mut person_sensor = person_sensor.into_standby_mode().await.unwrap();
    _ = person_sensor.capture_once().await.unwrap();

    let mut person_sensor = person_sensor.into_continuous_mode().await.unwrap();
    _ = person_sensor.get_detections().await.unwrap();
}
