use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        SomeData1 => u32,
    },
    SomeData2 => i32,
);

fn main() {}
