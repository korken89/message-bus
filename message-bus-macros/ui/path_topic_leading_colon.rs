use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        ::ohno::SomeData => some::Data,
    }
);

fn main() {}
