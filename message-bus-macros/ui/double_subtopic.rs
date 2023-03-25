use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
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
    },
);

fn main() {}
