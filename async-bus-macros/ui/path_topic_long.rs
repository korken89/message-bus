use async_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        ohno::too_long::SomeData => some::Data,
    }
);

fn main() {}
