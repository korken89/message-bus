use async_bus_macros::make_message_bus;

make_message_bus!(
    ohno::too_long::SomeData => some::Data,
);

fn main() {}
