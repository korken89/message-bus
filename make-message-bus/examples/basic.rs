use make_message_bus::make_message_bus;

//
// Topic defintion:
// TopicName [optional buffer size] => payload,
//
// Subtopic definition:
// module_name::SubtopicName => { ... },
//

make_message_bus!(
    bus::Toplevel => { // Toplevel topic
        Topic1 [10] => u8,
        Topic2 => u16,
        t1::SubTopic4 => {
            Topic5 [20] => u8,
            Topic6 => u16,
        },
        t2::SubTopic8 => {
            Topic9 [30] => u8,
            Topic10 => u16,
            t3::SubTopic12 => {
                Topic13 [40] => u8,
                Topic14 => u16,
            },
        },
    },
);

#[tokio::main]
async fn main() {
    // Subscirbe to all topics
    let mut sub_all = bus::Toplevel::subscribe();
    let mut sub_topic_13 = bus::t2::t3::Topic13::subscribe();

    // Publish on the bottom most topic
    bus::t2::t3::Topic13::publish(18);

    // Receive on the toplevel topic
    assert!(!sub_all.is_empty());
    let val = sub_all.try_recv().unwrap();

    println!("Toplevel val = {val:?}");

    assert!(matches!(
        val,
        bus::Toplevel::SubTopic8(bus::SubTopic8::SubTopic12(bus::t2::SubTopic12::Topic13(18)))
    ));

    // Receive on the specific topic
    assert!(!sub_topic_13.is_empty());
    let val = sub_topic_13.try_recv().unwrap();

    println!("Specific topic val = {val:?}");

    assert!(val == 18);
}
