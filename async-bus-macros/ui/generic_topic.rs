use async_bus_macros::make_message_bus;

make_message_bus!(
    SomeData<u32> => some::Data,
);

fn main() {}
