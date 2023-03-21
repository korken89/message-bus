use async_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel1 => {
        SomeData1 => i32,
    },
    bus::Toplevel2 => {
        SomeData2 => u32,
    },
);

fn main() {}
