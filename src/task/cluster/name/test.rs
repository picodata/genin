use super::Name;

#[test]
fn name_serialize_deserialize() {
    let name = Name::from("storage").with_index(1).with_index('1');

    assert_eq!(name.to_string(), String::from("storage-1-1"));

    let de_name: Name = serde_yaml::from_str("storage-1-1").unwrap();

    assert_eq!(de_name, name);
}

#[test]
fn name_with_underscore() {
    let name = Name::from("consumer_eq").with_index(1).with_index('1');

    assert_eq!(name.to_string(), String::from("consumer_eq-1-1"));

    let de_name: Name = serde_yaml::from_str("consumer_eq-1-1").unwrap();

    assert_eq!(de_name, name);
}
