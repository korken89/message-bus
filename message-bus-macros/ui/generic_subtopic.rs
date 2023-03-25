use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        sub::Topic<u32> => {
            SomeData => u32,
        },
    }
);

fn main() {}
