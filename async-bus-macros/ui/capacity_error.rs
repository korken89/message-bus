use async_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        SomeData1 [123] => u32, // Ok
        SomeData2 [0] => i32, // Err
    },
);

fn main() {}
