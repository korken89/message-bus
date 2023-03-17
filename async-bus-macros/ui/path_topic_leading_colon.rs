use async_bus_macros::make_message_bus;

make_message_bus!(
    ::ohno::SomeData => some::Data,
);

fn main() {}
