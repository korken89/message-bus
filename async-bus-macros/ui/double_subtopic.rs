use async_bus_macros::make_message_bus;

make_message_bus!(
    sub::Topic => { // Err
        SomeData => u32,
    },
    sub::Topic => { // Err
        SomeData => u32,
    },
    sub::TopicOk => { // Ok
        SomeData => u32,
        sub::TopicErr => { // Err
            SomeData => u32,
        },
        sub::TopicErr => { // Err
            SomeData => u32,
        },
    },
);

fn main() {}
