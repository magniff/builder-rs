#[test]
fn it_works() {
    #[derive(builder::Builder)]
    struct Human {
        name: String,
        age: u32,
    }

    let alice = Human::builder()
        .with_name("Alice".to_string())
        .with_age(20)
        .build();

    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.age, 20);
}
