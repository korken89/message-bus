use message_bus_macros::make_message_bus;

make_message_bus!(
    bus::Toplevel => {
        SomeData<u32> => some::Data,
    }
);

fn main() {}
